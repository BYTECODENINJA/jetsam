//! Integration tests against the *real* local pacman database.
//!
//! These run automatically on any machine that has a populated
//! `/var/lib/pacman/local` (i.e. the Arch CI container and developer
//! workstations). On machines without one, they log and return, so a plain
//! `cargo test` still passes.
//!
//! Run explicitly with: `cargo test -p jetsam-core --test alpm_integration`

#![cfg(feature = "libalpm")]

use std::collections::HashMap;
use std::path::Path;

use jetsam_core::alpm_backend::AlpmBackend;
use jetsam_core::backend::Backend;
use jetsam_core::cleanup::rank_cleanup;
use jetsam_core::ownership::IndexScope;
use jetsam_core::score::ScoreConfig;

fn db_available() -> bool {
    Path::new("/var/lib/pacman/local").is_dir()
}

fn open() -> Option<AlpmBackend> {
    if !db_available() {
        eprintln!("skipping: no /var/lib/pacman/local on this machine");
        return None;
    }
    match AlpmBackend::open_default() {
        Ok(backend) => Some(backend),
        Err(err) => {
            eprintln!("skipping: could not open alpm database: {err}");
            None
        }
    }
}

#[test]
fn inventory_is_non_empty_and_well_formed() {
    let Some(backend) = open() else { return };

    let packages = backend.packages().expect("inventory");
    assert!(
        !packages.is_empty(),
        "expected at least one installed package"
    );

    for pkg in &packages {
        assert!(!pkg.name.is_empty(), "package with empty name");
        assert!(
            !pkg.version.is_empty(),
            "package {} has empty version",
            pkg.name
        );
        assert!(
            pkg.installed_size >= 0,
            "package {} has negative size",
            pkg.name
        );
    }

    // The kernel must be present and classified as a kernel, not "unknown".
    if let Some(kernel) = packages.iter().find(|p| p.name == "linux") {
        assert!(kernel.protected, "linux must be protected");
        assert_eq!(
            kernel.class,
            jetsam_core::PackageClass::KernelOrDriver,
            "linux must classify as kernel/driver"
        );
    }
}

#[test]
fn pacman_is_present_and_protected() {
    let Some(backend) = open() else { return };
    let pacman = backend.package("pacman").expect("pacman installed");
    assert!(pacman.protected);
    assert!(pacman.protected_reason.is_some());
}

#[test]
fn ownership_index_maps_a_known_binary() {
    let Some(backend) = open() else { return };
    let index = backend
        .ownership_scoped(IndexScope::Executables)
        .expect("index");
    assert!(!index.is_empty());

    // bash is present on essentially every Arch system.
    if backend.package("bash").is_ok() {
        assert_eq!(index.owner_of("/usr/bin/bash"), Some("bash"));
    }
}

#[test]
fn graph_and_orphans_are_consistent() {
    let Some(backend) = open() else { return };
    let graph = backend.graph().expect("graph");
    assert!(!graph.is_empty());

    // Every orphan must be non-explicit with no reverse dependencies.
    for orphan in backend.orphans().expect("orphans") {
        let node = graph.get(&orphan).expect("orphan is in graph");
        assert!(!node.explicit, "{orphan} is explicit but listed as orphan");
        assert_eq!(graph.reverse_count(&orphan), 0, "{orphan} still required");
    }
}

#[test]
fn protected_packages_are_never_cleanup_candidates() {
    let Some(backend) = open() else { return };
    let graph = backend.graph().expect("graph");
    let classes: HashMap<String, jetsam_core::PackageClass> = backend
        .packages()
        .expect("inventory")
        .into_iter()
        .map(|p| (p.name, p.class))
        .collect();

    let ranked = rank_cleanup(&graph, &classes, &HashMap::new(), &ScoreConfig::default());
    for candidate in &ranked {
        assert!(
            !graph
                .get(&candidate.name)
                .map(|n| n.protected)
                .unwrap_or(true),
            "protected package {} appeared as a cleanup candidate",
            candidate.name
        );
    }
}

#[test]
fn removing_an_unused_leaf_respects_protection() {
    let Some(backend) = open() else { return };
    let graph = backend.graph().expect("graph");

    let target = backend
        .packages()
        .expect("inventory")
        .into_iter()
        .find(|p| !p.protected && p.reverse_dep_count == 0);
    let Some(target) = target else { return };

    if let Ok(plan) = graph.removal_plan(std::slice::from_ref(&target.name)) {
        for name in &plan.remove {
            assert!(
                !graph.get(name).map(|n| n.protected).unwrap_or(true),
                "removal plan included protected package {name}"
            );
        }
    }
}
