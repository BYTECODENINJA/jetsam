//! The transparent cleanup score.
//!
//! The score is a weighted sum of four normalised components, all in `[0, 1]`:
//!
//! * `space`     — how much removing the package (plus its exclusive orphans) frees
//! * `staleness` — how long since the package was last used
//! * `explicit`  — user-installed and unused scores higher than a dependency
//! * `safety`    — fewer reverse dependencies is safer
//!
//! The per-component values are returned alongside the total so the UI can show
//! the exact breakdown. When there is no usage history,
//! `staleness` is `None` and its weight is removed and the remaining weights are
//! renormalised, so a fresh install never mass-suggests everything just because
//! nothing has been "used" yet.

use serde::{Deserialize, Serialize};

fn clamp01(v: f64) -> f64 {
    if v.is_nan() {
        0.0
    } else {
        v.clamp(0.0, 1.0)
    }
}

/// Relative weights of the four components. They are normalised at scoring time.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScoreWeights {
    /// Weight of the reclaimed-space component.
    pub space: f64,
    /// Weight of the staleness component.
    pub staleness: f64,
    /// Weight of the explicitness component.
    pub explicit: f64,
    /// Weight of the safety component.
    pub safety: f64,
}

impl Default for ScoreWeights {
    fn default() -> Self {
        ScoreWeights {
            space: 0.30,
            staleness: 0.30,
            explicit: 0.20,
            safety: 0.20,
        }
    }
}

/// Tunable caps that map raw values onto `[0, 1]`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScoreConfig {
    /// Reclaimable bytes that count as "maximum space". Default 4 GiB.
    pub space_cap_bytes: f64,
    /// Days-since-use that counts as "fully stale". Default 365.
    pub stale_cap_days: f64,
    /// Reverse-dependency count that counts as "maximum risk". Default 20.
    pub reverse_dep_cap: f64,
    /// Component weights.
    pub weights: ScoreWeights,
}

impl Default for ScoreConfig {
    fn default() -> Self {
        ScoreConfig {
            space_cap_bytes: 4.0 * 1024.0 * 1024.0 * 1024.0,
            stale_cap_days: 365.0,
            reverse_dep_cap: 20.0,
            weights: ScoreWeights::default(),
        }
    }
}

/// Inputs to a single package's score.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScoreInput {
    /// Bytes freed if this package (and its exclusive orphans) were removed.
    pub reclaimable_bytes: i64,
    /// Days since the package was last used, or `None` when unknown.
    pub days_since_last_use: Option<f64>,
    /// Whether the user installed it explicitly.
    pub explicit: bool,
    /// Number of installed packages depending on it.
    pub reverse_dep_count: usize,
}

/// The normalised components, all in `[0, 1]`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScoreComponents {
    /// Reclaimed-space component.
    pub space: f64,
    /// Staleness component, or `None` when usage is unknown.
    pub staleness: Option<f64>,
    /// Explicitness component.
    pub explicit: f64,
    /// Safety component.
    pub safety: f64,
}

/// The final score and its breakdown.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CleanupScore {
    /// Total score in `0..=100`.
    pub total: u8,
    /// Per-component values.
    pub components: ScoreComponents,
}

/// Compute the cleanup score for one package.
pub fn score(input: &ScoreInput, cfg: &ScoreConfig) -> CleanupScore {
    let space_denom = (cfg.space_cap_bytes.max(1.0) + 1.0).ln();
    let space = if space_denom > 0.0 {
        clamp01((input.reclaimable_bytes.max(0) as f64 + 1.0).ln() / space_denom)
    } else {
        0.0
    };

    let staleness = input
        .days_since_last_use
        .map(|d| clamp01(d / cfg.stale_cap_days.max(1.0)));

    let explicit = if input.explicit { 1.0 } else { 0.5 };
    let safety = 1.0 - clamp01(input.reverse_dep_count as f64 / cfg.reverse_dep_cap.max(1.0));

    let w = cfg.weights;
    let mut numerator = w.space * space + w.explicit * explicit + w.safety * safety;
    let mut denominator = w.space + w.explicit + w.safety;
    if let Some(s) = staleness {
        numerator += w.staleness * s;
        denominator += w.staleness;
    }

    let frac = if denominator > 0.0 {
        clamp01(numerator / denominator)
    } else {
        0.0
    };

    CleanupScore {
        total: (frac * 100.0).round().clamp(0.0, 100.0) as u8,
        components: ScoreComponents {
            space,
            staleness,
            explicit,
            safety,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> ScoreInput {
        ScoreInput {
            reclaimable_bytes: 0,
            days_since_last_use: Some(0.0),
            explicit: false,
            reverse_dep_count: 0,
        }
    }

    #[test]
    fn more_space_scores_higher() {
        let cfg = ScoreConfig::default();
        let small = score(
            &ScoreInput {
                reclaimable_bytes: 1024,
                ..base()
            },
            &cfg,
        );
        let big = score(
            &ScoreInput {
                reclaimable_bytes: 1024 * 1024 * 1024,
                ..base()
            },
            &cfg,
        );
        assert!(big.total >= small.total);
        assert!(big.components.space > small.components.space);
    }

    #[test]
    fn stale_beats_recent() {
        let cfg = ScoreConfig::default();
        let recent = score(
            &ScoreInput {
                days_since_last_use: Some(1.0),
                ..base()
            },
            &cfg,
        );
        let stale = score(
            &ScoreInput {
                days_since_last_use: Some(400.0),
                ..base()
            },
            &cfg,
        );
        assert!(stale.total > recent.total);
        assert_eq!(stale.components.staleness, Some(1.0));
    }

    #[test]
    fn explicit_unused_beats_dependency() {
        let cfg = ScoreConfig::default();
        let dep = score(&base(), &cfg);
        let explicit = score(
            &ScoreInput {
                explicit: true,
                ..base()
            },
            &cfg,
        );
        assert!(explicit.total > dep.total);
    }

    #[test]
    fn many_reverse_deps_lower_the_score() {
        let cfg = ScoreConfig::default();
        let safe = score(&base(), &cfg);
        let risky = score(
            &ScoreInput {
                reverse_dep_count: 20,
                ..base()
            },
            &cfg,
        );
        assert!(risky.total < safe.total);
        assert_eq!(risky.components.safety, 0.0);
    }

    #[test]
    fn unknown_usage_renormalises_without_faking_staleness() {
        let cfg = ScoreConfig::default();
        let unknown = score(
            &ScoreInput {
                days_since_last_use: None,
                ..base()
            },
            &cfg,
        );
        assert_eq!(unknown.components.staleness, None);
        // No history must not, by itself, look like "fully stale".
        let stale = score(
            &ScoreInput {
                days_since_last_use: Some(365.0),
                ..base()
            },
            &cfg,
        );
        assert!(unknown.total < stale.total);
    }

    #[test]
    fn total_is_always_in_range() {
        let cfg = ScoreConfig::default();
        for bytes in [-10, 0, 1, 1 << 40] {
            for days in [None, Some(-5.0), Some(0.0), Some(10_000.0)] {
                for revdeps in [0, 5, 100] {
                    let s = score(
                        &ScoreInput {
                            reclaimable_bytes: bytes,
                            days_since_last_use: days,
                            explicit: true,
                            reverse_dep_count: revdeps,
                        },
                        &cfg,
                    );
                    assert!(s.total <= 100);
                    assert!(s.components.space >= 0.0 && s.components.space <= 1.0);
                }
            }
        }
    }
}
