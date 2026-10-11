//! Shared application state: a cached, Send-friendly analysis derived from
//! libalpm. The libalpm handle itself is not stored (it is neither `Send` nor
//! `Sync`); instead we rebuild the pure-data analysis on demand and cache it.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use jetsam_core::graph::DepGraph;
use jetsam_core::model::{PackageClass, PackageRecord};

/// Pure-data snapshot of the installed system, cheap to share between threads.
pub struct Analysis {
    /// Inventory rows.
    pub records: Vec<PackageRecord>,
    /// Inventory indexed by name.
    pub by_name: HashMap<String, PackageRecord>,
    /// Behavioural class by name.
    pub class_of: HashMap<String, PackageClass>,
    /// Installed dependency graph.
    pub graph: DepGraph,
    /// Packages installed as dependencies that nothing requires.
    pub orphans: Vec<String>,
}

impl Analysis {
    /// Packages on the protected list, with reasons.
    pub fn protected(&self) -> Vec<ProtectedItem> {
        self.records
            .iter()
            .filter(|r| r.protected)
            .map(|r| ProtectedItem {
                name: r.name.clone(),
                reason: r.protected_reason.clone().unwrap_or_default(),
            })
            .collect()
    }

    /// Total installed bytes.
    pub fn total_bytes(&self) -> i64 {
        self.records.iter().map(|r| r.installed_size.max(0)).sum()
    }

    /// Total installed bytes of orphaned packages.
    pub fn orphan_bytes(&self) -> i64 {
        self.orphans
            .iter()
            .filter_map(|name| self.by_name.get(name))
            .map(|r| r.installed_size.max(0))
            .sum()
    }

    /// Total installed bytes of foreign (AUR/local) packages.
    pub fn foreign_bytes(&self) -> i64 {
        self.records
            .iter()
            .filter(|r| r.repo.is_foreign())
            .map(|r| r.installed_size.max(0))
            .sum()
    }
}

/// A protected package and why it is protected.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProtectedItem {
    /// Package name.
    pub name: String,
    /// Human-readable reason.
    pub reason: String,
}

/// The global app state.
#[derive(Default)]
pub struct AppState {
    /// Cached analysis, if it has been built.
    pub analysis: Mutex<Option<Arc<Analysis>>>,
}

impl AppState {
    /// Return the cached analysis, if present.
    pub fn get(&self) -> Option<Arc<Analysis>> {
        self.analysis.lock().ok().and_then(|guard| guard.clone())
    }

    /// Replace the cached analysis.
    pub fn set(&self, analysis: Arc<Analysis>) {
        if let Ok(mut guard) = self.analysis.lock() {
            *guard = Some(analysis);
        }
    }

    /// Drop the cached analysis.
    pub fn clear(&self) {
        if let Ok(mut guard) = self.analysis.lock() {
            *guard = None;
        }
    }
}
