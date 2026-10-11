//! The sampling engine: turns process snapshots into launches, uptime deltas,
//! and PSS samples per package.
//!
//! The engine is deliberately pure — it takes a slice of [`ProcessInfo`] and an
//! `exe → package` resolver — so the accounting rules are unit tested without a
//! live system.
//!
//! # Rules
//!
//! * A **launch** is a *root* process: one with no ancestor (walking PPIDs)
//!   belonging to the same package. `firefox` spawning its own helpers is one
//!   launch; a shell spawning `vim` is a `vim` launch.
//! * **Uptime** accrues only for root processes, so an application's session
//!   time is counted once rather than summed across its process tree. A
//!   long-lived daemon (whose parent is `systemd --user`) accrues uptime every
//!   sample and never a launch after the first.
//! * **PSS** is summed per package over all mapped processes at the sample, and
//!   is a separate, slower pass because `smaps_rollup` is the expensive read.

use std::collections::{HashMap, HashSet};

/// A process observed at one sampling instant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    /// Process id.
    pub pid: u32,
    /// Parent process id.
    pub ppid: u32,
    /// Start time in clock ticks (distinguishes pid reuse).
    pub starttime: u64,
    /// Resolved executable path.
    pub exe: String,
    /// Proportional set size in kB, when this sample included a PSS pass.
    pub pss_kb: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
struct RunningProc {
    last_seen: u64,
    starttime: u64,
}

/// The result of one observation, ready to be written to the store.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SampleOutcome {
    /// One entry per counted launch.
    pub launches: Vec<String>,
    /// `(package, seconds)` uptime accrued since the previous sample.
    pub uptime: Vec<(String, u64)>,
    /// `(package, pss_kb)` for this PSS pass.
    pub pss: Vec<(String, u64)>,
    /// Number of processes that mapped to a package.
    pub attributed_processes: usize,
    /// Number of processes examined.
    pub examined_processes: usize,
}

/// Stateful sampling engine.
#[derive(Debug, Default)]
pub struct SampleEngine {
    running: HashMap<u32, RunningProc>,
    counted: HashSet<(u32, u64)>,
}

const MAX_ANCESTOR_DEPTH: usize = 128;

impl SampleEngine {
    /// Create an empty engine.
    pub fn new() -> Self {
        SampleEngine::default()
    }

    /// Observe a snapshot. `pkg_of` resolves an executable path to a package.
    pub fn observe<F>(
        &mut self,
        procs: &[ProcessInfo],
        mut pkg_of: F,
        now: u64,
        include_pss: bool,
    ) -> SampleOutcome
    where
        F: FnMut(&str) -> Option<String>,
    {
        // Map each process to a package.
        let mut info: HashMap<u32, &ProcessInfo> = HashMap::with_capacity(procs.len());
        let mut pkg_of_pid: HashMap<u32, String> = HashMap::new();
        for proc in procs {
            info.insert(proc.pid, proc);
            if let Some(pkg) = pkg_of(&proc.exe) {
                pkg_of_pid.insert(proc.pid, pkg);
            }
        }

        let is_root = |pid: u32| -> bool {
            let Some(pkg) = pkg_of_pid.get(&pid) else {
                return false;
            };
            let mut current = info.get(&pid).map(|p| p.ppid).unwrap_or(0);
            let mut depth = 0;
            while current != 0 && depth < MAX_ANCESTOR_DEPTH {
                if pkg_of_pid.get(&current).map(|p| p == pkg).unwrap_or(false) {
                    return false;
                }
                let Some(parent) = info.get(&current) else {
                    break;
                };
                if parent.ppid == current {
                    break;
                }
                current = parent.ppid;
                depth += 1;
            }
            true
        };

        let mut outcome = SampleOutcome {
            examined_processes: procs.len(),
            attributed_processes: pkg_of_pid.len(),
            ..SampleOutcome::default()
        };

        for proc in procs {
            let Some(pkg) = pkg_of_pid.get(&proc.pid) else {
                continue;
            };
            if !is_root(proc.pid) {
                continue;
            }

            let key = (proc.pid, proc.starttime);
            if !self.counted.contains(&key) {
                self.counted.insert(key);
                outcome.launches.push(pkg.clone());
            }

            let delta = match self.running.get(&proc.pid) {
                Some(prev) if prev.starttime == proc.starttime => {
                    now.saturating_sub(prev.last_seen)
                }
                _ => 0,
            };
            if delta > 0 {
                outcome.uptime.push((pkg.clone(), delta));
            }
            self.running.insert(
                proc.pid,
                RunningProc {
                    last_seen: now,
                    starttime: proc.starttime,
                },
            );
        }

        // Drop state for processes that are gone or that restarted (pid reuse).
        self.running.retain(|pid, prev| {
            info.get(pid)
                .map(|p| p.starttime == prev.starttime)
                .unwrap_or(false)
        });
        self.counted.retain(|(pid, starttime)| {
            info.get(pid)
                .map(|p| p.starttime == *starttime)
                .unwrap_or(false)
        });

        if include_pss {
            let mut sums: HashMap<String, u64> = HashMap::new();
            for proc in procs {
                if let (Some(pkg), Some(pss)) = (pkg_of_pid.get(&proc.pid), proc.pss_kb) {
                    *sums.entry(pkg.clone()).or_default() += pss;
                }
            }
            outcome.pss = sums.into_iter().collect();
        }

        outcome
    }

