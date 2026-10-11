//! Package classification heuristics.
//!
//! Linux does not record what an application is, so we infer it from the file
//! list shipped by the package. The inference is deliberately conservative and
//! entirely explainable: the UI shows the class as a hint, never as fact.
//!
//! Note on file modes: `alpm`'s file list only reliably populates `mode` when
//! mtree data is present, so we classify by *path* rather than by permission
//! bits. Executables live in well-known `bin`/`sbin` directories.

use crate::model::PackageClass;

/// Directories whose non-directory children are treated as executables.
pub const BIN_DIRS: &[&str] = &[
    "bin/",
    "sbin/",
    "usr/bin/",
    "usr/sbin/",
    "usr/local/bin/",
    "usr/local/sbin/",
];

/// Directories whose children define a systemd unit (i.e. a daemon/service).
pub const SYSTEMD_DIRS: &[&str] = &[
    "usr/lib/systemd/system/",
    "usr/lib/systemd/user/",
    "usr/lib/systemd/system-generators/",
    "usr/lib/systemd/user-generators/",
];

/// Kernels recognised by exact package name.
pub const KERNELS: &[&str] = &[
    "linux",
    "linux-lts",
    "linux-zen",
    "linux-hardened",
    "linux-rt",
    "linux-rt-lts",
    "linux-mainline",
    "linux-lqx",
    "linux-xanmod",
    "linux-cachyos",
    "linux-cachyos-lts",
    "linux-cachyos-bore",
    "linux-cachyos-eevdf",
    "linux-cachyos-rt",
    "linux-cachyos-server",
    "linux-cachyos-hardened",
];

/// Strip a single leading `/` so path rules work on both the relative paths
/// stored in the pacman database and the absolute paths used at runtime.
fn rel(path: &str) -> &str {
    path.strip_prefix('/').unwrap_or(path)
}

/// Whether a database file entry denotes a directory.
pub fn is_dir(path: &str) -> bool {
    path.ends_with('/')
}

/// Whether a path is an executable shipped in a `bin`/`sbin` directory.
pub fn is_executable_path(path: &str) -> bool {
    if is_dir(path) {
        return false;
    }
    let p = rel(path);
    BIN_DIRS.iter().any(|d| p.starts_with(d))
}

/// Whether a path is a shared object / library.
pub fn is_shared_lib(path: &str) -> bool {
    if is_dir(path) || !path.contains(".so") {
        return false;
    }
    let p = rel(path);
    p.starts_with("usr/lib/")
        || p.starts_with("usr/lib64/")
        || p.starts_with("usr/lib32/")
        || p.starts_with("lib/")
        || p.starts_with("lib64/")
}

/// Whether a path is a systemd unit shipped by the package.
pub fn is_service_path(path: &str) -> bool {
    if is_dir(path) {
        return false;
    }
    let p = rel(path);
    SYSTEMD_DIRS.iter().any(|d| p.starts_with(d))
}

/// Whether a path is a desktop entry.
pub fn is_desktop_entry(path: &str) -> bool {
    if is_dir(path) {
        return false;
    }
    let p = rel(path);
    p.starts_with("usr/share/applications/") && p.ends_with(".desktop")
}

/// Whether a package name denotes a kernel, firmware, or graphics driver.
pub fn is_kernel_or_driver(name: &str) -> bool {
    if KERNELS.contains(&name) {
        return true;
    }
    if name.starts_with("linux-firmware") {
        return true;
    }
    if name.starts_with("xf86-video-") {
        return true;
    }
    if name == "nvidia" || name.starts_with("nvidia-") || name.starts_with("lib32-nvidia") {
        return true;
    }
    false
}

