//! A minimal stderr logger.
//!
//! Deliberately dependency-free: the daemon's logging needs are tiny and we do
//! not want a logging framework in a process that runs continuously. Level is
//! controlled by `JETSAM_LOG` (`error`, `warn`, `info`, `debug`; default `info`).

use std::sync::atomic::{AtomicU8, Ordering};

const ERROR: u8 = 0;
const WARN: u8 = 1;
const INFO: u8 = 2;
const DEBUG: u8 = 3;

static LEVEL: AtomicU8 = AtomicU8::new(INFO);

/// Initialise the log level from the environment.
pub fn init() {
    let level = match std::env::var("JETSAM_LOG").as_deref() {
        Ok("error") => ERROR,
        Ok("warn") => WARN,
        Ok("debug") => DEBUG,
        _ => INFO,
    };
    LEVEL.store(level, Ordering::Relaxed);
}

fn emit(level: u8, label: &str, message: &str) {
    if level <= LEVEL.load(Ordering::Relaxed) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        eprintln!("[jetsamd {now} {label}] {message}");
    }
}

/// Log at error level.
pub fn error(message: impl AsRef<str>) {
    emit(ERROR, "ERROR", message.as_ref());
}

/// Log at warn level.
pub fn warn(message: impl AsRef<str>) {
    emit(WARN, "WARN", message.as_ref());
}

/// Log at info level.
pub fn info(message: impl AsRef<str>) {
    emit(INFO, "INFO", message.as_ref());
}

/// Log at debug level.
pub fn debug(message: impl AsRef<str>) {
    emit(DEBUG, "DEBUG", message.as_ref());
}
