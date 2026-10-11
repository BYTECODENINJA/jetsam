//! Tauri commands: the entire GUI ⇄ backend surface.
//!
//! Every command is a plain function returning `Result<T, String>`; the frontend
//! never touches libalpm, pacman, or pkexec directly.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use jetsam_core::alpm_backend::AlpmBackend;
use jetsam_core::backend::Backend;
use jetsam_core::graph::RemovalPlan;
use jetsam_core::model::{PackageDetail, PackageRecord};
use jetsam_core::protocol::{DaemonStatus, UsageInfo};
use jetsam_core::score::ScoreConfig;

use crate::state::{Analysis, AppState, ProtectedItem};
use crate::{daemon, helper};

/// Build a fresh analysis from the live system.
fn build_analysis() -> Result<Analysis, String> {
    let backend = AlpmBackend::open_default().map_err(|e| e.to_string())?;
    let records = backend.packages().map_err(|e| e.to_string())?;

    let mut by_name = HashMap::with_capacity(records.len());
    let mut class_of = HashMap::with_capacity(records.len());
    for record in &records {
        by_name.insert(record.name.clone(), record.clone());
        class_of.insert(record.name.clone(), record.class);
    }

    let graph = backend.graph().map_err(|e| e.to_string())?;
    let orphans = backend.orphans().map_err(|e| e.to_string())?;
    Ok(Analysis {
        records,
        by_name,
        class_of,
        graph,
        orphans,
    })
}

fn ensure_analysis(state: &AppState) -> Result<Arc<Analysis>, String> {
    if let Some(analysis) = state.get() {
        return Ok(analysis);
    }
    let analysis = Arc::new(build_analysis()?);
    state.set(Arc::clone(&analysis));
    Ok(analysis)
}

fn now_epoch() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Summary of the loaded inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySummary {
    /// Number of installed packages.
    pub packages: usize,
    /// Number of orphans.
    pub orphans: usize,
    /// Number of protected packages.
    pub protected: usize,
    /// Total installed bytes.
    pub total_bytes: i64,
    /// Bytes held by orphaned packages.
    pub orphan_bytes: i64,
    /// Bytes held by foreign (AUR/local) packages.
    pub foreign_bytes: i64,
    /// Time taken to build the analysis, in milliseconds.
    pub build_ms: u64,
}

/// The cleanup view: ranked candidates plus the protected section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupView {
    /// Ranked suggestions.
    pub candidates: Vec<jetsam_core::cleanup::Candidate>,
    /// Packages that are never suggested, with reasons.
    pub protected: Vec<ProtectedItem>,
    /// Whether any usage history exists (staleness is only meaningful then).
    pub usage_known: bool,
}

/// A pending package update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingUpdate {
    /// Package name.
    pub name: String,
    /// Currently installed version.
    pub installed: String,
    /// Version available in the sync databases.
    pub available: String,
}

/// Overall status for the top bar and banners.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStatus {
    /// Daemon status when reachable.
    pub daemon: Option<DaemonStatus>,
    /// Whether the inventory has been loaded.
    pub packages_loaded: bool,
    /// Number of installed packages (0 when not loaded).
    pub packages: usize,
    /// Whether the privileged helper is available.
    pub helper_available: bool,
    /// The helper path, for display.
    pub helper_path: Option<String>,
}

/// Load the inventory (rebuilding it when `force` is set).
#[tauri::command]
pub fn load_inventory(
    state: tauri::State<'_, AppState>,
    force: bool,
) -> Result<InventorySummary, String> {
    if force {
        state.clear();
    }
    let started = std::time::Instant::now();
    let analysis = ensure_analysis(&state)?;
    Ok(InventorySummary {
        packages: analysis.records.len(),
        orphans: analysis.orphans.len(),
        protected: analysis.protected().len(),
        total_bytes: analysis.total_bytes(),
        orphan_bytes: analysis.orphan_bytes(),
        foreign_bytes: analysis.foreign_bytes(),
        build_ms: started.elapsed().as_millis() as u64,
    })
}

/// The full package inventory.
#[tauri::command]
pub fn list_packages(state: tauri::State<'_, AppState>) -> Result<Vec<PackageRecord>, String> {
    Ok(ensure_analysis(&state)?.records.clone())
}

