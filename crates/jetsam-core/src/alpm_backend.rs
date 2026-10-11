//! The live libalpm implementation of [`Backend`].
//!
//! We use libalpm through the `alpm` crate rather than shelling out to `pacman`:
//! it is the same library pacman uses, gives us the dependency graph and file
//! lists directly, and never risks parsing human-oriented command output.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use alpm::{Alpm, Package, PackageReason, SigLevel};

use crate::backend::Backend;
use crate::classify;
use crate::error::{CoreError, Result};
use crate::graph::{DepGraph, PackageNode};
use crate::model::{InstallReason, PackageDetail, PackageRecord, RepoKind};
use crate::ownership::{IndexScope, OwnershipIndex};
use crate::protected;

/// Default pacman database directory.
pub const DEFAULT_DBPATH: &str = "/var/lib/pacman";
/// Default filesystem root.
pub const DEFAULT_ROOT: &str = "/";

/// A [`Backend`] backed by the real libalpm database.
pub struct AlpmBackend {
    handle: Alpm,
    repo_of: HashMap<String, RepoKind>,
    sync_version: HashMap<String, String>,
}

impl AlpmBackend {
    /// Open the backend at the given root and database path.
    ///
    /// Any sync databases present in `<dbpath>/sync/*.db` are registered so the
    /// inventory can label packages by repository and detect foreign packages.
    /// A missing or empty sync directory is not an error: the inventory still
    /// works, every non-local package simply reads as foreign.
    pub fn open(root: &str, dbpath: &str) -> Result<Self> {
        let handle = Alpm::new(root, dbpath)?;

        let sync_dir = Path::new(dbpath).join("sync");
        let mut names = Vec::new();
        if let Ok(entries) = fs::read_dir(&sync_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("db") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        names.push(stem.to_string());
                    }
                }
            }
        }
        names.sort();
        names.dedup();

        for name in &names {
            // Signature checking is pacman's job during `-Sy`; here we only read
            // the already-verified local copy of the sync database.
            let _ = handle.register_syncdb(name.as_str(), SigLevel::NONE);
        }

        let mut backend = AlpmBackend {
            handle,
            repo_of: HashMap::new(),
            sync_version: HashMap::new(),
        };
        backend.rebuild_repo_map();
        Ok(backend)
    }

    /// Open using the standard system paths.
    pub fn open_default() -> Result<Self> {
        AlpmBackend::open(DEFAULT_ROOT, DEFAULT_DBPATH)
    }

    /// Rebuild the `name → repo` and `name → sync version` maps from the sync
    /// databases. Repositories are visited in sorted order, so `core` wins when
    /// a name appears in more than one.
    fn rebuild_repo_map(&mut self) {
        let mut repo_of: HashMap<String, RepoKind> = HashMap::new();
        let mut sync_version: HashMap<String, String> = HashMap::new();
        for db in self.handle.syncdbs() {
            let kind = RepoKind::from_name(db.name());
            for pkg in db.pkgs() {
                repo_of
                    .entry(pkg.name().to_string())
                    .or_insert_with(|| kind.clone());
                sync_version
                    .entry(pkg.name().to_string())
                    .or_insert_with(|| pkg.version().to_string());
            }
        }
        self.repo_of = repo_of;
        self.sync_version = sync_version;
    }

    /// The version available in the sync databases, if any. Used later by the
    /// update center; exposed now because the repo map already knows it.
    pub fn sync_version(&self, name: &str) -> Option<&str> {
        self.sync_version.get(name).map(String::as_str)
    }

    fn record_of(&self, pkg: &Package) -> PackageRecord {
        let name = pkg.name();
        let groups: Vec<String> = pkg.groups().iter().map(str::to_string).collect();
        let provides: Vec<String> = pkg
            .provides()
            .iter()
            .map(|d| d.name().to_string())
            .collect();
        let depends: Vec<String> = pkg.depends().iter().map(|d| d.name().to_string()).collect();
        let licenses: Vec<String> = pkg.licenses().iter().map(str::to_string).collect();
        let paths: Vec<String> = pkg
            .files()
            .files()
            .iter()
            .map(|f| String::from_utf8_lossy(f.name()).into_owned())
            .collect();

        let class = classify::classify_paths(name, paths.iter().map(String::as_str));
        let protected_reason = protected::protected_reason(name, &groups);

        PackageRecord {
            name: name.to_string(),
            version: pkg.version().to_string(),
            repo: self.repo_of.get(name).cloned().unwrap_or(RepoKind::Foreign),
            reason: match pkg.reason() {
                PackageReason::Explicit => InstallReason::Explicit,
                PackageReason::Depend => InstallReason::Dependency,
            },
            install_date: pkg.install_date(),
            installed_size: pkg.isize(),
            package_size: pkg.size(),
            description: pkg.desc().map(str::to_string),
            url: pkg.url().map(str::to_string),
            licenses,
            groups,
            provides,
            depends,
            class,
            protected: protected_reason.is_some(),
            protected_reason,
            reverse_dep_count: pkg.required_by().len(),
            file_count: paths.len(),
        }
    }
}

