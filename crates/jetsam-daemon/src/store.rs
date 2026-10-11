//! SQLite persistence for usage samples.
//!
//! Four tables:
//!
//! * `usage` — one row per package: lifetime launch count, cumulative uptime,
//!   and an averaging PSS accumulator.
//! * `launch_event` — an append-only launch timeline (pruned by retention) so
//!   the UI can chart *when* a package is used, not just how often.
//! * `pss_daily` — per-day PSS rollups for sparkline history.
//! * `meta` / `ignored` — small key/value and preference tables.
//!
//! The database is the single source of truth for the opt-in state so the GUI
//! can toggle it without restarting the daemon.

use std::collections::BTreeMap;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

use jetsam_core::protocol::UsageInfo;

use crate::error::Result;
use crate::sampler::SampleOutcome;

const SCHEMA: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS usage (
    package          TEXT PRIMARY KEY,
    launches         INTEGER NOT NULL DEFAULT 0,
    uptime_secs      INTEGER NOT NULL DEFAULT 0,
    last_used_epoch  INTEGER,
    first_seen_epoch INTEGER,
    pss_sum_kb       INTEGER NOT NULL DEFAULT 0,
    pss_samples      INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS launch_event (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    package TEXT NOT NULL,
    epoch   INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_launch_event_epoch ON launch_event(epoch);

CREATE TABLE IF NOT EXISTS pss_daily (
    day        INTEGER NOT NULL,
    package    TEXT NOT NULL,
    pss_sum_kb INTEGER NOT NULL DEFAULT 0,
    samples    INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (day, package)
);

CREATE TABLE IF NOT EXISTS ignored (
    package TEXT PRIMARY KEY
);
"#;

const KEY_OPTED_IN: &str = "opted_in";
const KEY_FIRST_SAMPLE: &str = "first_sample_epoch";
const KEY_LAST_SAMPLE: &str = "last_sample_epoch";

/// Handle to the usage database.
pub struct Store {
    conn: Connection,
}

impl Store {
    /// Open (creating if needed) the database at `path`.
    pub fn open(path: &Path) -> Result<Self> {
        let in_memory = path.as_os_str() == ":memory:";
        if !in_memory {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
                restrict_dir(parent);
            }
        }
        let conn = Connection::open(path)?;
        let store = Store { conn };
        store.conn.execute_batch(SCHEMA)?;
        if !in_memory {
            restrict_file(path);
        }
        Ok(store)
    }

    /// Open an ephemeral in-memory store (tests).
    pub fn in_memory() -> Result<Self> {
        Self::open(Path::new(":memory:"))
    }

    fn get_meta(&self, key: &str) -> Result<Option<String>> {
        let value = self
            .conn
            .query_row(
                "SELECT value FROM meta WHERE key = ?1",
                params![key],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        Ok(value)
    }

    fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO meta(key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    fn del_meta(&self, key: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM meta WHERE key = ?1", params![key])?;
        Ok(())
    }

    /// Whether usage collection is enabled.
    pub fn is_opted_in(&self) -> Result<bool> {
        Ok(self.get_meta(KEY_OPTED_IN)?.as_deref() == Some("true"))
    }

    /// Whether the opt-in state has ever been written to the database. Used at
    /// startup to seed the database from the config file exactly once; after
    /// that the database is authoritative (so the UI can toggle it live).
    pub fn opt_in_recorded(&self) -> Result<bool> {
        Ok(self.get_meta(KEY_OPTED_IN)?.is_some())
    }

    /// Set the opt-in state, returning the new value.
    pub fn set_opt_in(&self, enabled: bool) -> Result<bool> {
        self.set_meta(KEY_OPTED_IN, if enabled { "true" } else { "false" })?;
        Ok(enabled)
    }

    /// The ignore list, sorted.
    pub fn ignored(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT package FROM ignored ORDER BY package")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// Add or remove a package from the ignore list. Ignoring a package also
    /// deletes any usage already recorded for it.
    pub fn set_ignored(&self, name: &str, ignored: bool) -> Result<()> {
        if ignored {
            self.conn.execute(
                "INSERT OR IGNORE INTO ignored(package) VALUES (?1)",
                params![name],
            )?;
            self.conn
                .execute("DELETE FROM usage WHERE package = ?1", params![name])?;
            self.conn
                .execute("DELETE FROM launch_event WHERE package = ?1", params![name])?;
        } else {
            self.conn
                .execute("DELETE FROM ignored WHERE package = ?1", params![name])?;
        }
        Ok(())
    }

    /// Apply one sample to the database. Packages on the ignore list are
    /// skipped.
    pub fn record_sample(&mut self, outcome: &SampleOutcome, epoch: i64) -> Result<()> {
        let ignored: std::collections::HashSet<String> = self.ignored()?.into_iter().collect();
        let tx = self.conn.transaction()?;
        let day = epoch.div_euclid(86_400);

        for pkg in &outcome.launches {
            if ignored.contains(pkg) {
                continue;
            }
            tx.execute(
                "INSERT INTO usage(package, launches, last_used_epoch, first_seen_epoch)
                 VALUES (?1, 1, ?2, ?2)
                 ON CONFLICT(package) DO UPDATE SET
                     launches = launches + 1,
                     last_used_epoch = MAX(COALESCE(last_used_epoch, excluded.last_used_epoch), excluded.last_used_epoch),
                     first_seen_epoch = MIN(COALESCE(first_seen_epoch, excluded.first_seen_epoch), excluded.first_seen_epoch)",
                params![pkg, epoch],
            )?;
            tx.execute(
                "INSERT INTO launch_event(package, epoch) VALUES (?1, ?2)",
                params![pkg, epoch],
            )?;
        }

        for (pkg, secs) in &outcome.uptime {
            if ignored.contains(pkg) {
                continue;
            }
            let secs = i64::try_from(*secs).unwrap_or(i64::MAX);
            tx.execute(
                "INSERT INTO usage(package, uptime_secs, last_used_epoch, first_seen_epoch)
                 VALUES (?1, ?2, ?3, ?3)
                 ON CONFLICT(package) DO UPDATE SET
                     uptime_secs = uptime_secs + excluded.uptime_secs,
                     last_used_epoch = MAX(COALESCE(last_used_epoch, excluded.last_used_epoch), excluded.last_used_epoch),
                     first_seen_epoch = MIN(COALESCE(first_seen_epoch, excluded.first_seen_epoch), excluded.first_seen_epoch)",
                params![pkg, secs, epoch],
            )?;
        }

        for (pkg, pss_kb) in &outcome.pss {
            if ignored.contains(pkg) {
                continue;
            }
            let pss_kb = i64::try_from(*pss_kb).unwrap_or(i64::MAX);
            tx.execute(
                "INSERT INTO usage(package, pss_sum_kb, pss_samples, last_used_epoch, first_seen_epoch)
                 VALUES (?1, ?2, 1, ?3, ?3)
                 ON CONFLICT(package) DO UPDATE SET
                     pss_sum_kb = pss_sum_kb + excluded.pss_sum_kb,
                     pss_samples = pss_samples + 1,
                     last_used_epoch = MAX(COALESCE(last_used_epoch, excluded.last_used_epoch), excluded.last_used_epoch),
                     first_seen_epoch = MIN(COALESCE(first_seen_epoch, excluded.first_seen_epoch), excluded.first_seen_epoch)",
                params![pkg, pss_kb, epoch],
            )?;
            tx.execute(
                "INSERT INTO pss_daily(day, package, pss_sum_kb, samples)
                 VALUES (?1, ?2, ?3, 1)
                 ON CONFLICT(day, package) DO UPDATE SET
                     pss_sum_kb = pss_sum_kb + excluded.pss_sum_kb,
                     samples = samples + 1",
                params![day, pkg, pss_kb],
            )?;
        }

        tx.execute(
            "INSERT INTO meta(key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![KEY_LAST_SAMPLE, epoch.to_string()],
        )?;
        tx.execute(
            "INSERT OR IGNORE INTO meta(key, value) VALUES (?1, ?2)",
            params![KEY_FIRST_SAMPLE, epoch.to_string()],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Usage for one package, if it has any history.
    pub fn usage(&self, name: &str) -> Result<Option<UsageInfo>> {
        let info = self
            .conn
            .query_row(
                "SELECT package, launches, uptime_secs, last_used_epoch, first_seen_epoch,
                        pss_sum_kb, pss_samples
                 FROM usage WHERE package = ?1",
                params![name],
                |row| self.row_to_usage(row),
            )
            .optional()?;
        Ok(info)
    }

    /// The most-used packages, ordered by launches then uptime.
    pub fn top_usage(&self, limit: u32) -> Result<Vec<UsageInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT package, launches, uptime_secs, last_used_epoch, first_seen_epoch,
                    pss_sum_kb, pss_samples
             FROM usage
             ORDER BY launches DESC, uptime_secs DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![i64::from(limit)], |row| self.row_to_usage(row))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// Usage for every package with history.
    pub fn usage_map(&self) -> Result<BTreeMap<String, UsageInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT package, launches, uptime_secs, last_used_epoch, first_seen_epoch,
                    pss_sum_kb, pss_samples
             FROM usage",
        )?;
        let rows = stmt.query_map([], |row| self.row_to_usage(row))?;
        let mut out = BTreeMap::new();
        for row in rows {
            let info = row?;
            out.insert(info.name.clone(), info);
        }
        Ok(out)
    }

    fn row_to_usage(&self, row: &rusqlite::Row<'_>) -> rusqlite::Result<UsageInfo> {
        let pss_sum_kb: i64 = row.get(5)?;
        let pss_samples: i64 = row.get(6)?;
        let avg_pss_bytes = if pss_samples > 0 {
            Some((pss_sum_kb.max(0) as u64).saturating_mul(1024) / (pss_samples as u64))
        } else {
            None
        };
        Ok(UsageInfo {
            name: row.get(0)?,
            launches: row.get::<_, i64>(1)?.max(0) as u64,
            uptime_secs: row.get::<_, i64>(2)?.max(0) as u64,
            last_used_epoch: row.get(3)?,
            first_seen_epoch: row.get(4)?,
            pss_samples: pss_samples.max(0) as u64,
            avg_pss_bytes,
        })
    }

    /// Number of distinct packages with recorded usage.
    pub fn packages_observed(&self) -> Result<u64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM usage", [], |row| row.get(0))?;
        Ok(count.max(0) as u64)
    }

    /// Number of packages on the ignore list.
    pub fn ignored_count(&self) -> Result<u64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM ignored", [], |row| row.get(0))?;
        Ok(count.max(0) as u64)
    }

    fn meta_epoch(&self, key: &str) -> Result<Option<i64>> {
        Ok(self
            .get_meta(key)?
            .and_then(|value| value.parse::<i64>().ok()))
    }

    /// Unix time of the first recorded sample.
    pub fn first_sample_epoch(&self) -> Result<Option<i64>> {
        self.meta_epoch(KEY_FIRST_SAMPLE)
    }

    /// Unix time of the most recent recorded sample.
    pub fn last_sample_epoch(&self) -> Result<Option<i64>> {
        self.meta_epoch(KEY_LAST_SAMPLE)
    }

    /// Number of distinct days with recorded launch history.
    pub fn days_of_history(&self) -> Result<u64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(DISTINCT epoch / 86400) FROM launch_event",
            [],
            |row| row.get(0),
        )?;
        Ok(count.max(0) as u64)
    }

    /// Delete all collected usage data. Preferences (opt-in, ignore list) are
    /// kept.
    pub fn wipe(&self) -> Result<()> {
        self.conn.execute_batch(
            "DELETE FROM usage;
             DELETE FROM launch_event;
             DELETE FROM pss_daily;",
        )?;
        self.del_meta(KEY_FIRST_SAMPLE)?;
        self.del_meta(KEY_LAST_SAMPLE)?;
        Ok(())
    }

    /// Drop data older than the retention windows, relative to `now`.
    pub fn prune(&self, retention_days: u64, event_days: u64, now: i64) -> Result<u64> {
        let usage_cutoff = now - (retention_days.min(i64::MAX as u64) as i64) * 86_400;
        let event_cutoff = now - (event_days.min(i64::MAX as u64) as i64) * 86_400;
        let events = self.conn.execute(
            "DELETE FROM launch_event WHERE epoch < ?1",
            params![event_cutoff],
        )?;
        let daily = self.conn.execute(
            "DELETE FROM pss_daily WHERE day < ?1",
            params![usage_cutoff.div_euclid(86_400)],
        )?;
        Ok((events + daily) as u64)
    }
}