/// One package with its heavy detail (files, reverse deps).
#[tauri::command]
pub fn package_detail(name: String) -> Result<PackageDetail, String> {
    let backend = AlpmBackend::open_default().map_err(|e| e.to_string())?;
    backend.detail(&name).map_err(|e| e.to_string())
}

/// Ranked cleanup suggestions plus the protected list.
#[tauri::command]
pub fn cleanup_suggestions(state: tauri::State<'_, AppState>) -> Result<CleanupView, String> {
    let analysis = ensure_analysis(&state)?;
    let usage = daemon::usage_map();
    let now = now_epoch();

    let mut usage_days: HashMap<String, f64> = HashMap::new();
    let mut usage_known = false;
    if let Some(map) = &usage {
        for (name, info) in map {
            if let Some(last) = info.last_used_epoch {
                usage_known = true;
                usage_days.insert(name.clone(), ((now - last).max(0) as f64) / 86_400.0);
            }
        }
    }

    let candidates = jetsam_core::cleanup::rank_cleanup(
        &analysis.graph,
        &analysis.class_of,
        &usage_days,
        &ScoreConfig::default(),
    );

    Ok(CleanupView {
        candidates,
        protected: analysis.protected(),
        usage_known,
    })
}

/// Usage map from the daemon (empty when the daemon is down).
#[tauri::command]
pub fn usage_map() -> Result<BTreeMap<String, UsageInfo>, String> {
    Ok(daemon::usage_map().unwrap_or_default())
}

/// The daemon's ignore list.
#[tauri::command]
pub fn ignore_list() -> Result<Vec<String>, String> {
    Ok(daemon::ignore_list().unwrap_or_default())
}

/// Add or remove a package from the ignore list.
#[tauri::command]
pub fn set_ignored(name: String, ignored: bool) -> Result<(), String> {
    daemon::set_ignored(&name, ignored)
}

/// Enable or disable usage collection.
#[tauri::command]
pub fn set_opt_in(enabled: bool) -> Result<(), String> {
    daemon::set_opt_in(enabled)
}

/// Delete all collected usage data.
#[tauri::command]
pub fn wipe_data() -> Result<(), String> {
    daemon::wipe()
}

/// Overall application status.
#[tauri::command]
pub fn app_status(state: tauri::State<'_, AppState>) -> Result<AppStatus, String> {
    let analysis = state.get();
    Ok(AppStatus {
        daemon: daemon::status(),
        packages_loaded: analysis.is_some(),
        packages: analysis.as_ref().map(|a| a.records.len()).unwrap_or(0),
        helper_available: helper::available(),
        helper_path: helper::helper_path().map(|p| p.display().to_string()),
    })
}

/// Compute exactly what a removal would delete, before anything runs.
#[tauri::command]
pub fn dry_run(
    state: tauri::State<'_, AppState>,
    names: Vec<String>,
) -> Result<RemovalPlan, String> {
    let analysis = ensure_analysis(&state)?;
    analysis
        .graph
        .removal_plan(&names)
        .map_err(|e| e.to_string())
}

/// Perform a removal through the privileged helper.
#[tauri::command]
pub fn apply_remove(names: Vec<String>) -> Result<helper::HelperOutcome, String> {
    helper::remove(&names)
}

/// Run a full system upgrade through the privileged helper.
#[tauri::command]
pub fn run_upgrade() -> Result<helper::HelperOutcome, String> {
    helper::upgrade()
}

/// Pending updates: installed version differs from the sync version.
#[tauri::command]
pub fn check_updates(state: tauri::State<'_, AppState>) -> Result<Vec<PendingUpdate>, String> {
    let analysis = ensure_analysis(&state)?;
    let backend = AlpmBackend::open_default().map_err(|e| e.to_string())?;
    let mut pending = Vec::new();
    for record in &analysis.records {
        if let Some(available) = backend.sync_version(&record.name) {
            if available != record.version {
                pending.push(PendingUpdate {
                    name: record.name.clone(),
                    installed: record.version.clone(),
                    available: available.to_string(),
                });
            }
        }
    }
    pending.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(pending)
}

/// Start the daemon and report whether it came up.
#[tauri::command]
pub fn start_daemon() -> Result<bool, String> {
    daemon::start()
}

/// Whether the privileged helper is available.
#[tauri::command]
pub fn helper_available() -> Result<bool, String> {
    Ok(helper::available())
}
