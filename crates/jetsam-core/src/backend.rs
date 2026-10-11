//! The [`Backend`] trait: the seam between pure core logic and a package source.
//!
//! v0.1 ships the libalpm implementation in [`crate::alpm_backend`]. Future
//! backends (Flatpak) implement this trait without touching scoring, cleanup, or
//! the UI.

use crate::error::Result;
use crate::graph::DepGraph;
use crate::model::{PackageDetail, PackageRecord, RepoKind};
use crate::ownership::{IndexScope, OwnershipIndex};

/// A source of installed packages and their relationships.
pub trait Backend {
    /// All installed packages, as inventory rows.
    fn packages(&self) -> Result<Vec<PackageRecord>>;

    /// A single installed package, as an inventory row.
    fn package(&self, name: &str) -> Result<PackageRecord>;

    /// A single installed package with its heavier detail.
    fn detail(&self, name: &str) -> Result<PackageDetail>;

    /// The installed dependency graph, ready for removal planning.
    fn graph(&self) -> Result<DepGraph>;

    /// The file-ownership index, using the backend's default scope.
    fn ownership(&self) -> Result<OwnershipIndex>;

    /// The file-ownership index with an explicit scope.
    fn ownership_scoped(&self, scope: IndexScope) -> Result<OwnershipIndex>;

    /// Packages installed as dependencies that nothing requires.
    fn orphans(&self) -> Result<Vec<String>>;

    /// The repository a package was installed from.
    fn repo_of(&self, name: &str) -> RepoKind;
}
