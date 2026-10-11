# Jetsam Security Review (v0.1.0)

This is a threat-model-first product. The goal: remove cruft safely, never break the system, and never trust the GUI.

## Core Principles
- **Principle of least privilege**: the GUI and CLI run as the unprivileged user. Only `jetsam-helper` can run privileged actions; only when invoked through `pkexec` with arguments it explicitly validates.
- **No shell construction**: the helper invokes `/usr/bin/pacman` via `std::process::Command` with discrete arguments. No `sh -c`, no string interpolation into shell.
- **Explicit allowlist of verbs**: the helper accepts exactly `remove <pkg>...` and `upgrade`. Unknown subcommands are rejected.
- **Validate everything**: package names must match `^[A-Za-z0-9@._+-]+$`, must start with alphanumeric, length ≤ 255. The GUI never passes raw strings to pacman.
- **Hard protection**: the helper refuses any name returned by `jetsam_core::protected::is_protected` (base group, `pacman`, `glibc`, `systemd`, kernels, initramfs, firmware, graphics drivers, toolchains, etc.). The daemon also never suggests protected packages as removal candidates.
- **Versioned IPC**: daemon speaks protocol v1 over newline-delimited JSON. A client seeing a version mismatch refuses to talk.
- **No network**: the daemon has no network dependencies. The collector reads only local `/proc` and local filesystem paths. libalpm talks to local package DBs only.
- **User-scoped daemon**: `jetsamd` is a systemd user service. It runs under the logged-in user, sees only that user's processes, and cannot touch other users.

## Attack Surface

### 1) GUI → Daemon (Unix socket)
- **Transport**: `$XDG_RUNTIME_DIR/jetsam/daemon.sock`, mode 0600; the runtime directory is created 0700 so only the owning user can connect.
- **Input**: newline-delimited JSON. A line length limit (1 MiB) prevents unbounded reads. Malformed JSON is rejected with `bad_request` and the connection is not dropped unless the stream breaks.
- **Authorization**: filesystem permissions only. Any process running as the same user can connect (by design) — the daemon is a user-local service, not a system-wide multi-tenant API.
- **DoS**: sequential per-connection handling; a single slow client cannot starve others indefinitely, and requests are tiny. No global state mutation from unauthenticated "commands" except through the defined method set.
- **Versioning**: requests with `v != PROTOCOL_VERSION` return `protocol_mismatch` and are never dispatched.

### 2) Daemon
- **Privileged ops**: the daemon never attempts to elevate privileges. It never shells out to `pkexec`, `sudo`, or `pacman`. Privileged operations are delegated entirely to the helper.
- **Process attribution**: reads `/proc/<pid>/exe` (symlink), `/proc/<pid>/stat`, and optionally `/proc/<pid>/smaps_rollup`. Parsing is conservative and tested with fixtures. Path normalisation collapses `//` and strips the kernel's `" (deleted)"` suffix.
- **PSS**: the slower `smaps_rollup` pass is opt-in and rate-limited (`pss_interval_secs`). Failures reading a single process are silently ignored.
- **Storage**: SQLite database at `$XDG_DATA_HOME/jetsam/usage.db`, created with WAL and `synchronous=NORMAL`, mode 0600. WAL files inherit directory permissions. The store enforces ignore-list filtering on write (a package ignored before/after recording has its history purged on ignore; future samples skip it).
- **Config**: TOML at `$XDG_CONFIG_HOME/jetsam/config.toml`. Values are clamped (intervals 1–3600/86k, retention 1–3650). The opt-in state is seeded from config on first run but is the database's `meta.opted_in` after first write; the IPC `set_opt_in` persists back to config so restarts honour the last user choice.
- **Race conditions**: pid reuse is detected via `starttime`. The engine tracks running roots by `(pid, starttime)` and drops state when a process exits or restarts.
- **No telemetry**: no outbound network calls, no analytics, no crash reporting.