    /// Number of processes currently tracked as running roots.
    pub fn tracked_processes(&self) -> usize {
        self.running.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proc(pid: u32, ppid: u32, exe: &str, starttime: u64) -> ProcessInfo {
        ProcessInfo {
            pid,
            ppid,
            starttime,
            exe: exe.to_string(),
            pss_kb: None,
        }
    }

    fn resolver(map: &[(&str, &str)]) -> HashMap<String, String> {
        map.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn children_of_the_same_package_are_one_launch() {
        let mut engine = SampleEngine::new();
        let map = resolver(&[("/usr/lib/firefox/firefox", "firefox")]);
        let procs = vec![
            proc(100, 1, "/usr/lib/firefox/firefox", 10),
            proc(101, 100, "/usr/lib/firefox/firefox", 20),
            proc(102, 100, "/usr/lib/firefox/firefox", 21),
        ];
        let outcome = engine.observe(&procs, |e| map.get(e).cloned(), 1000, false);
        assert_eq!(outcome.launches, vec!["firefox".to_string()]);
        assert_eq!(outcome.attributed_processes, 3);
    }

    #[test]
    fn shell_spawning_another_package_is_a_new_launch() {
        let mut engine = SampleEngine::new();
        let map = resolver(&[("/usr/bin/bash", "bash"), ("/usr/bin/vim", "vim")]);
        let procs = vec![
            proc(200, 1, "/usr/bin/bash", 10),
            proc(201, 200, "/usr/bin/vim", 20),
        ];
        let outcome = engine.observe(&procs, |e| map.get(e).cloned(), 1000, false);
        assert!(outcome.launches.contains(&"bash".to_string()));
        assert!(outcome.launches.contains(&"vim".to_string()));
        assert_eq!(outcome.launches.len(), 2);
    }

    #[test]
    fn a_child_vim_under_bash_vim_is_one_launch() {
        let mut engine = SampleEngine::new();
        let map = resolver(&[("/usr/bin/bash", "bash"), ("/usr/bin/vim", "vim")]);
        let procs = vec![
            proc(200, 1, "/usr/bin/bash", 10),
            proc(201, 200, "/usr/bin/vim", 20),
            proc(202, 201, "/usr/bin/vim", 21),
        ];
        let outcome = engine.observe(&procs, |e| map.get(e).cloned(), 1000, false);
        assert_eq!(outcome.launches.iter().filter(|p| *p == "vim").count(), 1);
    }

    #[test]
    fn uptime_accrues_only_for_roots() {
        let mut engine = SampleEngine::new();
        let map = resolver(&[("/usr/lib/firefox/firefox", "firefox")]);
        let procs = vec![
            proc(100, 1, "/usr/lib/firefox/firefox", 10),
            proc(101, 100, "/usr/lib/firefox/firefox", 20),
        ];
        engine.observe(&procs, |e| map.get(e).cloned(), 1000, false);
        let outcome = engine.observe(&procs, |e| map.get(e).cloned(), 1010, false);
        // Only the root (pid 100) accrues 10 seconds.
        assert_eq!(outcome.uptime, vec![("firefox".to_string(), 10)]);
        assert!(outcome.launches.is_empty());
    }

    #[test]
    fn exited_process_stops_accruing_and_a_restart_relaunches() {
        let mut engine = SampleEngine::new();
        let map = resolver(&[("/usr/bin/foo", "foo")]);
        engine.observe(
            &[proc(5, 1, "/usr/bin/foo", 1)],
            |e| map.get(e).cloned(),
            100,
            false,
        );
        // Same pid, new starttime => pid reuse => new launch, no uptime.
        let outcome = engine.observe(
            &[proc(5, 1, "/usr/bin/foo", 2)],
            |e| map.get(e).cloned(),
            200,
            false,
        );
        assert_eq!(outcome.launches, vec!["foo".to_string()]);
        assert!(outcome.uptime.is_empty());
    }

    #[test]
    fn pss_is_summed_per_package_when_requested() {
        let mut engine = SampleEngine::new();
        let map = resolver(&[("/usr/bin/foo", "foo"), ("/usr/bin/bar", "bar")]);
        let mut a = proc(1, 1, "/usr/bin/foo", 1);
        a.pss_kb = Some(100);
        let mut b = proc(2, 1, "/usr/bin/foo", 2);
        b.pss_kb = Some(50);
        let mut c = proc(3, 1, "/usr/bin/bar", 3);
        c.pss_kb = Some(7);
        let outcome = engine.observe(&[a, b, c], |e| map.get(e).cloned(), 1000, true);
        let foo = outcome.pss.iter().find(|(p, _)| p == "foo").unwrap();
        assert_eq!(foo.1, 150);
        let bar = outcome.pss.iter().find(|(p, _)| p == "bar").unwrap();
        assert_eq!(bar.1, 7);
    }

    #[test]
    fn pss_is_skipped_when_not_requested() {
        let mut engine = SampleEngine::new();
        let map = resolver(&[("/usr/bin/foo", "foo")]);
        let mut a = proc(1, 1, "/usr/bin/foo", 1);
        a.pss_kb = Some(100);
        let outcome = engine.observe(&[a], |e| map.get(e).cloned(), 1000, false);
        assert!(outcome.pss.is_empty());
    }

    #[test]
    fn unmapped_processes_are_ignored() {
        let mut engine = SampleEngine::new();
        let map = resolver(&[]);
        let outcome = engine.observe(
            &[proc(1, 1, "/home/user/script.py", 1)],
            |e| map.get(e).cloned(),
            1000,
            false,
        );
        assert!(outcome.launches.is_empty());
        assert_eq!(outcome.attributed_processes, 0);
    }
}
