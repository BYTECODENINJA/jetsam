//! Thin client helpers around the Jetsam usage daemon, plus daemon start.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use jetsam_core::protocol::{DaemonStatus, Method, OkData, UsageInfo};
use jetsam_daemon::client::Client;

/// Whether the daemon answers a ping.
pub fn is_up() -> bool {
    match Client::connect_default() {
        Ok(mut client) => client.request(Method::Ping).is_ok(),
        Err(_) => false,
    }
}

/// Fetch daemon status, if reachable.
pub fn status() -> Option<DaemonStatus> {
    let mut client = Client::connect_default().ok()?;
    match client.request(Method::Status).ok()? {
        OkData::Status(status) => Some(status),
        _ => None,
    }
}

/// Fetch the full usage map, if reachable.
pub fn usage_map() -> Option<BTreeMap<String, UsageInfo>> {
    let mut client = Client::connect_default().ok()?;
    match client.request(Method::UsageMap).ok()? {
        OkData::UsageMap(map) => Some(map),
        _ => None,
    }
}

/// Fetch the ignore list, if reachable.
pub fn ignore_list() -> Option<Vec<String>> {
    let mut client = Client::connect_default().ok()?;
    match client.request(Method::IgnoreList).ok()? {
        OkData::Names(names) => Some(names),
        _ => None,
    }
}

/// Set a package's ignored state.
pub fn set_ignored(name: &str, ignored: bool) -> Result<(), String> {
    let mut client = Client::connect_default().map_err(|e| e.to_string())?;
    client
        .request(Method::SetIgnored {
            name: name.to_string(),
            ignored,
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Enable or disable usage collection.
pub fn set_opt_in(enabled: bool) -> Result<(), String> {
    let mut client = Client::connect_default().map_err(|e| e.to_string())?;
    client
        .request(Method::SetOptIn { enabled })
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Delete all collected usage data.
pub fn wipe() -> Result<(), String> {
    let mut client = Client::connect_default().map_err(|e| e.to_string())?;
    client.request(Method::Wipe).map_err(|e| e.to_string())?;
    Ok(())
}

/// Try to start the daemon: first via systemd, then by spawning a sibling
/// `jetsamd` binary. Returns whether the daemon answered a ping afterwards.
pub fn start() -> Result<bool, String> {
    if is_up() {
        return Ok(true);
    }

    // Preferred: the packaged systemd user service.
    let _ = Command::new("systemctl")
        .args(["--user", "start", "jetsam-daemon.service"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    for _ in 0..20 {
        if is_up() {
            return Ok(true);
        }
        std::thread::sleep(std::time::Duration::from_millis(150));
    }

    // Fallback: spawn a jetsamd found next to us or on PATH.
    if let Some(bin) = find_jetsamd() {
        let spawned = Command::new(bin)
            .arg("run")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        if spawned.is_ok() {
            for _ in 0..20 {
                if is_up() {
                    return Ok(true);
                }
                std::thread::sleep(std::time::Duration::from_millis(150));
            }
        }
    }

    Ok(is_up())
}

fn find_jetsamd() -> Option<PathBuf> {
    if let Ok(explicit) = std::env::var("JETSAMD") {
        let path = PathBuf::from(explicit);
        if path.is_file() {
            return Some(path);
        }
    }
    if let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(PathBuf::from))
    {
        let candidate = exe_dir.join("jetsamd");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join("jetsamd");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}
