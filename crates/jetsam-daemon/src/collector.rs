//! The collector: reads `/proc`, attributes processes to packages via the
//! ownership index, and produces [`SampleOutcome`]s.

use jetsam_core::ownership::OwnershipIndex;

use crate::procfs;
use crate::sampler::{ProcessInfo, SampleEngine, SampleOutcome};

/// Owns the sampling engine and the ownership index.
pub struct Collector {
    engine: SampleEngine,
    index: OwnershipIndex,
}

impl Collector {
    /// Create a collector from a prebuilt ownership index.
    pub fn new(index: OwnershipIndex) -> Self {
        Collector {
            engine: SampleEngine::new(),
            index,
        }
    }

    /// Number of entries in the ownership index.
    pub fn index_len(&self) -> usize {
        self.index.len()
    }

    /// Take a live snapshot of `/proc` and observe it.
    ///
    /// `include_pss` controls the expensive `smaps_rollup` pass.
    pub fn scan(&mut self, include_pss: bool) -> SampleOutcome {
        let pids = procfs::list_pids();
        let mut procs = Vec::with_capacity(pids.len());
        for pid in pids {
            let Some(stat) = procfs::read_stat(pid) else {
                continue;
            };
            let Some(exe) = procfs::read_exe(pid) else {
                continue;
            };
            let pss_kb = if include_pss {
                procfs::read_pss_kb(pid)
            } else {
                None
            };
            procs.push(ProcessInfo {
                pid: stat.pid,
                ppid: stat.ppid,
                starttime: stat.starttime,
                exe,
                pss_kb,
            });
        }
        self.observe(&procs, include_pss, crate::now_epoch())
    }

    /// Observe a caller-supplied snapshot (used by tests and `jetsamd once`).
    pub fn observe(&mut self, procs: &[ProcessInfo], include_pss: bool, now: i64) -> SampleOutcome {
        let index = &self.index;
        self.engine.observe(
            procs,
            |exe| index.owner_of(exe).map(str::to_string),
            now.max(0) as u64,
            include_pss,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jetsam_core::ownership::IndexScope;

    #[test]
    fn observe_attributes_owned_executables() {
        let mut index = OwnershipIndex::new(IndexScope::Executables);
        index.insert("usr/bin/firefox", "firefox");
        let mut collector = Collector::new(index);

        let procs = vec![
            ProcessInfo {
                pid: 10,
                ppid: 1,
                starttime: 1,
                exe: "/usr/bin/firefox".to_string(),
                pss_kb: None,
            },
            ProcessInfo {
                pid: 11,
                ppid: 1,
                starttime: 2,
                exe: "/home/me/script.py".to_string(),
                pss_kb: None,
            },
        ];
        let outcome = collector.observe(&procs, false, 1_000);
        assert_eq!(outcome.launches, vec!["firefox".to_string()]);
        assert_eq!(outcome.attributed_processes, 1);
        assert_eq!(outcome.examined_processes, 2);
    }
}
