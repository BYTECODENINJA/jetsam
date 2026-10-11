//! The installed dependency graph and safe removal planning.
//!
//! This is the engine behind the flagship "reclaim space safely" feature. It is
//! pure data + algorithms with no libalpm dependency, so it is exhaustively
//! unit tested with synthetic graphs and reused verbatim by the live backend.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// One installed package in the graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageNode {
    /// Package name.
    pub name: String,
    /// Whether the user installed it explicitly.
    pub explicit: bool,
    /// Installed size in bytes.
    pub installed_size: i64,
    /// Whether the package is protected (see [`crate::protected`]).
    pub protected: bool,
    /// Names of installed packages this one directly requires.
    pub requires: Vec<String>,
}

/// Why a removal request was refused outright.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum CleanupRefusal {
    /// A protected package was named.
    #[error("package is protected and cannot be removed: {0}")]
    Protected(String),
    /// A named package is not installed.
    #[error("unknown package: {0}")]
    Unknown(String),
}

/// Exactly what a removal would delete, computed before anything is run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemovalPlan {
    /// Every package that would be removed: the selection plus pulled-in orphans.
    pub remove: Vec<String>,
    /// The subset of [`RemovalPlan::remove`] that is *not* the original selection,
    /// i.e. dependencies that become orphans. This is what makes the dry run
    /// honest and is shown prominently in the UI.
    pub exclusive_orphans: Vec<String>,
    /// Total bytes freed.
    pub freed_bytes: i64,
    /// `(dependent, dependency)` pairs where a retained package requires one of
    /// the selected packages. Non-empty means the removal would break the
    /// system and pacman would refuse it; surfaced as a warning.
    pub breaks: Vec<(String, String)>,
}

/// An installed dependency graph with a reverse index.
#[derive(Debug, Clone, Default)]
pub struct DepGraph {
    nodes: BTreeMap<String, PackageNode>,
    reverse: BTreeMap<String, BTreeSet<String>>,
}

impl DepGraph {
    /// Create an empty graph.
    pub fn new() -> Self {
        DepGraph::default()
    }

    /// Add a node. Edges are resolved in [`DepGraph::finalize`].
    pub fn insert(&mut self, node: PackageNode) {
        self.nodes.insert(node.name.clone(), node);
    }

    /// Build the reverse index. Must be called after all nodes are inserted.
    pub fn finalize(&mut self) {
        self.reverse.clear();
        for (name, node) in &self.nodes {
            for dep in &node.requires {
                if self.nodes.contains_key(dep) {
                    self.reverse
                        .entry(dep.clone())
                        .or_default()
                        .insert(name.clone());
                }
            }
        }
    }

    /// Number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the graph has no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Look up a node.
    pub fn get(&self, name: &str) -> Option<&PackageNode> {
        self.nodes.get(name)
    }

    /// Mutably look up a node (used while resolving edges).
    pub fn node_mut(&mut self, name: &str) -> Option<&mut PackageNode> {
        self.nodes.get_mut(name)
    }

    /// Iterate over all nodes.
    pub fn nodes(&self) -> impl Iterator<Item = (&String, &PackageNode)> {
        self.nodes.iter()
    }

    /// Iterate over the names that directly require `name`.
    pub fn required_by(&self, name: &str) -> impl Iterator<Item = &str> {
        self.reverse
            .get(name)
            .into_iter()
            .flat_map(|set| set.iter().map(|s| s.as_str()))
    }

    /// Count of packages that directly require `name`.
    pub fn reverse_count(&self, name: &str) -> usize {
        self.reverse.get(name).map(|s| s.len()).unwrap_or(0)
    }

    /// Packages installed only as dependencies and required by nothing.
    pub fn orphans(&self) -> Vec<String> {
        self.nodes
            .values()
            .filter(|n| !n.explicit && self.reverse_count(&n.name) == 0)
            .map(|n| n.name.clone())
            .collect()
    }

