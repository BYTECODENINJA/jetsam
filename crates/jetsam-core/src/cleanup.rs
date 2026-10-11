//! Turning the dependency graph into a ranked list of cleanup candidates.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::graph::DepGraph;
use crate::model::PackageClass;
use crate::score::{score, CleanupScore, ScoreConfig, ScoreInput};

/// One ranked cleanup suggestion with its full, explainable breakdown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    /// Package name.
    pub name: String,
    /// Behavioural class, when known.
    pub class: Option<PackageClass>,
    /// Whether the user installed it explicitly.
    pub explicit: bool,
    /// Number of installed packages depending on it.
    pub reverse_dep_count: usize,
    /// Days since last use, or `None` when unknown.
    pub days_since_last_use: Option<f64>,
    /// Bytes freed by removing this package and its exclusive orphans.
    pub reclaimable_bytes: i64,
    /// Dependencies that would become orphans and be removed too.
    pub exclusive_orphans: Vec<String>,
    /// Pairs where a retained package requires this one — a warning.
    pub breaks: Vec<(String, String)>,
    /// The score and its components.
    pub score: CleanupScore,
}

/// Rank every removable package by cleanup score.
///
/// Protected packages are never included. `usage_days` maps a package name to
/// days-since-last-use; absent entries are treated as unknown usage.
pub fn rank_cleanup(
    graph: &DepGraph,
    classes: &HashMap<String, PackageClass>,
    usage_days: &HashMap<String, f64>,
    cfg: &ScoreConfig,
) -> Vec<Candidate> {
    let mut out = Vec::new();

    for (name, node) in graph.nodes() {
        if node.protected {
            continue;
        }
        let Ok(plan) = graph.removal_plan(std::slice::from_ref(name)) else {
            continue;
        };
        let input = ScoreInput {
            reclaimable_bytes: plan.freed_bytes,
            days_since_last_use: usage_days.get(name).copied(),
            explicit: node.explicit,
            reverse_dep_count: graph.reverse_count(name),
        };
        out.push(Candidate {
            name: name.clone(),
            class: classes.get(name).copied(),
            explicit: node.explicit,
            reverse_dep_count: input.reverse_dep_count,
            days_since_last_use: input.days_since_last_use,
            reclaimable_bytes: plan.freed_bytes,
            exclusive_orphans: plan.exclusive_orphans,
            breaks: plan.breaks,
            score: score(&input, cfg),
        });
    }

    out.sort_by(|a, b| {
        b.score
            .total
            .cmp(&a.score.total)
            .then_with(|| b.reclaimable_bytes.cmp(&a.reclaimable_bytes))
            .then_with(|| a.name.cmp(&b.name))
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::PackageNode;

    fn node(name: &str, explicit: bool, size: i64, requires: &[&str]) -> PackageNode {
        PackageNode {
            name: name.to_string(),
            explicit,
            installed_size: size,
            protected: false,
            requires: requires.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn graph(nodes: Vec<PackageNode>) -> DepGraph {
        let mut g = DepGraph::new();
        for n in nodes {
            g.insert(n);
        }
        g.finalize();
        g
    }

    #[test]
    fn protected_packages_are_excluded() {
        let mut sys = node("systemd", false, 1024, &[]);
        sys.protected = true;
        let g = graph(vec![node("app", true, 1024, &[]), sys]);

        let ranked = rank_cleanup(
            &g,
            &HashMap::new(),
            &HashMap::new(),
            &ScoreConfig::default(),
        );
        let names: Vec<&str> = ranked.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"app"));
        assert!(!names.contains(&"systemd"));
    }

    #[test]
    fn unused_explicit_package_beats_used_one() {
        let g = graph(vec![
            node("forgotten", true, 100 * 1024 * 1024, &[]),
            node("loved", true, 100 * 1024 * 1024, &[]),
        ]);
        let mut usage = HashMap::new();
        usage.insert("loved".to_string(), 0.0);
        usage.insert("forgotten".to_string(), 300.0);

        let ranked = rank_cleanup(&g, &HashMap::new(), &usage, &ScoreConfig::default());
        assert_eq!(ranked[0].name, "forgotten");
    }

    #[test]
    fn reclaimable_includes_exclusive_orphans() {
        let g = graph(vec![
            node("app", true, 1000, &["lib"]),
            node("lib", false, 2000, &[]),
        ]);
        let ranked = rank_cleanup(
            &g,
            &HashMap::new(),
            &HashMap::new(),
            &ScoreConfig::default(),
        );
        let app = ranked.iter().find(|c| c.name == "app").unwrap();
        assert_eq!(app.reclaimable_bytes, 3000);
        assert_eq!(app.exclusive_orphans, vec!["lib".to_string()]);
    }
}
