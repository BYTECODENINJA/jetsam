//! Jetsam usage daemon.
//!
//! The daemon is an unprivileged **systemd user service**. It samples `/proc`,
//! maps each process executable to its owning package with the ownership index
//! from `jetsam-core`, and stores usage history in SQLite. It exposes a small
//! newline-delimited JSON protocol over a Unix socket for the GUI and CLI.
//!
//! It never runs as root, never calls `pacman`, and opens no network sockets.

#![forbid(unsafe_code)]

pub mod client;
pub mod collector;
pub mod config;
pub mod error;
pub mod ipc;
pub mod logging;
pub mod notify;
pub mod paths;
pub mod procfs;
pub mod sampler;
pub mod store;

use std::path::PathBuf;
use std::sync::Mutex;

pub use error::{DaemonError, Result};

/// State shared between the collector loop and IPC connections.
pub struct SharedState {
    /// The usage database.
    pub store: Mutex<store::Store>,
    /// The current configuration (reloaded live by the client).
    pub config: Mutex<config::Config>,
    /// Where the configuration file lives.
    pub config_path: PathBuf,
    /// Daemon crate version, reported to clients.
    pub daemon_version: &'static str,
    /// Unix time the daemon started.
    pub started_at: i64,
}

impl SharedState {
    /// Build the shared state.
    pub fn new(
        store: store::Store,
        config: config::Config,
        config_path: PathBuf,
        daemon_version: &'static str,
        started_at: i64,
    ) -> Self {
        SharedState {
            store: Mutex::new(store),
            config: Mutex::new(config),
            config_path,
            daemon_version,
            started_at,
        }
    }

    /// Lock the usage store, mapping a poisoned lock to a startup error.
    pub fn lock_store(&self) -> Result<std::sync::MutexGuard<'_, store::Store>> {
        self.store
            .lock()
            .map_err(|_| DaemonError::Startup("store lock poisoned".to_string()))
    }

    /// Lock the configuration, mapping a poisoned lock to a startup error.
    pub fn lock_config(&self) -> Result<std::sync::MutexGuard<'_, config::Config>> {
        self.config
            .lock()
            .map_err(|_| DaemonError::Startup("config lock poisoned".to_string()))
    }
}

/// Current Unix time in seconds.
pub fn now_epoch() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
