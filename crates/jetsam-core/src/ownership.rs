//! The file-ownership index used to map a running process back to a package.
//!
//! libalpm exposes the file list of every installed package. Reversing that into
//! `path → package` lets the collector attribute `\/proc/<pid>/exe` to a package
//! without shelling out to `pacman -Qo`.
//!
//! # Memory
//!
//! A full system owns on the order of 300k–400k files (measured: ~380k on the
//! development machine), which is far too much for the daemon's sub-15 MiB
//! budget. The default [`IndexScope::Executables`] therefore indexes only
//! executable files, desktop entries and shared objects — a few thousand
//! entries — which is everything the process mapper actually needs. Directory
//! entries are always indexed so a runtime-created file inside a package-owned
//! directory can still be attributed (best-effort).

use std::collections::HashMap;

use crate::classify;

/// Which files an index stores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IndexScope {
    /// Index every file the package owns.
    All,
    /// Index only executables, desktop entries and shared objects (default).
    #[default]
    Executables,
}

/// Normalise a database or runtime path to an absolute, single-slash form.
///
/// Directory paths keep their trailing slash; file paths do not gain one.
pub fn normalize_path(path: &str) -> String {
    let trimmed = path.trim();
    let mut out = String::with_capacity(trimmed.len() + 1);
    if !trimmed.starts_with('/') {
        out.push('/');
    }
    out.push_str(trimmed);
    if out.contains("//") {
        while out.contains("//") {
            out = out.replace("//", "/");
        }
    }
    out
}

/// A reverse index from owned path to owning package name.
#[derive(Debug, Clone, Default)]
pub struct OwnershipIndex {
    scope: IndexScope,
    files: HashMap<String, String>,
    dirs: HashMap<String, String>,
    package_count: usize,
}

impl OwnershipIndex {
    /// Create an empty index with the given scope.
    pub fn new(scope: IndexScope) -> Self {
        OwnershipIndex {
            scope,
            ..OwnershipIndex::default()
        }
    }

    /// The scope this index was built with.
    pub fn scope(&self) -> IndexScope {
        self.scope
    }

    /// Record that `pkg` owns `path`, honouring the index scope.
    pub fn insert(&mut self, path: &str, pkg: &str) {
        let normalized = normalize_path(path);
        if normalized.ends_with('/') {
            // Directories are always indexed; first writer wins on collision
            // (directory ownership is inherently ambiguous).
            self.dirs
                .entry(normalized)
                .or_insert_with(|| pkg.to_string());
            return;
        }
        let keep = match self.scope {
            IndexScope::All => true,
            IndexScope::Executables => {
                classify::is_executable_path(&normalized)
                    || classify::is_desktop_entry(&normalized)
                    || classify::is_shared_lib(&normalized)
            }
        };
        if keep {
            self.files
                .entry(normalized)
                .or_insert_with(|| pkg.to_string());
        }
    }

    /// Record how many packages fed the index.
    pub fn set_package_count(&mut self, count: usize) {
        self.package_count = count;
    }

    /// Number of packages that contributed to the index.
    pub fn package_count(&self) -> usize {
        self.package_count
    }

    /// Number of indexed files.
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Number of indexed directories.
    pub fn dir_count(&self) -> usize {
        self.dirs.len()
    }

    /// Total number of indexed entries.
    pub fn len(&self) -> usize {
        self.files.len() + self.dirs.len()
    }

    /// Whether the index is empty.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty() && self.dirs.is_empty()
    }

    /// Find the owning package for a path.
    ///
    /// Exact file matches win. Otherwise the nearest ancestor directory that is
    /// owned by a package is returned, which attributes runtime-created files.
    pub fn owner_of(&self, path: &str) -> Option<&str> {
        let normalized = normalize_path(path);
        if let Some(pkg) = self.files.get(&normalized) {
            return Some(pkg);
        }
        let mut cursor = normalized.as_str();
        loop {
            match cursor.rfind('/') {
                Some(0) => return self.dirs.get("/").map(String::as_str),
                Some(index) => {
                    let dir = &cursor[..=index];
                    if let Some(pkg) = self.dirs.get(dir) {
                        return Some(pkg.as_str());
                    }
                    cursor = &cursor[..index];
                }
                None => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_adds_leading_slash_and_collapses() {
        assert_eq!(normalize_path("usr/bin/foo"), "/usr/bin/foo");
        assert_eq!(normalize_path("/usr//bin/foo"), "/usr/bin/foo");
        assert_eq!(normalize_path("usr/lib/"), "/usr/lib/");
        assert_eq!(normalize_path(""), "/");
    }

    #[test]
    fn exact_file_match_wins() {
        let mut idx = OwnershipIndex::new(IndexScope::All);
        idx.insert("usr/bin/bash", "bash");
        assert_eq!(idx.owner_of("usr/bin/bash"), Some("bash"));
        assert_eq!(idx.owner_of("/usr/bin/bash"), Some("bash"));
    }

    #[test]
    fn executable_scope_skips_plain_data_files() {
        let mut idx = OwnershipIndex::new(IndexScope::Executables);
        idx.insert("usr/bin/rg", "ripgrep");
        idx.insert("usr/share/man/man1/rg.1", "ripgrep");
        idx.insert("usr/lib/libz.so", "zlib");
        idx.insert("usr/share/applications/vlc.desktop", "vlc");
        assert_eq!(idx.owner_of("usr/bin/rg"), Some("ripgrep"));
        assert_eq!(idx.owner_of("usr/lib/libz.so"), Some("zlib"));
        assert_eq!(
            idx.owner_of("usr/share/applications/vlc.desktop"),
            Some("vlc")
        );
        assert_eq!(idx.owner_of("usr/share/man/man1/rg.1"), None);
    }

    #[test]
    fn directory_prefix_attributes_runtime_files() {
        let mut idx = OwnershipIndex::new(IndexScope::Executables);
        idx.insert("usr/lib/jvm/", "jre-openjdk");
        idx.insert("usr/lib/jre/", "jre-openjdk");
        // A file created at runtime inside an owned directory.
        assert_eq!(
            idx.owner_of("/usr/lib/jvm/java-21-openjdk/lib/foo.so"),
            Some("jre-openjdk")
        );
    }

    #[test]
    fn nearest_directory_wins() {
        let mut idx = OwnershipIndex::new(IndexScope::Executables);
        idx.insert("usr/", "filesystem");
        idx.insert("usr/lib/foo/", "foo");
        assert_eq!(idx.owner_of("usr/lib/foo/runtime.dat"), Some("foo"));
        assert_eq!(
            idx.owner_of("usr/lib/other/runtime.dat"),
            Some("filesystem")
        );
    }

    #[test]
    fn unknown_path_has_no_owner() {
        let idx = OwnershipIndex::new(IndexScope::Executables);
        assert_eq!(idx.owner_of("/home/user/script.sh"), None);
    }
}
