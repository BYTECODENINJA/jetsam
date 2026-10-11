//! Reading and parsing `/proc`.
//!
//! Only the owning user's processes are visible to a user service, which is
//! exactly the scope the daemon wants. Parsing is split from reading so the
//! parsers can be unit tested against fixtures without a live `/proc`.

use std::path::Path;

/// The subset of `/proc/<pid>/stat` we care about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcStat {
    /// Process id.
    pub pid: u32,
    /// Parent process id.
    pub ppid: u32,
    /// Process start time in clock ticks since boot (guards against pid reuse).
    pub starttime: u64,
}

/// Parse a `/proc/<pid>/stat` line.
///
/// The second field is `(comm)` and may itself contain spaces and parentheses,
/// so we split on the first `(` and last `)` as the kernel's own documentation
/// recommends, then count whitespace-separated fields from there.
pub fn parse_stat(pid: u32, line: &str) -> Option<ProcStat> {
    let open = line.find('(')?;
    let close = line.rfind(')')?;
    if close < open {
        return None;
    }
    let after = line.get(close + 1..)?.trim_start();
    let fields: Vec<&str> = after.split_whitespace().collect();
    // `after` starts at field 3 (state); ppid is field 4, starttime is field 22.
    let ppid = fields.get(1)?.parse().ok()?;
    let starttime = fields.get(19)?.parse().ok()?;
    Some(ProcStat {
        pid,
        ppid,
        starttime,
    })
}

/// Parse the `Pss:` value (in kB) from a `smaps_rollup` file.
pub fn parse_pss_kb(contents: &str) -> Option<u64> {
    for line in contents.lines() {
        if let Some(rest) = line.strip_prefix("Pss:") {
            let value = rest.split_whitespace().next()?;
            return value.parse().ok();
        }
    }
    None
}

/// Strip the `" (deleted)"` suffix the kernel appends to exe links of replaced
/// or removed binaries.
pub fn strip_deleted(path: &str) -> &str {
    path.strip_suffix(" (deleted)").unwrap_or(path)
}

/// List numeric process ids under `/proc`.
pub fn list_pids() -> Vec<u32> {
    let mut pids = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if let Ok(pid) = name.parse::<u32>() {
                    pids.push(pid);
                }
            }
        }
    }
    pids
}

/// Read and parse `/proc/<pid>/stat`.
pub fn read_stat(pid: u32) -> Option<ProcStat> {
    let text = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    parse_stat(pid, &text)
}

/// Resolve `/proc/<pid>/exe` to a path string, stripping the deleted marker.
pub fn read_exe(pid: u32) -> Option<String> {
    let link = std::fs::read_link(format!("/proc/{pid}/exe")).ok()?;
    let text = link.to_string_lossy();
    let stripped = strip_deleted(&text);
    if stripped.is_empty() {
        None
    } else {
        Some(stripped.to_string())
    }
}

/// Read proportional set size (kB) from `/proc/<pid>/smaps_rollup`.
pub fn read_pss_kb(pid: u32) -> Option<u64> {
    let text = std::fs::read_to_string(format!("/proc/{pid}/smaps_rollup")).ok()?;
    parse_pss_kb(&text)
}

/// Read the numeric entries of a `/proc/<pid>/` directory (threads etc.).
pub fn is_alive(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stat_with_spaces_in_comm() {
        // A process whose command name contains spaces and parentheses.
        let line = "1234 (weird )(name) S 42 1234 1234 0 -1 4194560 1000 0 0 0 \
                    12 34 0 0 20 0 5 0 987654 12345678 900 18446744073709551615 \
                    0 0 0 0 0 0 0 0 0 0 0 0 17 0 0 0 0 0 0 0 0 0 0 0 0 0 0";
        let stat = parse_stat(1234, line).unwrap();
        assert_eq!(stat.pid, 1234);
        assert_eq!(stat.ppid, 42);
        assert_eq!(stat.starttime, 987654);
    }

    #[test]
    fn parses_simple_stat() {
        let line = "7 (init) S 1 7 7 0 -1 4194560 0 0 0 0 0 0 0 0 20 0 1 0 555 0 0 0";
        let stat = parse_stat(7, line).unwrap();
        assert_eq!(stat.ppid, 1);
        assert_eq!(stat.starttime, 555);
    }

    #[test]
    fn rejects_malformed_stat() {
        assert!(parse_stat(1, "not a stat line").is_none());
        assert!(parse_stat(1, "").is_none());
    }

    #[test]
    fn parses_pss() {
        let rollup = "\
Rss:   12345 kB
Pss:    6789 kB
Pss_Anon: 1000 kB
";
        assert_eq!(parse_pss_kb(rollup), Some(6789));
        assert_eq!(parse_pss_kb("Rss: 1 kB\n"), None);
    }

    #[test]
    fn strips_deleted_marker() {
        assert_eq!(strip_deleted("/usr/bin/foo (deleted)"), "/usr/bin/foo");
        assert_eq!(strip_deleted("/usr/bin/foo"), "/usr/bin/foo");
    }

    #[test]
    fn live_procfs_is_readable() {
        // The test process itself must be visible.
        let me = std::process::id();
        let stat = read_stat(me).expect("own stat");
        assert_eq!(stat.pid, me);
        assert!(read_exe(me).is_some());
    }
}
