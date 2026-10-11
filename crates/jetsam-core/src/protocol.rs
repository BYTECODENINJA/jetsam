//! The local wire protocol between Jetsam clients (GUI, CLI, modules) and the
//! usage daemon.
//!
//! Frames are newline-delimited JSON on a Unix domain socket. The protocol is
//! explicitly versioned; a client that sees an unexpected [`PROTOCOL_VERSION`]
//! should refuse to talk rather than guess. These types are also the basis of
//! every `--json` CLI output, so they are deliberately plain and stable.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Protocol version. Bump on any breaking change.
pub const PROTOCOL_VERSION: u32 = 1;

/// A client request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    /// Protocol version.
    pub v: u32,
    /// Client-chosen request id, echoed in the response.
    pub id: u64,
    /// The method and its parameters.
    pub method: Method,
}

impl Request {
    /// Build a request with the current protocol version.
    pub fn new(id: u64, method: Method) -> Self {
        Request {
            v: PROTOCOL_VERSION,
            id,
            method,
        }
    }
}

/// The methods a client may call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Method {
    /// Liveness check.
    Ping,
    /// Daemon and collector status.
    Status,
    /// Usage for a single package.
    PackageUsage {
        /// Package name.
        name: String,
    },
    /// The most-used packages.
    TopUsage {
        /// Maximum number of rows.
        limit: u32,
    },
    /// Usage for every observed package.
    UsageMap,
    /// The ignore list.
    IgnoreList,
    /// Add or remove a package from the ignore list.
    SetIgnored {
        /// Package name.
        name: String,
        /// Whether it should be ignored.
        ignored: bool,
    },
    /// Enable or disable usage collection.
    SetOptIn {
        /// The new opt-in state.
        enabled: bool,
    },
    /// Delete all collected usage data.
    Wipe,
}

/// A daemon response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    /// Protocol version.
    pub v: u32,
    /// Echoed request id (0 when the request could not be parsed).
    pub id: u64,
    /// The result or error.
    pub payload: Payload,
}

impl Response {
    /// A successful response.
    pub fn ok(id: u64, data: OkData) -> Self {
        Response {
            v: PROTOCOL_VERSION,
            id,
            payload: Payload::Ok(data),
        }
    }

    /// An error response.
    pub fn err(id: u64, code: impl Into<String>, message: impl Into<String>) -> Self {
        Response {
            v: PROTOCOL_VERSION,
            id,
            payload: Payload::Err(ApiError {
                code: code.into(),
                message: message.into(),
            }),
        }
    }
}

/// Either a successful payload or an error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Payload {
    /// Success.
    Ok(OkData),
    /// Failure.
    Err(ApiError),
}

/// Successful response data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "data", rename_all = "snake_case")]
pub enum OkData {
    /// Reply to [`Method::Ping`].
    Pong,
    /// Generic acknowledgement.
    Ack,
    /// Reply to [`Method::Status`].
    Status(DaemonStatus),
    /// Reply to [`Method::PackageUsage`].
    Usage(UsageInfo),
    /// Reply to [`Method::TopUsage`].
    UsageList(Vec<UsageInfo>),
    /// Reply to [`Method::UsageMap`].
    UsageMap(BTreeMap<String, UsageInfo>),
    /// Reply to [`Method::IgnoreList`].
    Names(Vec<String>),
}

/// A structured error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiError {
    /// Stable machine-readable code (e.g. `unknown_method`).
    pub code: String,
    /// Human-readable message.
    pub message: String,
}

/// Snapshot of daemon and collector state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonStatus {
    /// Daemon crate version.
    pub daemon_version: String,
    /// Protocol version the daemon speaks.
    pub protocol_version: u32,
    /// Whether usage collection is enabled.
    pub opted_in: bool,
    /// Fast sampling interval, in seconds.
    pub sampling_interval_secs: u64,
    /// Slow PSS sampling interval, in seconds.
    pub pss_interval_secs: u64,
    /// Number of distinct packages with recorded usage.
    pub packages_observed: u64,
    /// Number of packages on the ignore list.
    pub ignored_count: u64,
    /// Unix time of the first recorded sample.
    pub first_sample_epoch: Option<i64>,
    /// Unix time of the most recent recorded sample.
    pub last_sample_epoch: Option<i64>,
    /// Number of distinct days with recorded data.
    pub days_of_history: u64,
    /// Unix time the daemon started.
    pub started_at_epoch: i64,
    /// Seconds the daemon has been running.
    pub uptime_secs: u64,
}

/// Usage summary for one package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageInfo {
    /// Package name.
    pub name: String,
    /// Number of counted launches.
    pub launches: u64,
    /// Cumulative runtime in seconds.
    pub uptime_secs: u64,
    /// Unix time of the most recent use.
    pub last_used_epoch: Option<i64>,
    /// Unix time the package was first observed.
    pub first_seen_epoch: Option<i64>,
    /// Number of PSS samples averaged.
    pub pss_samples: u64,
    /// Average proportional set size, in bytes.
    pub avg_pss_bytes: Option<u64>,
}

impl UsageInfo {
    /// A zeroed record for a package with no usage yet.
    pub fn empty(name: impl Into<String>) -> Self {
        UsageInfo {
            name: name.into(),
            launches: 0,
            uptime_secs: 0,
            last_used_epoch: None,
            first_seen_epoch: None,
            pss_samples: 0,
            avg_pss_bytes: None,
        }
    }

    /// Days since last use relative to `now` (Unix seconds), if known.
    pub fn days_since_last_use(&self, now: i64) -> Option<f64> {
        self.last_used_epoch
            .map(|last| ((now - last).max(0) as f64) / 86_400.0)
    }
}
