//! Invocation of the privileged helper through `pkexec`.
//!
//! The GUI never calls pacman and never runs as root. It asks pkexec to run
//! `jetsam-helper`, which re-validates everything. Argument vectors are passed
//! directly to the process; no shell is involved.

use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};

/// Mirrors `jetsam_helper::HelperResult` (kept local so the GUI does not link
/// the helper crate).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperOutcome {
    /// `remove` or `upgrade`.
    pub action: String,
    /// Whether pacman reported success.
    pub ok: bool,
    /// pacman's exit code, when it ran.
    pub exit_code: Option<i32>,
    /// Raw stdout.
    pub stdout: String,
    /// Raw stderr.
    pub stderr: String,
}

/// Whether a `jetsam-helper` binary can be located.
pub fn available() -> bool {
    find_helper().is_some()
}

/// The path to the helper, if found.
pub fn helper_path() -> Option<PathBuf> {
    find_helper()
}

/// Run `jetsam-helper remove <names>` via pkexec.
pub fn remove(names: &[String]) -> Result<HelperOutcome, String> {
    let helper = find_helper().ok_or("jetsam-helper was not found")?;
    let mut command = Command::new("pkexec");
    command.arg(helper).arg("remove").args(names);
    run(command)
}

/// Run `jetsam-helper upgrade` via pkexec.
pub fn upgrade() -> Result<HelperOutcome, String> {
    let helper = find_helper().ok_or("jetsam-helper was not found")?;
    let mut command = Command::new("pkexec");
    command.arg(helper).arg("upgrade");
    run(command)
}

fn run(mut command: Command) -> Result<HelperOutcome, String> {
    let output = command.output().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // The helper prints exactly one JSON line on stdout when it executed pacman.
    if let Some(line) = stdout.lines().find(|l| l.trim_start().starts_with('{')) {
        if let Ok(outcome) = serde_json::from_str::<HelperOutcome>(line) {
            return Ok(outcome);
        }
    }

    // Otherwise the helper refused before running (bad names, protected, not
    // root through a faulty policy, etc.).
    Ok(HelperOutcome {
        action: "unknown".to_string(),
        ok: false,
        exit_code: output.status.code(),
        stdout: stdout.into_owned(),
        stderr: stderr.into_owned(),
    })
}

fn find_helper() -> Option<PathBuf> {
    if let Ok(explicit) = std::env::var("JETSAM_HELPER") {
        let path = PathBuf::from(explicit);
        if path.is_file() {
            return Some(path);
        }
    }
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(PathBuf::from))
    {
        let candidate = dir.join("jetsam-helper");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join("jetsam-helper");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}
