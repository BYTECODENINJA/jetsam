//! The Unix-socket IPC server.
//!
//! One JSON [`Request`] per line in, one JSON [`Response`] per line out. The
//! socket lives at `$XDG_RUNTIME_DIR/jetsam/daemon.sock`, is created `0600`, and
//! the runtime directory is `0700`, so only the owning user may connect.

use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::Arc;

use jetsam_core::protocol::{
    ApiError, DaemonStatus, Method, OkData, Payload, Request, Response, UsageInfo, PROTOCOL_VERSION,
};

use crate::error::{DaemonError, Result};
use crate::{logging, now_epoch, paths, SharedState};

/// Maximum accepted request line length (1 MiB) to bound memory on malformed or
/// hostile input.
const MAX_LINE: u64 = 1024 * 1024;

/// Bind the daemon socket, refusing to start a second daemon.
pub fn bind() -> Result<UnixListener> {
    let dir = paths::runtime_dir();
    std::fs::create_dir_all(&dir)?;
    let mut perms = std::fs::metadata(&dir)?.permissions();
    perms.set_mode(0o700);
    std::fs::set_permissions(&dir, perms)?;

    let path = paths::socket_path();
    if path.exists() {
        if UnixStream::connect(&path).is_ok() {
            return Err(DaemonError::Startup(format!(
                "another daemon is already listening on {}",
                path.display()
            )));
        }
        // Stale socket from a crashed daemon.
        let _ = std::fs::remove_file(&path);
    }

    let listener = UnixListener::bind(&path)?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    Ok(listener)
}

/// Serve connections until the process is stopped. Blocks.
pub fn serve(state: Arc<SharedState>) -> Result<()> {
    let listener = bind()?;
    serve_with(listener, state)
}

/// Serve on an already-bound listener, reporting the socket path.
pub fn serve_with(listener: UnixListener, state: Arc<SharedState>) -> Result<()> {
    logging::info(format!(
        "listening on {} (protocol v{PROTOCOL_VERSION})",
        paths::socket_path().display()
    ));
    // Restrict to sequential handling: the workload is tiny and this keeps the
    // store's single writer trivially safe. Each connection is short-lived.
    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                if let Err(err) = handle_stream(&state, stream) {
                    logging::warn(format!("connection error: {err}"));
                }
            }
            Err(err) => logging::warn(format!("accept error: {err}")),
        }
    }
    Ok(())
}

/// Handle a single connection: read requests, dispatch, write responses.
pub fn handle_stream(state: &SharedState, stream: UnixStream) -> Result<()> {
    let reader = BufReader::new(&stream);
    let mut writer = stream.try_clone()?;
    let mut line = String::new();
    let mut limited = reader.take(MAX_LINE);
    loop {
        line.clear();
        let read = limited.read_line(&mut line)?;
        if read == 0 {
            return Ok(());
        }
        let response = match serde_json::from_str::<Request>(line.trim_end()) {
            Ok(request) => dispatch(state, &request),
            Err(err) => Response {
                v: PROTOCOL_VERSION,
                id: 0,
                payload: Payload::Err(ApiError {
                    code: "bad_request".to_string(),
                    message: err.to_string(),
                }),
            },
        };
        let mut text = serde_json::to_string(&response)?;
        text.push('\n');
        writer.write_all(text.as_bytes())?;
        writer.flush()?;
    }
}

/// Dispatch a fully parsed request.
pub fn dispatch(state: &SharedState, request: &Request) -> Response {
    if request.v != PROTOCOL_VERSION {
        return Response::err(
            request.id,
            "protocol_mismatch",
            format!(
                "daemon speaks protocol v{PROTOCOL_VERSION}, client sent v{}",
                request.v
            ),
        );
    }
    let id = request.id;
    match &request.method {
        Method::Ping => Response::ok(id, OkData::Pong),
        Method::Status => match build_status(state) {
            Ok(status) => Response::ok(id, OkData::Status(status)),
            Err(err) => internal(id, err),
        },
        Method::PackageUsage { name } => match state.lock_store() {
            Ok(store) => match store.usage(name) {
                Ok(Some(info)) => Response::ok(id, OkData::Usage(info)),
                Ok(None) => Response::ok(id, OkData::Usage(UsageInfo::empty(name.clone()))),
                Err(err) => internal(id, err),
            },
            Err(err) => internal(id, err),
        },
        Method::TopUsage { limit } => {
            let limit = (*limit).clamp(1, 10_000);
            match state.lock_store() {
                Ok(store) => match store.top_usage(limit) {
                    Ok(list) => Response::ok(id, OkData::UsageList(list)),
                    Err(err) => internal(id, err),
                },
                Err(err) => internal(id, err),
            }
        }
        Method::UsageMap => match state.lock_store() {
            Ok(store) => match store.usage_map() {
                Ok(map) => Response::ok(id, OkData::UsageMap(map)),
                Err(err) => internal(id, err),
            },
            Err(err) => internal(id, err),
        },
        Method::IgnoreList => match state.lock_store() {
            Ok(store) => match store.ignored() {
                Ok(names) => Response::ok(id, OkData::Names(names)),
                Err(err) => internal(id, err),
            },
            Err(err) => internal(id, err),
        },
        Method::SetIgnored { name, ignored } => match state.lock_store() {
            Ok(store) => match store.set_ignored(name, *ignored) {
                Ok(()) => Response::ok(id, OkData::Ack),
                Err(err) => internal(id, err),
            },
            Err(err) => internal(id, err),
        },
        Method::SetOptIn { enabled } => set_opt_in(state, id, *enabled),
        Method::Wipe => match state.lock_store() {
            Ok(store) => match store.wipe() {
                Ok(()) => Response::ok(id, OkData::Ack),
                Err(err) => internal(id, err),
            },
            Err(err) => internal(id, err),
        },
    }
}

