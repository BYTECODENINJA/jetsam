//! A blocking client for the daemon's Unix-socket protocol.
//!
//! Used by the CLI and by the Tauri backend. The GUI never talks to the daemon
//! directly; it goes through this client in the Rust process.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

use jetsam_core::protocol::{Method, OkData, Payload, Request, Response, PROTOCOL_VERSION};

use crate::error::{DaemonError, Result};
use crate::paths;

/// A connected client.
pub struct Client {
    stream: UnixStream,
    reader: BufReader<UnixStream>,
    next_id: u64,
}

impl Client {
    /// Connect to the daemon socket at an explicit path.
    pub fn connect(path: &Path) -> Result<Self> {
        let stream = UnixStream::connect(path)?;
        Self::from_stream(stream)
    }

    /// Connect to the daemon socket in the current user's runtime directory.
    pub fn connect_default() -> Result<Self> {
        Self::connect(&paths::socket_path())
    }

    /// Wrap an already-connected stream (used in tests).
    pub fn from_stream(stream: UnixStream) -> Result<Self> {
        let reader = BufReader::new(stream.try_clone()?);
        Ok(Client {
            stream,
            reader,
            next_id: 1,
        })
    }

    /// The socket path a default client would use.
    pub fn default_socket_path() -> PathBuf {
        paths::socket_path()
    }

    /// Send a request and return the full response.
    pub fn call(&mut self, method: Method) -> Result<Response> {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let request = Request::new(id, method);
        let mut text = serde_json::to_string(&request)?;
        text.push('\n');
        self.stream.write_all(text.as_bytes())?;
        self.stream.flush()?;

        let mut line = String::new();
        let read = self.reader.read_line(&mut line)?;
        if read == 0 {
            return Err(DaemonError::Protocol(
                "daemon closed the connection".to_string(),
            ));
        }
        let response: Response = serde_json::from_str(line.trim_end())?;
        if response.v != PROTOCOL_VERSION {
            return Err(DaemonError::Protocol(format!(
                "daemon speaks protocol v{}, client v{PROTOCOL_VERSION}",
                response.v
            )));
        }
        Ok(response)
    }

    /// Send a request and unwrap the success payload, turning an [`ApiError`]
    /// into a [`DaemonError::Protocol`].
    ///
    /// [`ApiError`]: jetsam_core::protocol::ApiError
    pub fn request(&mut self, method: Method) -> Result<OkData> {
        match self.call(method)?.payload {
            Payload::Ok(data) => Ok(data),
            Payload::Err(err) => Err(DaemonError::Protocol(format!(
                "{}: {}",
                err.code, err.message
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::ipc;
    use crate::store::Store;
    use crate::SharedState;
    use std::sync::Arc;

    fn connected() -> Client {
        let state = Arc::new(SharedState::new(
            Store::in_memory().unwrap(),
            Config::default(),
            std::path::PathBuf::from("/nonexistent/config.toml"),
            "0.0.0-test",
            0,
        ));
        let (client, server) = UnixStream::pair().unwrap();
        std::thread::spawn(move || {
            let _ = ipc::handle_stream(&state, server);
        });
        Client::from_stream(client).unwrap()
    }

    #[test]
    fn ping_round_trips() {
        let mut client = connected();
        assert_eq!(client.request(Method::Ping).unwrap(), OkData::Pong);
    }

    #[test]
    fn opt_in_then_status() {
        let mut client = connected();
        assert_eq!(
            client.request(Method::SetOptIn { enabled: true }).unwrap(),
            OkData::Ack
        );
        let OkData::Status(status) = client.request(Method::Status).unwrap() else {
            panic!("expected status");
        };
        assert!(status.opted_in);
    }

    #[test]
    fn ids_increment() {
        let mut client = connected();
        let first = client.call(Method::Ping).unwrap();
        let second = client.call(Method::Ping).unwrap();
        assert_eq!(second.id, first.id + 1);
    }
}