impl Backend for AlpmBackend {
    fn packages(&self) -> Result<Vec<PackageRecord>> {
        let local = self.handle.localdb();
        let mut out = Vec::with_capacity(local.pkgs().len());
        for pkg in local.pkgs() {
            out.push(self.record_of(pkg));
        }
        Ok(out)
    }

    fn package(&self, name: &str) -> Result<PackageRecord> {
        let pkg = self
            .handle
            .localdb()
            .pkg(name)
            .map_err(|_| CoreError::PackageNotFound(name.to_string()))?;
        Ok(self.record_of(pkg))
    }

    fn detail(&self, name: &str) -> Result<PackageDetail> {
        let pkg = self
            .handle
            .localdb()
            .pkg(name)
            .map_err(|_| CoreError::PackageNotFound(name.to_string()))?;
        let record = self.record_of(pkg);
        let files: Vec<String> = pkg
            .files()
            .files()
            .iter()
            .map(|f| String::from_utf8_lossy(f.name()).into_owned())
            .collect();
        let required_by: Vec<String> = pkg.required_by().into_iter().collect();
        Ok(PackageDetail {
            record,
            files,
            required_by,
        })
    }

    fn graph(&self) -> Result<DepGraph> {
        let local = self.handle.localdb();
        let installed = local.pkgs();

        // Pass 1: nodes with explicit/size/protected flags.
        let mut graph = DepGraph::new();
        for pkg in installed {
            let name = pkg.name();
            let groups: Vec<String> = pkg.groups().iter().map(str::to_string).collect();
            graph.insert(PackageNode {
                name: name.to_string(),
                explicit: matches!(pkg.reason(), PackageReason::Explicit),
                installed_size: pkg.isize(),
                protected: protected::protected_reason(name, &groups).is_some(),
                requires: Vec::new(),
            });
        }

        // Pass 2: resolve each dependency expression to an installed package.
        for pkg in installed {
            let name = pkg.name().to_string();
            let mut requires = Vec::new();
            for dep in pkg.depends() {
                let dep_name = dep.name();
                let resolved = if graph.get(dep_name).is_some() {
                    Some(dep_name.to_string())
                } else {
                    installed
                        .find_satisfier(dep_name)
                        .map(|p| p.name().to_string())
                };
                if let Some(resolved) = resolved {
                    if resolved != name {
                        requires.push(resolved);
                    }
                }
            }
            if let Some(node) = graph.node_mut(&name) {
                node.requires = requires;
            }
        }

        graph.finalize();
        Ok(graph)
    }

    fn ownership(&self) -> Result<OwnershipIndex> {
        self.ownership_scoped(IndexScope::Executables)
    }

    fn ownership_scoped(&self, scope: IndexScope) -> Result<OwnershipIndex> {
        let local = self.handle.localdb();
        let mut index = OwnershipIndex::new(scope);
        let mut packages = 0usize;
        for pkg in local.pkgs() {
            packages += 1;
            let name = pkg.name();
            for file in pkg.files().files() {
                let path = String::from_utf8_lossy(file.name());
                index.insert(&path, name);
            }
        }
        index.set_package_count(packages);
        Ok(index)
    }

    fn orphans(&self) -> Result<Vec<String>> {
        let local = self.handle.localdb();
        let mut out = Vec::new();
        for pkg in local.pkgs() {
            if matches!(pkg.reason(), PackageReason::Depend) && pkg.required_by().is_empty() {
                out.push(pkg.name().to_string());
            }
        }
        Ok(out)
    }

    fn repo_of(&self, name: &str) -> RepoKind {
        self.repo_of.get(name).cloned().unwrap_or(RepoKind::Foreign)
    }
}