/// Classify a package from its file paths.
///
/// `files` yields the package's file paths exactly as stored (relative, with
/// directories ending in `/`). Ordering of the returned class is:
/// kernel/driver, desktop app, daemon, CLI tool, library, data, unknown.
pub fn classify_paths<'a, I>(name: &str, files: I) -> PackageClass
where
    I: IntoIterator<Item = &'a str>,
{
    if is_kernel_or_driver(name) {
        return PackageClass::KernelOrDriver;
    }

    let mut any = false;
    let mut desktop = false;
    let mut service = false;
    let mut executable = false;
    let mut library = false;

    for path in files {
        any = true;
        if is_dir(path) {
            continue;
        }
        if is_desktop_entry(path) {
            desktop = true;
        } else if is_service_path(path) {
            service = true;
        } else if is_executable_path(path) {
            executable = true;
        } else if is_shared_lib(path) {
            library = true;
        }
    }

    if !any {
        return PackageClass::Unknown;
    }
    if desktop {
        PackageClass::DesktopApp
    } else if service {
        PackageClass::Daemon
    } else if executable {
        PackageClass::CliTool
    } else if library {
        PackageClass::Library
    } else {
        PackageClass::Data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classify(name: &str, files: &[&str]) -> PackageClass {
        classify_paths(name, files.iter().copied())
    }

    #[test]
    fn dirs_and_relative_paths_are_recognised() {
        assert!(is_dir("usr/bin/"));
        assert!(!is_dir("usr/bin/foo"));
        assert!(is_executable_path("usr/bin/foo"));
        assert!(is_executable_path("/usr/bin/foo"));
        assert!(!is_executable_path("usr/bin/"));
        assert!(is_shared_lib("usr/lib/libfoo.so.1"));
        assert!(is_shared_lib("/usr/lib/libfoo.so"));
        assert!(!is_shared_lib("usr/share/doc/foo.so.txt"));
        assert!(is_service_path("usr/lib/systemd/system/foo.service"));
        assert!(is_desktop_entry("usr/share/applications/foo.desktop"));
    }

    #[test]
    fn gui_app_is_desktop_app_even_with_a_binary() {
        let c = classify(
            "vlc",
            &["usr/bin/vlc", "usr/share/applications/vlc.desktop"],
        );
        assert_eq!(c, PackageClass::DesktopApp);
    }

    #[test]
    fn service_beats_binary_but_not_desktop() {
        assert_eq!(
            classify(
                "docker",
                &["usr/bin/dockerd", "usr/lib/systemd/system/docker.service"]
            ),
            PackageClass::Daemon
        );
        assert_eq!(
            classify(
                "weird",
                &[
                    "usr/bin/weird",
                    "usr/lib/systemd/system/weird.service",
                    "usr/share/applications/weird.desktop",
                ]
            ),
            PackageClass::DesktopApp
        );
    }

    #[test]
    fn binary_only_is_cli_tool() {
        assert_eq!(
            classify("ripgrep", &["usr/bin/rg", "usr/share/man/man1/rg.1"]),
            PackageClass::CliTool
        );
    }

    #[test]
    fn objects_only_is_library() {
        assert_eq!(
            classify(
                "zlib",
                &["usr/lib/libz.so", "usr/lib/libz.so.1", "usr/include/zlib.h"]
            ),
            PackageClass::Library
        );
    }

    #[test]
    fn inert_content_is_data() {
        assert_eq!(
            classify("ttf-fira-code", &["usr/share/fonts/TTF/FiraCode.ttf"]),
            PackageClass::Data
        );
    }

    #[test]
    fn no_files_is_unknown() {
        assert_eq!(classify("mystery", &[]), PackageClass::Unknown);
    }

    #[test]
    fn kernels_and_drivers_win() {
        assert_eq!(
            classify("linux", &["boot/vmlinuz-linux"]),
            PackageClass::KernelOrDriver
        );
        assert_eq!(
            classify("linux-firmware", &["usr/lib/firmware/iwlwifi.bin"]),
            PackageClass::KernelOrDriver
        );
        assert_eq!(
            classify("nvidia-utils", &["usr/lib/libnvidia.so"]),
            PackageClass::KernelOrDriver
        );
        assert_eq!(
            classify(
                "xf86-video-amdgpu",
                &["usr/lib/xorg/modules/drivers/amdgpu.so"]
            ),
            PackageClass::KernelOrDriver
        );
    }
}
