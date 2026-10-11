//! Core domain logic for Jetsam.
//!
//! This crate is intentionally free of any UI, IPC, or privileged concerns. It
//! contains:
//!
//! * [`model`] — serializable domain types shared across the workspace.
//! * [`classify`] — the "a package is not an application" heuristics.
//! * [`protected`] — the set of packages that may never be suggested for removal.
//! * [`graph`] — the installed dependency graph and removal planning.
//! * [`score`] — the transparent cleanup scoring formula.
//! * [`cleanup`] — ranking packages into cleanup candidates.
//! * [`ownership`] — the process-executable → owning-package index.
//! * [`backend`] — the [`backend::Backend`] trait.
//! * [`alpm_backend`] — the live libalpm implementation (feature `libalpm`).
//!
//! Pure logic modules have no dependency on libalpm and are fully unit tested,
//! so `cargo test -p jetsam-core --no-default-features` runs anywhere.

#![forbid(unsafe_code)]

pub mod backend;
pub mod classify;
pub mod cleanup;
pub mod error;
pub mod graph;
pub mod model;
pub mod ownership;
pub mod protected;
pub mod protocol;
pub mod score;

#[cfg(feature = "libalpm")]
pub mod alpm_backend;

pub use error::{CoreError, Result};
pub use model::{InstallReason, PackageClass, PackageDetail, PackageRecord, RepoKind};
