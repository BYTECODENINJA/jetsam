//! Daemon error type.

use thiserror::Error;

/// Errors produced by the daemon.
#[derive(Debug, Error)]
pub enum DaemonError {
    /// Filesystem or socket I/O failed.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    /// SQLite failed.
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// Configuration (de)serialisation failed.
    #[error("config error: {0}")]
    Config(String),

    /// Protocol (de)serialisation failed.
    #[error("protocol error: {0}")]
    Protocol(String),

    /// An error bubbled up from `jetsam-core`.
    #[error("core error: {0}")]
    Core(#[from] jetsam_core::CoreError),

    /// The daemon could not be started.
    #[error("startup error: {0}")]
    Startup(String),
}

impl From<serde_json::Error> for DaemonError {
    fn from(err: serde_json::Error) -> Self {
        DaemonError::Protocol(err.to_string())
    }
}

impl From<toml::de::Error> for DaemonError {
    fn from(err: toml::de::Error) -> Self {
        DaemonError::Config(err.to_string())
    }
}

impl From<toml::ser::Error> for DaemonError {
    fn from(err: toml::ser::Error) -> Self {
        DaemonError::Config(err.to_string())
    }
}

/// Convenience result alias.
pub type Result<T> = std::result::Result<T, DaemonError>;
