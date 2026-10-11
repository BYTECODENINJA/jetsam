//! The privileged helper.
//!
//! This binary is the only component of Jetsam that ever runs with elevated
//! privileges. It is invoked through `pkexec`, verifies that it actually runs
//! as root, strictly validates every argument, refuses anything on the protected
//! list, and then executes `pacman` through `Command` (never through a shell),
//! reporting the raw result back as JSON.
//!
//! Threat model: even a fully compromised GUI cannot make the helper run
//! arbitrary commands or remove critical packages. The GUI's worst-case power is
//! "remove packages that were valid, non-protected package names".

#![forbid(unsafe_code)]

use std::process::Command;

use jetsam_core::protected::is_protected;
use serde::{Deserialize, Serialize};

/// The operations the helper knows how to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    /// Remove the given packages (`--noconfirm -R ...`). Undependended, so
    /// packages that other installed packages need are refused by pacman itself.
    Remove,
    /// Full system upgrade (`--noconfirm -Syu`). Partial upgrades are never performed.
    Upgrade,
}

impl Action {
    /// The stable string used in [`HelperResult`].
    pub fn as_str(&self) -> &'static str {
        match self {
            Action::Remove => "remove",
            Action::Upgrade => "upgrade",
        }
    }
}

/// The machine-readable result sent to the caller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelperResult {
    /// Which action was attempted.
    pub action: String,
    /// Whether the operation succeeded.
    pub ok: bool,
    /// `pacman`'s exit code (None when it could not be started).
    pub exit_code: Option<i32>,
    /// `pacman`'s stdout.
    pub stdout: String,
    /// `pacman`'s stderr.
    pub stderr: String,
}

/// Validation failures (printed to stderr, not returned).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelperError {
    InvalidPackage(String),
    ProtectedPackage(String),
    TooManyPackages(usize),
    NoPackages,
    NotRoot,
}

impl HelperError {
    /// Exit code conveyed to the process.
    pub fn exit_code(&self) -> i32 {
        match self {
            HelperError::NotRoot => 2,
            _ => 3,
        }
    }

    /// A stderr message.
    pub fn message(&self) -> String {
        match self {
            HelperError::InvalidPackage(name) => format!("refusing invalid package name: {name}"),
            HelperError::ProtectedPackage(name) => {
                format!("refusing protected package: {name}")
            }
            HelperError::TooManyPackages(n) => format!("too many packages: {n}"),
            HelperError::NoPackages => "no packages to act on".to_string(),
            HelperError::NotRoot => "jetsam-helper must run as root (via pkexec)".to_string(),
        }
    }
}

/// The maximum number of packages a single remove request may carry.
pub const MAX_PACKAGES: usize = 256;

/// The maximum length of a single package identifier.
pub const MAX_NAME_LEN: usize = 255;

/// A valid Arch package identifier has a leading alphanumeric and contains
/// only `[A-Za-z0-9@._+-]`. The leading-character rule guarantees the name can
/// never be parsed as a pacman option.
pub fn validate_package_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_NAME_LEN {
        return false;
    }
    if !bytes[0].is_ascii_alphanumeric() {
        return false;
    }
    bytes
        .iter()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'@' | b'.' | b'_' | b'+' | b'-'))
}

/// Validate and deduplicate a removal list. This is the helper's security
/// gate: nothing reaches `pacman` without passing here, and anything on the
/// protected list is refused no matter what the GUI says.
pub fn validate_remove_names(names: &[String]) -> Result<Vec<String>, HelperError> {
    if names.is_empty() {
        return Err(HelperError::NoPackages);
    }
    if names.len() > MAX_PACKAGES {
        return Err(HelperError::TooManyPackages(names.len()));
    }
    let mut seen = Vec::new();
    for name in names {
        if !validate_package_name(name) {
            return Err(HelperError::InvalidPackage(name.clone()));
        }
        if is_protected(name, &[]) {
            return Err(HelperError::ProtectedPackage(name.clone()));
        }
        if !seen.contains(name) {
            seen.push(name.clone());
        }
    }
    Ok(seen)
}

/// Build the argv for an action, given already-validated names. No shell is
/// involved; every element is a separate [`Command`] argument.
pub fn pacman_command(action: &Action, names: &[String]) -> Vec<String> {
    match action {
        Action::Remove => {
            let mut argv = vec![
                "/usr/bin/pacman".to_string(),
                "--noconfirm".to_string(),
                "-R".to_string(),
            ];
            argv.extend(names.iter().cloned());
            argv
        }
        Action::Upgrade => vec![
            "/usr/bin/pacman".to_string(),
            "--noconfirm".to_string(),
            "-Syu".to_string(),
        ],
    }
}