fn restrict_dir(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(path) {
        let mut perms = meta.permissions();
        perms.set_mode(0o700);
        let _ = std::fs::set_permissions(path, perms);
    }
}

fn restrict_file(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(path) {
        let mut perms = meta.permissions();
        perms.set_mode(0o600);
        let _ = std::fs::set_permissions(path, perms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outcome(launches: &[&str], uptime: &[(&str, u64)], pss: &[(&str, u64)]) -> SampleOutcome {
        SampleOutcome {
            launches: launches.iter().map(|s| s.to_string()).collect(),
            uptime: uptime.iter().map(|(a, b)| (a.to_string(), *b)).collect(),
            pss: pss.iter().map(|(a, b)| (a.to_string(), *b)).collect(),
            attributed_processes: launches.len(),
            examined_processes: launches.len(),
        }
    }

    #[test]
    fn opt_in_round_trips() {
        let store = Store::in_memory().unwrap();
        assert!(!store.is_opted_in().unwrap());
        store.set_opt_in(true).unwrap();
        assert!(store.is_opted_in().unwrap());
    }

    #[test]
    fn launches_and_uptime_accumulate() {
        let mut store = Store::in_memory().unwrap();
        store
            .record_sample(&outcome(&["firefox"], &[("firefox", 30)], &[]), 1_000)
            .unwrap();
        store
            .record_sample(&outcome(&["firefox"], &[("firefox", 30)], &[]), 1_030)
            .unwrap();
        let info = store.usage("firefox").unwrap().unwrap();
        assert_eq!(info.launches, 2);
        assert_eq!(info.uptime_secs, 60);
        assert_eq!(info.first_seen_epoch, Some(1_000));
        assert_eq!(info.last_used_epoch, Some(1_030));
    }

    #[test]
    fn pss_is_averaged_in_bytes() {
        let mut store = Store::in_memory().unwrap();
        store
            .record_sample(&outcome(&[], &[], &[("foo", 100)]), 10)
            .unwrap();
        store
            .record_sample(&outcome(&[], &[], &[("foo", 300)]), 20)
            .unwrap();
        let info = store.usage("foo").unwrap().unwrap();
        assert_eq!(info.pss_samples, 2);
        assert_eq!(info.avg_pss_bytes, Some(200 * 1024));
    }

    #[test]
    fn top_usage_orders_by_launches() {
        let mut store = Store::in_memory().unwrap();
        store
            .record_sample(&outcome(&["a", "a", "b"], &[], &[]), 1)
            .unwrap();
        let top = store.top_usage(10).unwrap();
        assert_eq!(top[0].name, "a");
        assert_eq!(top[0].launches, 2);
        assert_eq!(top[1].name, "b");
    }

    #[test]
    fn ignore_removes_usage() {
        let mut store = Store::in_memory().unwrap();
        store
            .record_sample(&outcome(&["foo"], &[], &[]), 1)
            .unwrap();
        store.set_ignored("foo", true).unwrap();
        assert_eq!(store.ignored().unwrap(), vec!["foo".to_string()]);
        assert!(store.usage("foo").unwrap().is_none());
        store.set_ignored("foo", false).unwrap();
        assert!(store.ignored().unwrap().is_empty());
    }

    #[test]
    fn ignored_packages_are_not_recorded_even_after_unignore() {
        let mut store = Store::in_memory().unwrap();
        store.set_ignored("foo", true).unwrap();
        store
            .record_sample(&outcome(&["foo", "bar"], &[("foo", 10)], &[("bar", 5)]), 1)
            .unwrap();
        assert!(store.usage("foo").unwrap().is_none());
        let bar = store.usage("bar").unwrap().unwrap();
        assert_eq!(bar.launches, 1);
        assert_eq!(bar.uptime_secs, 0);
        assert_eq!(bar.avg_pss_bytes, Some(5 * 1024));
        store.set_ignored("foo", false).unwrap();
        store
            .record_sample(&outcome(&["foo"], &[], &[]), 2)
            .unwrap();
        let foo = store.usage("foo").unwrap().unwrap();
        assert_eq!(foo.launches, 1);
    }

    #[test]
    fn wipe_clears_usage_but_keeps_preferences() {
        let mut store = Store::in_memory().unwrap();
        store.set_opt_in(true).unwrap();
        store.set_ignored("bar", true).unwrap();
        store
            .record_sample(&outcome(&["foo"], &[], &[("foo", 1)]), 1)
            .unwrap();
        store.wipe().unwrap();
        assert_eq!(store.packages_observed().unwrap(), 0);
        assert!(store.first_sample_epoch().unwrap().is_none());
        assert!(store.is_opted_in().unwrap());
        assert_eq!(store.ignored().unwrap(), vec!["bar".to_string()]);
    }

    #[test]
    fn prune_drops_old_events() {
        let mut store = Store::in_memory().unwrap();
        let now = 100 * 86_400;
        store
            .record_sample(&outcome(&["old"], &[], &[]), 10)
            .unwrap();
        store
            .record_sample(&outcome(&["new"], &[], &[]), now)
            .unwrap();
        store.prune(180, 30, now).unwrap();
        let events: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM launch_event", [], |r| r.get(0))
            .unwrap();
        assert_eq!(events, 1);
    }

    #[test]
    fn days_of_history_counts_distinct_days() {
        let mut store = Store::in_memory().unwrap();
        store
            .record_sample(&outcome(&["a"], &[], &[]), 100)
            .unwrap();
        store
            .record_sample(&outcome(&["b"], &[], &[]), 200)
            .unwrap();
        store
            .record_sample(&outcome(&["c"], &[], &[]), 86_400 + 50)
            .unwrap();
        assert_eq!(store.days_of_history().unwrap(), 2);
    }
}
