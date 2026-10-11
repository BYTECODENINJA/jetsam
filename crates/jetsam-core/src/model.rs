//! Serializable domain types shared across the Jetsam workspace.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Why a package is installed, mirroring `alpm`'s reason flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallReason {
    /// Installed because the user asked for it.
    Explicit,
    /// Installed only to satisfy another package's dependency.
    Dependency,
}

impl InstallReason {
    /// Stable lowercase identifier used in the wire protocol and CLI `--json`.
    pub fn as_str(self) -> &'static str {
        match self {
            InstallReason::Explicit => "explicit",
            InstallReason::Dependency => "dependency",
        }
    }

    /// Whether the package was explicitly requested by the user.
    pub fn is_explicit(self) -> bool {
        matches!(self, InstallReason::Explicit)
    }
}

impl fmt::Display for InstallReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Where a package comes from.
///
/// The three official repositories are named because the UI and cleanup logic
/// care about them; anything else (including custom repos found in
/// `pacman.conf`, e.g. a vendor repo) is preserved verbatim in [`RepoKind::Other`].
/// Packages that are installed but present in no sync database are
/// [`RepoKind::Foreign`] — the AUR / local-build case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoKind {
    /// The `core` repository.
    Core,
    /// The `extra` repository.
    Extra,
    /// The `multilib` repository.
    Multilib,
    /// Installed but not found in any configured sync database.
    Foreign,
    /// Any other configured repository, by name.
    Other(String),
}

impl RepoKind {
    /// Build a [`RepoKind`] from a sync database name.
    pub fn from_name(name: &str) -> Self {
        match name {
            "core" => RepoKind::Core,
            "extra" => RepoKind::Extra,
            "multilib" => RepoKind::Multilib,
            "foreign" => RepoKind::Foreign,
            other => RepoKind::Other(other.to_string()),
        }
    }

    /// Stable lowercase identifier.
    pub fn as_str(&self) -> &str {
        match self {
            RepoKind::Core => "core",
            RepoKind::Extra => "extra",
            RepoKind::Multilib => "multilib",
            RepoKind::Foreign => "foreign",
            RepoKind::Other(name) => name.as_str(),
        }
    }

    /// Whether this package is foreign (AUR / local build).
    pub fn is_foreign(&self) -> bool {
        matches!(self, RepoKind::Foreign)
    }
}

impl fmt::Display for RepoKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How a package behaves, which decides how "usage" is even defined for it.
///
/// A package is not an application. Only [`PackageClass::DesktopApp`] and
/// [`PackageClass::CliTool`] have a meaningful launch count; a
/// [`PackageClass::Library`] is judged by its reverse dependencies and a
/// [`PackageClass::Daemon`] by runtime rather than launches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageClass {
    /// Ships a `.desktop` entry (or otherwise presents as a GUI application).
    DesktopApp,
    /// Ships an executable in a `bin`/`sbin` directory, heads-up or TUI.
    CliTool,
    /// Ships only shared objects / headers.
    Library,
    /// Ships a systemd unit (system or user).
    Daemon,
    /// Kernel, firmware, or graphics driver.
    KernelOrDriver,
    /// Fonts, icons, documentation, and other inert data.
    Data,
    /// Nothing in the file list matched a known pattern.
    Unknown,
}

impl PackageClass {
    /// Stable lowercase identifier.
    pub fn as_str(self) -> &'static str {
        match self {
            PackageClass::DesktopApp => "desktop_app",
            PackageClass::CliTool => "cli_tool",
            PackageClass::Library => "library",
            PackageClass::Daemon => "daemon",
            PackageClass::KernelOrDriver => "kernel_or_driver",
            PackageClass::Data => "data",
            PackageClass::Unknown => "unknown",
        }
    }

    /// Whether a launch count is a meaningful usage signal for this class.
    pub fn tracks_launches(self) -> bool {
        matches!(self, PackageClass::DesktopApp | PackageClass::CliTool)
    }
}

impl fmt::Display for PackageClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single row of the package inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageRecord {
    /// Package name.
    pub name: String,
    /// Installed version, as a string.
    pub version: String,
    /// Repository the package came from.
    pub repo: RepoKind,
    /// Explicit or pulled in as a dependency.
    pub reason: InstallReason,
    /// Install time as Unix epoch seconds, if known.
    pub install_date: Option<i64>,
    /// Installed size on disk in bytes (`alpm` `isize`).
    pub installed_size: i64,
    /// Compressed package size in bytes (`alpm` `size`).
    pub package_size: i64,
    /// Short description.
    pub description: Option<String>,
    /// Upstream URL.
    pub url: Option<String>,
    /// SPDX-ish license identifiers.
    pub licenses: Vec<String>,
    /// Package groups (e.g. `base`, `gnome`).
    pub groups: Vec<String>,
    /// Names this package provides (for dependency resolution).
    pub provides: Vec<String>,
    /// Direct dependency expressions (e.g. `glibc`, `libfoo>=1.2`).
    pub depends: Vec<String>,
    /// Behavioural classification.
    pub class: PackageClass,
    /// Whether the package is on the protected list.
    pub protected: bool,
    /// Why it is protected, when it is.
    pub protected_reason: Option<String>,
    /// Number of installed packages that depend on this one.
    pub reverse_dep_count: usize,
    /// Number of files owned by the package.
    pub file_count: usize,
}

/// A package plus the heavier detail only needed on the detail screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageDetail {
    /// The inventory row.
    pub record: PackageRecord,
    /// Owned files, as stored in the database (relative paths).
    pub files: Vec<String>,
    /// Names of installed packages that depend on this one.
    pub required_by: Vec<String>,
}