/// Run the action against pacman and return the raw result.
pub fn run(action: &Action, names: &[String]) -> HelperResult {
    let argv = pacman_command(action, names);
    let output = Command::new(&argv[0]).args(&argv[1..]).output();
    match output {
        Ok(output) => HelperResult {
            action: action.as_str().to_string(),
            ok: output.status.success(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        },
        Err(err) => HelperResult {
            action: action.as_str().to_string(),
            ok: false,
            exit_code: None,
            stdout: String::new(),
            stderr: format!("could not execute pacman: {err}"),
        },
    }
}

/// Parse the first `Uid:` field from `/proc/<pid>/status`-style text.
pub fn real_uid_zero(status: &str) -> bool {
    status
        .lines()
        .find_map(|line| line.strip_prefix("Uid:"))
        .and_then(|rest| rest.split_whitespace().next())
        .map(|uid| uid == "0")
        .unwrap_or(false)
}

/// Whether the current process runs as root (real uid 0). The helper refuses to
/// act unless this is true, so `pkexec`'s elevation is verified rather than
/// assumed.
pub fn current_is_root() -> bool {
    std::fs::read_to_string("/proc/self/status")
        .map(|text| real_uid_zero(&text))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_package_names_pass() {
        for name in [
            "firefox",
            "zlib",
            "lib32-gcc-libs",
            "x86_64-pc-linux-gnu-foo",
            "gcc-libs",
            "python-3.12.7-1",
            "0ad",
        ] {
            assert!(validate_package_name(name), "{name}");
        }
    }

    #[test]
    fn names_that_could_be_options_are_rejected() {
        for name in ["-R", "-Syu", "--help", "-", "--noconfirm", "-y", "-U"] {
            assert!(!validate_package_name(name), "{name}");
        }
    }

    #[test]
    fn junk_is_rejected() {
        for name in [
            "",
            "a b",
            "a;rm",
            "a$(x)",
            "sys\u{0}tem",
            "!bang",
            "back\\slash",
        ] {
            assert!(!validate_package_name(name), "{name:?}");
        }
    }

    #[test]
    fn overlong_names_are_rejected() {
        let name = "a".repeat(MAX_NAME_LEN + 1);
        assert!(!validate_package_name(&name));
        assert!(validate_package_name(&"a".repeat(MAX_NAME_LEN)));
    }

    #[test]
    fn removal_list_is_deduped_and_validated() {
        let names: Vec<String> = vec!["a".into(), "b".into(), "a".into()];
        assert_eq!(validate_remove_names(&names).unwrap(), vec!["a", "b"]);
    }

    #[test]
    fn protected_packages_are_always_refused() {
        for name in ["pacman", "glibc", "systemd", "linux"] {
            let err = validate_remove_names(&[name.to_string()]).unwrap_err();
            assert_eq!(err, HelperError::ProtectedPackage(name.to_string()));
        }
    }

    #[test]
    fn empty_and_oversized_lists_are_refused() {
        assert_eq!(
            validate_remove_names(&[]).unwrap_err(),
            HelperError::NoPackages
        );
        let many: Vec<String> = (0..(MAX_PACKAGES + 1))
            .map(|i| format!("pkg-{i}"))
            .collect();
        assert!(matches!(
            validate_remove_names(&many).unwrap_err(),
            HelperError::TooManyPackages(_)
        ));
    }

    #[test]
    fn command_building_never_uses_shell_strings() {
        let mut names: Vec<String> = (0..3).map(|i| format!("pkg-{i}")).collect();
        names.push("-f".to_string()); // would be caught by validation, but argv must still be sane
        let argv = pacman_command(&Action::Remove, &names);
        assert!(argv.contains(&"-R".to_string()));
        assert!(argv.contains(&"--noconfirm".to_string()));
        // Each name is a discrete argv element, never inline text.
        for n in &names {
            assert!(argv.contains(n));
        }
        let up = pacman_command(&Action::Upgrade, &[]);
        assert!(up.contains(&"-Syu".to_string()));
        assert_eq!(up.len(), 3, "upgrade is exactly pacman --noconfirm -Syu");
    }

    #[test]
    fn uid_zero_detection() {
        assert!(real_uid_zero("Uid:\t0\t0\t0\t0\n"));
        assert!(real_uid_zero("Uid:\t0   \n"));
        assert!(!real_uid_zero("Uid:\t1000\t1000\t1000\t1000\n"));
        assert!(!real_uid_zero("Gid:\t0\n"));
        assert!(!real_uid_zero(""));
    }
}
