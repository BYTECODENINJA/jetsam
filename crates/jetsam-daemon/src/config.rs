//! Daemon configuration (TOML under XDG paths).

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{DaemonError, Result};

/// User-editable configuration. Every field has a sane default so first run is
/// zero-config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Whether usage collection is enabled. The daemon mirrors this into the
    /// database on first run; afterwards the database value wins so toggling it
    /// from the UI takes effect without a restart.
    pub opted_in: bool,
    /// Fast scan interval for launches and uptime, in seconds.
    pub sampling_interval_secs: u64,
    /// Slow interval for reading PSS from `smaps_rollup`, in seconds.
    pub pss_interval_secs: u64,
    /// Whether to read PSS at all.
    pub track_pss: bool,
    /// Daily rollup retention, in days.
    pub retention_days: u64,
    /// Launch-event retention, in days.
    pub launch_event_retention_days: u64,
    /// Whether desktop notifications are enabled.
    pub notify: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            opted_in: false,
            sampling_interval_secs: 5,
            pss_interval_secs: 30,
            track_pss: true,
            retention_days: 180,
            launch_event_retention_days: 30,
            notify: false,
        }
    }
}

impl Config {
    /// Load configuration from a path, returning defaults when the file does
    /// not exist.
    pub fn load(path: &Path) -> Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(text) => Ok(toml::from_str(&text)?),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
            Err(err) => Err(DaemonError::Io(err)),
        }
    }

    /// Write the configuration, creating parent directories as needed.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self).map_err(DaemonError::from)?;
        std::fs::write(path, text)?;
        Ok(())
    }

    /// Clamp pathological values from hand-edited files.
    pub fn sanitize(&mut self) {
        self.sampling_interval_secs = self.sampling_interval_secs.clamp(1, 3600);
        self.pss_interval_secs = self
            .pss_interval_secs
            .clamp(self.sampling_interval_secs, 86_400);
        self.retention_days = self.retention_days.clamp(1, 3650);
        self.launch_event_retention_days = self.launch_event_retention_days.clamp(1, 3650);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let cfg = Config::default();
        assert_eq!(cfg.sampling_interval_secs, 5);
        assert!(cfg.pss_interval_secs >= cfg.sampling_interval_secs);
        assert!(!cfg.opted_in);
    }

    #[test]
    fn sanitize_clamps_extremes() {
        let mut cfg = Config {
            sampling_interval_secs: 0,
            pss_interval_secs: 0,
            retention_days: 0,
            launch_event_retention_days: 9_999_999,
            ..Config::default()
        };
        cfg.sanitize();
        assert_eq!(cfg.sampling_interval_secs, 1);
        assert!(cfg.pss_interval_secs >= 1);
        assert_eq!(cfg.retention_days, 1);
        assert!(cfg.launch_event_retention_days <= 3650);
    }

    #[test]
    fn round_trips_through_toml() {
        let cfg = Config {
            opted_in: true,
            sampling_interval_secs: 10,
            ..Config::default()
        };
        let text = toml::to_string_pretty(&cfg).unwrap();
        let back: Config = toml::from_str(&text).unwrap();
        assert_eq!(cfg, back);
    }
}
