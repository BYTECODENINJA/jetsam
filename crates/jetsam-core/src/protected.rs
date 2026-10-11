//! The protected set: packages that can never be suggested for removal.
//!
//! Being wrong here is much worse than being wrong about cleanup ranking, so
//! the rules are explicit and conservative. A package is protected if:
//!
//! * it belongs to the `base` group, or
//! * it is a known critical system package, or
//! * it is a kernel / firmware / graphics driver, or
//! * it is a bootloader or init-system component.
//!
//! The UI always shows protected packages and *why* they are excluded, so the
//! user is never left wondering whether Jetsam simply forgot about them.

use crate::classify;

/// Packages that are individually critical to a bootable, updatable system.
pub const CRITICAL: &[&str] = &[
    "pacman",
    "pacman-mirrorlist",
    "glibc",
    "filesystem",
    "base",
    "bash",
    "coreutils",
    "util-linux",
    "systemd",
    "systemd-libs",
    "systemd-sysvcompat",
    "dbus",
    "dbus-broker",
    "sudo",
    "polkit",
    "shadow",
    "pam",
    "kmod",
    "e2fsprogs",
    "gcc-libs",
    "zlib",
    "xz",
    "zstd",
    "openssl",
    "libarchive",
    "acl",
    "attr",
    "gpgme",
    "libgpg-error",
    "libgcrypt",
    "mkinitcpio",
    "dracut",
    "linux-api-headers",
];

/// Bootloaders. Removing one can render a machine unbootable.
pub const BOOTLOADERS: &[&str] = &[
    "grub",
    "grub-btrfs",
    "refind",
    "syslinux",
    "limine",
    "efibootmgr",
    "os-prober",
];

/// Determine whether a package is protected, returning a human-readable reason.
pub fn protected_reason(name: &str, groups: &[String]) -> Option<String> {
    if groups.iter().any(|g| g == "base") {
        return Some("member of the base group".to_string());
    }
    if CRITICAL.contains(&name) {
        return Some("critical system package".to_string());
    }
    if BOOTLOADERS.contains(&name) {
        return Some("bootloader".to_string());
    }
    if classify::KERNELS.contains(&name) {
        return Some("kernel".to_string());
    }
    if name.starts_with("linux-firmware") {
        return Some("firmware".to_string());
    }
    if name.starts_with("xf86-video-") {
        return Some("graphics driver".to_string());
    }
    if name == "nvidia" || name.starts_with("nvidia-") || name.starts_with("lib32-nvidia") {
        return Some("graphics driver".to_string());
    }
    None
}

/// Convenience predicate.
pub fn is_protected(name: &str, groups: &[String]) -> bool {
    protected_reason(name, groups).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_group_is_protected() {
        let groups = vec!["base".to_string()];
        assert!(is_protected("anything", &groups));
    }

    #[test]
    fn critical_packages_are_protected() {
        assert!(is_protected("pacman", &[]));
        assert!(is_protected("glibc", &[]));
        assert!(is_protected("systemd", &[]));
    }

    #[test]
    fn kernels_and_drivers_are_protected() {
        assert_eq!(protected_reason("linux", &[]).as_deref(), Some("kernel"));
        assert_eq!(
            protected_reason("linux-firmware", &[]).as_deref(),
            Some("firmware")
        );
        assert_eq!(
            protected_reason("nvidia", &[]).as_deref(),
            Some("graphics driver")
        );
    }

    #[test]
    fn bootloaders_are_protected() {
        assert_eq!(protected_reason("grub", &[]).as_deref(), Some("bootloader"));
    }

    #[test]
    fn ordinary_packages_are_not_protected() {
        assert!(!is_protected("vlc", &[]));
        assert!(!is_protected("ripgrep", &["some-group".to_string()]));
    }
}