    /// Compute the full removal plan for a selection of package names.
    ///
    /// The plan includes dependency-reason packages that would become orphans:
    /// a non-explicit, non-protected package is pulled in when *every* package
    /// requiring it is itself being removed. Protected packages are never
    /// pulled in, which transitively protects their whole dependency subtree.
    pub fn removal_plan(&self, selected: &[String]) -> Result<RemovalPlan, CleanupRefusal> {
        for name in selected {
            match self.nodes.get(name) {
                None => return Err(CleanupRefusal::Unknown(name.clone())),
                Some(node) if node.protected => {
                    return Err(CleanupRefusal::Protected(name.clone()))
                }
                Some(_) => {}
            }
        }

        let selected_set: BTreeSet<&String> = selected.iter().collect();
        let mut remove: BTreeSet<String> = selected.iter().cloned().collect();

        loop {
            let mut changed = false;
            for (name, node) in &self.nodes {
                if remove.contains(name) || node.explicit || node.protected {
                    continue;
                }
                let Some(requirers) = self.reverse.get(name) else {
                    continue;
                };
                if requirers.is_empty() {
                    continue;
                }
                if requirers.iter().all(|r| remove.contains(r)) {
                    remove.insert(name.clone());
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        let mut breaks = Vec::new();
        for name in selected {
            for dependent in self.required_by(name) {
                if !remove.contains(dependent) {
                    breaks.push((dependent.to_string(), name.clone()));
                }
            }
        }

        let exclusive_orphans: Vec<String> = remove
            .iter()
            .filter(|n| !selected_set.contains(n))
            .cloned()
            .collect();
        let freed_bytes: i64 = remove
            .iter()
            .filter_map(|n| self.nodes.get(n))
            .map(|n| n.installed_size)
            .sum();

        Ok(RemovalPlan {
            remove: remove.into_iter().collect(),
            exclusive_orphans,
            freed_bytes,
            breaks,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(name: &str, explicit: bool, requires: &[&str]) -> PackageNode {
        PackageNode {
            name: name.to_string(),
            explicit,
            installed_size: 1024,
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
    fn orphans_are_dependency_reason_with_no_requirers() {
        let g = graph(vec![
            node("app", true, &["lib"]),
            node("lib", false, &[]),
            node("leftover", false, &[]),
        ]);
        assert_eq!(g.orphans(), vec!["leftover".to_string()]);
    }

    #[test]
    fn removing_app_pulls_its_exclusive_orphan() {
        let g = graph(vec![node("app", true, &["lib"]), node("lib", false, &[])]);
        let plan = g.removal_plan(&["app".to_string()]).unwrap();
        assert_eq!(plan.remove, vec!["app".to_string(), "lib".to_string()]);
        assert_eq!(plan.exclusive_orphans, vec!["lib".to_string()]);
        assert_eq!(plan.freed_bytes, 2048);
        assert!(plan.breaks.is_empty());
    }

    #[test]
    fn shared_dependency_is_not_removed() {
        let g = graph(vec![
            node("app", true, &["lib"]),
            node("other", true, &["lib"]),
            node("lib", false, &[]),
        ]);
        let plan = g.removal_plan(&["app".to_string()]).unwrap();
        assert_eq!(plan.remove, vec!["app".to_string()]);
        assert!(plan.exclusive_orphans.is_empty());
    }

    #[test]
    fn transitive_chain_is_pulled_in() {
        // app -> mid -> leaf, all dependency reason except app.
        let g = graph(vec![
            node("app", true, &["mid"]),
            node("mid", false, &["leaf"]),
            node("leaf", false, &[]),
        ]);
        let plan = g.removal_plan(&["app".to_string()]).unwrap();
        assert_eq!(
            plan.remove,
            vec!["app".to_string(), "leaf".to_string(), "mid".to_string()]
        );
        assert_eq!(plan.exclusive_orphans.len(), 2);
    }

    #[test]
    fn protected_package_is_refused() {
        let mut p = node("glibc", false, &[]);
        p.protected = true;
        let g = graph(vec![node("app", true, &["glibc"]), p]);
        assert_eq!(
            g.removal_plan(&["glibc".to_string()]),
            Err(CleanupRefusal::Protected("glibc".to_string()))
        );
    }

    #[test]
    fn protected_requirer_keeps_its_dependencies() {
        // systemd (protected, explicit) -> glibc; removing an unrelated app that
        // also depends on glibc must not orphan glibc.
        let mut systemd = node("systemd", true, &["glibc"]);
        systemd.protected = true;
        let g = graph(vec![
            node("app", true, &["glibc"]),
            systemd,
            node("glibc", false, &[]),
        ]);
        let plan = g.removal_plan(&["app".to_string()]).unwrap();
        assert_eq!(plan.remove, vec!["app".to_string()]);
    }

    #[test]
    fn breaking_a_retained_package_is_reported() {
        // app is required by keep, which is not being removed.
        let g = graph(vec![node("app", true, &[]), node("keep", true, &["app"])]);
        let plan = g.removal_plan(&["app".to_string()]).unwrap();
        assert_eq!(plan.breaks, vec![("keep".to_string(), "app".to_string())]);
    }

    #[test]
    fn unknown_package_is_refused() {
        let g = graph(vec![node("app", true, &[])]);
        assert_eq!(
            g.removal_plan(&["nope".to_string()]),
            Err(CleanupRefusal::Unknown("nope".to_string()))
        );
    }
}
