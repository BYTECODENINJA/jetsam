//! XDG-based filesystem locations.

use std::path::PathBuf;

fn env_path(var: &str, fallback: PathBuf) -> PathBuf {
    match std::env::var_os(var) {
        Some(value) if !value.is_empty() => PathBuf::from(value),
        _ => fallback,
    }
}

/// The user's home directory, or a temp fallback.
pub fn home() -> PathBuf {
    match std::env::var_os("HOME") {
        Some(value) if !value.is_empty() => PathBuf::from(value),
        _ => std::env::temp_dir(),
    }
}

/// `$XDG_CONFIG_HOME/jetsam`.
pub fn config_dir() -> PathBuf {
    env_path("XDG_CONFIG_HOME", home().join(".config")).join("jetsam")
}

/// `$XDG_DATA_HOME/jetsam`.
pub fn data_dir() -> PathBuf {
    env_path("XDG_DATA_HOME", home().join(".local/share")).join("jetsam")
}

/// `$XDG_STATE_HOME/jetsam`.
pub fn state_dir() -> PathBuf {
    env_path("XDG_STATE_HOME", home().join(".local/state")).join("jetsam")
}

/// `$XDG_RUNTIME_DIR/jetsam` (falls back to the temp directory).
pub fn runtime_dir() -> PathBuf {
    match std::env::var_os("XDG_RUNTIME_DIR") {
        Some(value) if !value.is_empty() => PathBuf::from(value),
        _ => std::env::temp_dir(),
    }
    .join("jetsam")
}

/// Path to the daemon socket.
pub fn socket_path() -> PathBuf {
    runtime_dir().join("daemon.sock")
}

/// Path to the usage database.
pub fn db_path() -> PathBuf {
    data_dir().join("usage.db")
}

/// Path to the configuration file.
pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}