### 3) Helper (pkexec)
- **Elevation path**: invoked by the GUI/CLI via `pkexec jetsam-helper ...`. The helper does **not** depend on being setuid; pkexec is the trusted gate.
- **Root verification**: `jetsam-helper` re-checks `real uid == 0` by reading `/proc/self/status` (`Uid:` first field). If not root, it exits with code 2 and prints nothing sensitive to stdout.
- **Argument validation**:
  - `remove`: requires ≥1 and ≤256 package names. Each name passes the regex/length/leading-char check and `is_protected(...)`. Duplicates are removed.
  - `upgrade`: takes no extra arguments.
- **Command execution**: `Command::new("/usr/bin/pacman")` with `["--noconfirm", "-R", ...]` or `["--noconfirm", "-Syu"]`. Never shell-quoted. `-R` is the unconditional removal of named packages (no recursive `-Rns` by design). `-Syu` is full system upgrade only (no `-S`, no partial upgrades).
- **Output**: a single JSON line on stdout (`HelperResult`) containing `action`, `ok`, `exit_code`, `stdout`, `stderr`. On failure, a short human message is also written to stderr. The JSON is the contract consumed by the caller.
- **Failure safety**: protected packages are rejected before executing pacman. Invalid package names are rejected. Oversized lists rejected. Unknown subcommands rejected as `InvalidPackage` (a safe classification). The helper never "falls back" to shell.
- **CVE-mitigation posture**: argument injection is impossible because no shell interpolation occurs and every token is an argument; name validation blocks flags (`-R` cannot appear as a name). Path is hardcoded to `/usr/bin/pacman`.

### 4) libalpm
- **Usage**: only read-only for inventory, graph, orphans, and the file lists used to build the ownership index. No write operations against the package database.
- **Compatibility note**: `alpm` crate 5.0.2's docs claim libalpm 15.x but the development system runs libalpm 16.0.1; the `checkver` feature is disabled. The build uses pkg-config (clang/bindgen not required). Runtime compatibility is the Arch-container/CI gate.
- **Error handling**: alpm errors are stringified (not leaked as C-pointer-heavy types) and mapped to `CoreError::Alpm`.

## Operational Hardening
- **Socket lifetime**: on startup, `bind()` connects to an existing socket to detect a running daemon; if alive, it refuses to start (exit). If stale (connect fails), it removes the socket before binding.
- **Filesystem modes**: runtime dir 0700, socket 0600, data dir 0700, database 0600, config files written with user-only permissions.
- **Log hygiene**: the minimal logger writes to stderr only (no sensitive values logged by default). Log level controlled by `JETSAM_LOG`.
- **Notifications**: `notify-send` is best-effort and never affects daemon correctness.
- **No setuid binaries**: none in this workspace. Helper is executed via pkexec's elevation mechanism, not installed setuid.

## Known Limitations / Assumptions
- **Single user**: designed for a single desktop user session (XDG runtime/user service).
- **Arch Linux only**: depends on pacman/libalpm, `/proc`, and Arch package naming. Flatpak support is deferred.
- **pkexec policy**: packaging must install a polkit policy that allows the invoking user to run `/usr/bin/jetsam-helper` for the two verbs without an interactive password if desired, or require authentication per policy. The helper itself does not assume passwordless auth.
- **PSS accuracy**: `smaps_rollup` may be unavailable in some containerised environments; the daemon treats missing PSS as `None` and still records launches/uptime.

## Verification Steps (for maintainers)
1. As non-root: `jetsam-helper remove glibc` → stderr "must run as root", exit 2.
2. As non-root via pkexec simulation: expect refusal at validation (root check passes only when real uid 0). With real root: `jetsam-helper remove -R`? but `-R` is rejected as invalid name.
3. `jetsam-helper remove pacman` → `refusing protected package: pacman`, exit 3, JSON has `ok:false`.
4. Start daemon, connect via client: `ping`/`status` return protocol v1.
5. Socket mode: `stat -c "%a" $XDG_RUNTIME_DIR/jetsam/daemon.sock` → `600`.
6. Database mode: `stat -c "%a" $XDG_DATA_HOME/jetsam/usage.db` → `600`.

## Conclusion
This design achieves the stated hard rules: GUI never runs as root, never shells out to pacman, privileged ops only via a strictly validated helper, no shell construction, protected list enforced, versioned IPC, and no telemetry. The implementation matches these rules.
