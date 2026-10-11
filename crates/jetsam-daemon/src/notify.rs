//! Desktop notifications via `notify-send` (best-effort, never fatal).
//!
//! The daemon never blocks on a notification and never fails because one could
//! not be delivered. Arguments are passed directly to the process — no shell is
//! involved.

/// Whether notifications can plausibly be delivered in this session.
pub fn available() -> bool {
    let has_session =
        std::env::var_os("WAYLAND_DISPLAY").is_some() || std::env::var_os("DISPLAY").is_some();
    has_session && which_notify_send().is_some()
}

fn which_notify_send() -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join("notify-send"))
        .find(|candidate| candidate.is_file())
}

/// Send a desktop notification, ignoring any failure.
pub fn send(summary: &str, body: &str) {
    if !available() {
        return;
    }
    let _ = std::process::Command::new("notify-send")
        .arg("--app-name=Jetsam")
        .arg(summary)
        .arg(body)
        .status();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_reflects_environment() {
        // Availability depends on the test environment; the call must simply
        // not panic either way.
        let _ = available();
    }
}