fn set_opt_in(state: &SharedState, id: u64, enabled: bool) -> Response {
    match state.lock_store() {
        Ok(store) => {
            if let Err(err) = store.set_opt_in(enabled) {
                return internal(id, err);
            }
        }
        Err(err) => return internal(id, err),
    }
    // Persist to the config file so the choice survives a restart.
    if let Ok(mut config) = state.config.lock() {
        config.opted_in = enabled;
        if let Err(err) = config.save(&state.config_path) {
            logging::warn(format!("could not persist config: {err}"));
        }
    }
    Response::ok(id, OkData::Ack)
}

fn build_status(state: &SharedState) -> Result<DaemonStatus> {
    let store = state.lock_store()?;
    let config = state.lock_config()?;
    let now = now_epoch();
    Ok(DaemonStatus {
        daemon_version: state.daemon_version.to_string(),
        protocol_version: PROTOCOL_VERSION,
        opted_in: store.is_opted_in()?,
        sampling_interval_secs: config.sampling_interval_secs,
        pss_interval_secs: config.pss_interval_secs,
        packages_observed: store.packages_observed()?,
        ignored_count: store.ignored_count()?,
        first_sample_epoch: store.first_sample_epoch()?,
        last_sample_epoch: store.last_sample_epoch()?,
        days_of_history: store.days_of_history()?,
        started_at_epoch: state.started_at,
        uptime_secs: now.saturating_sub(state.started_at).max(0) as u64,
    })
}

fn internal(id: u64, err: DaemonError) -> Response {
    Response::err(id, "internal", err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::store::Store;

    fn state() -> SharedState {
        SharedState::new(
            Store::in_memory().unwrap(),
            Config::default(),
            std::path::PathBuf::from("/nonexistent/config.toml"),
            "0.0.0-test",
            1_000,
        )
    }

    fn call(state: &SharedState, method: Method) -> OkData {
        let response = dispatch(state, &Request::new(7, method));
        assert_eq!(response.id, 7);
        match response.payload {
            Payload::Ok(data) => data,
            Payload::Err(err) => panic!("unexpected error: {err:?}"),
        }
    }

    #[test]
    fn ping_returns_pong() {
        let state = state();
        assert_eq!(call(&state, Method::Ping), OkData::Pong);
    }

    #[test]
    fn status_reports_version_and_defaults() {
        let state = state();
        let OkData::Status(status) = call(&state, Method::Status) else {
            panic!("expected status");
        };
        assert_eq!(status.protocol_version, PROTOCOL_VERSION);
        assert_eq!(status.daemon_version, "0.0.0-test");
        assert!(!status.opted_in);
        assert_eq!(status.packages_observed, 0);
    }

    #[test]
    fn version_mismatch_is_rejected() {
        let state = state();
        let request = Request {
            v: PROTOCOL_VERSION + 1,
            id: 3,
            method: Method::Ping,
        };
        let response = dispatch(&state, &request);
        match response.payload {
            Payload::Err(err) => assert_eq!(err.code, "protocol_mismatch"),
            other => panic!("expected error, got {other:?}"),
        }
    }

    #[test]
    fn package_usage_unknown_is_zeroed() {
        let state = state();
        let OkData::Usage(info) = call(
            &state,
            Method::PackageUsage {
                name: "ghost".to_string(),
            },
        ) else {
            panic!("expected usage");
        };
        assert_eq!(info.name, "ghost");
        assert_eq!(info.launches, 0);
    }

    #[test]
    fn set_opt_in_mutates_store() {
        let state = state();
        assert_eq!(
            call(&state, Method::SetOptIn { enabled: true }),
            OkData::Ack
        );
        let OkData::Status(status) = call(&state, Method::Status) else {
            panic!("expected status");
        };
        assert!(status.opted_in);
    }

    #[test]
    fn set_and_get_ignore_list() {
        let state = state();
        call(
            &state,
            Method::SetIgnored {
                name: "foo".to_string(),
                ignored: true,
            },
        );
        let OkData::Names(names) = call(&state, Method::IgnoreList) else {
            panic!("expected names");
        };
        assert_eq!(names, vec!["foo".to_string()]);
    }

    #[test]
    fn top_usage_limit_is_clamped_not_panicking() {
        let state = state();
        let OkData::UsageList(list) = call(&state, Method::TopUsage { limit: 0 }) else {
            panic!("expected list");
        };
        assert!(list.is_empty());
    }
}
