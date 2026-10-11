# Jetsam — Design Document (Phase 1)

> **Pitch:** A usage-aware package companion for Arch Linux. See what you actually
> use, reclaim disk space safely, and update without breaking your system.
>
> **This is not** a system monitor, and **not** a pacman front-end clone. The
> unique value is *long-term per-package usage history* combined with *safe
> cleanup* and *safe update guidance*.

Status: Phase 1 (design + brief). No code delivered in this phase. Author worked
from the live host: rustc 1.98.1, pacman 7.1.0, libalpm 15.x, `alpm` crate 5.0.2.

---

## 0. Name

Three candidates were screened against crates.io, the AUR RPC, and the GitHub
repo namespace (`github.com/<name>/<name>`). All three below are free on all
three surfaces at the time of writing.

| Name | Meaning | crates.io | AUR | GitHub repo ns | Verdict |
|---|---|---|---|---|---|
| **jetsam** | cargo thrown overboard to keep a ship afloat; *to jettison* | free | free | free | **chosen** |
| fallow | land left unworked to rest — the *state* of unused packages | free | free | free | runner-up |
| larder | a pantry of provisions — an *inventory*, not usage | free | free | free | rejected |

**Choice: `jetsam`.** It names the *action* the product is built around —
safely shedding dead weight without sinking the ship (a full, consistent
system). It is short, unambiguous, phonetically boring (good for a CLI verb:
`jetsam scan`, `jetsam clean`), and gives the project a coherent identity:
the disk map is a cargo hold, the cleanup score is a manifest of what to toss.
`fallow` describes the problem but is passive; `larder` describes inventory, not
usage. Binary/command: `jetsam`; daemon: `jetsamd`; helper: `jetsam-helper`.
GitHub host: an org namespace (e.g. `jetsam-dev/jetsam`) since the bare
`jetsam` user handle is taken (dormant).

---

## 1. Positioning — confirmed and sharpened

Confirmed as given. Sharpened into three pillars, in priority order:

1. **Observe** — build durable per-package usage history (launches, uptime,
   last-used, average PSS) that Linux does not record natively.
2. **Reclaim** — turn that history plus real dependency analysis into a
   *transparent, ranked, reversible-by-inspection* cleanup advisor.
3. **Advance** — update safely: one full-system upgrade, Arch-news gate,
   multi-source changelogs, post-upgrade report.

The moat is pillar 1: competitors can copy a package table overnight, but not
weeks of local usage data. Everything else is designed to be *honest* (no
fabricated history, no false precision) and *safe* (never break the system).

---

## 2. Assumptions (explicit; if wrong, the doc says where it breaks)

1. **Host:** Arch, EndeavourOS, CachyOS, or a Manjaro-adjacent system with
   `pacman` ≥ 6. **Verified on the development host:** `alpm` 5.0.2 (crate docs
   claim libalpm 15.x) links and runs correctly against **libalpm 16.0.1**
   (pacman 7.1). We therefore do **not** enable the crate's `checkver` feature
   (it would reject 16.x) and instead verify at runtime with a clear error; the
   CI Arch container is the compatibility gate. libalpm is discovered via
   `pkg-config` (needs the `pacman` package's headers; `clang` is only needed if
   the `generate` feature were used — we use the shipped bindings).
2. **systemd present.** The collector is a *user* service (no root). If systemd
   is absent we fall back to a documented `jetsamd` invocation; no auto-start.
3. **Single interactive user.** v0.1 tracks the *owning user's* processes only.
   Other users' and root's processes are invisible and reported as such, never
   guessed.
4. **English-first, i18n-ready.** All UI strings go through an i18n layer from
   day one; only `en` ships.
5. **Opt-in collector.** Usage tracking is off until the user consents on first
   run. The app is fully useful without it (sizes, orphans, dates, updates).
6. **polkit is available** for the privileged helper.
7. **X11 or Wayland**, HiDPI/fractional scaling both in scope; we test on
   Hyprland (Wayland) and GNOME/KDE.
8. **`/proc` is readable for own-user processes** (`smaps_rollup` present).
9. **Network is optional.** Every network feature degrades to cached/empty.
10. **Packages are pacman-managed.** Flatpak/AppImage are out of v0.1 (backend
    trait reserved, §5).
11. **Repo→upstream mapping is best-effort.** Many Arch packages have no
    resolvable changelog; the UI says so rather than inventing one.
12. **The user can read a dry run.** We never hide what a removal deletes.

---

## 3. Hard truths → design decisions

| Hard truth | Decision |
|---|---|
| Linux records no per-app history | Background `jetsamd` user service samples `/proc`; SQLite with daily rollups. v0.1 is useful on day one via sizes/orphans/dates/reclaimable, and the UI clearly shows "usage data: accumulating — N days". |
| A package ≠ an application | Each package is classified (`desktop-app`, `cli-tool`, `library`, `daemon`, `kernel/driver`, `data`). Only desktop-entry apps and PATH executables count as "used"; libraries are judged by reverse-dependents, daemons by uptime, not launches. |
| No partial upgrades | The app never exposes a per-package "update" button. Updates are *listed* per package but applied by a single `pacman -Syu`-equivalent through the helper. Any attempted selective update is refused with an explanation. |
| Repos lag; changelogs missing | Changelog data from multiple sources with a confidence label; graceful "no changelog available". Never blocks the update. |
| Tracking usage is sensitive | Local-only, opt-in, transparent, per-package ignore list, one-click wipe, collector has **zero** network capability. |

---

## 4. Competitive analysis — exactly what each lacks

| Tool | What it is | What it lacks (relevant to us) |
|---|---|---|
| **Octopi** | Qt pacman GUI + update notifier | No usage history; no usage-aware cleanup; no per-package score; no PSS/RAM tracking; packages ≠ apps; AUR UX dated. |
| **pamac** | GTK/tray manager (Manjaro) | Manjaro-centric; has historically offered *selective* updates (unsafe on Arch); no usage history; no size/usage scoring; bundled AUR building we explicitly avoid. |
| **Bauh** | Multi-backend (pacman/AUR/flatpak/snap/appimage) | Broad but shallow; no usage history; no cleanup advisor; no changelog aggregation; no Arch-news gate; inconsistent UX. |
| **arch-update** | CLI/tray notifier + timer | Only updates + news. No usage tracking, no cleanup, no dependency reclaim analysis. *Complementary* — we could integrate its news source. |
| **btop** | Live system/process monitor | Live only; no package mapping; no persistence; no cleanup; no updates. Different category entirely. |
| **Mission Center** | GNOME-style resource monitor | Live CPU/GPU/RAM/disk/net; no package awareness, no per-package history, no cleanup/updates. |
| **Resources** | GTK4/Rust resource monitor | Live process/system view; no package mapping, no history, no cleanup/updates. |
| *(pacman-contrib: paccache, pacgraph)* | CLI utilities | Cache pruning and size graphs; no usage, no GUI, no advisor. We surface their *ideas*, not their UX. |

**White space:** only Jetsam links *persistent per-package usage history* to
*reclaimable-space scoring* and *safe update guidance* in one GUI-first,
local-only tool. Monitors show the present; pacman front-ends show the catalog;
nobody shows the *relationship between what you use and what you carry*.

---

## 5. Architecture

### Stack
- **Tauri v2** (Rust backend + system webview). Backend owns all logic.
- **Front end: Svelte 5.** Justification: compile-time reactivity with no
  virtual-DOM runtime → smallest bundle and fastest cold start (target < 1 s)
  for a thin client; less runtime churn than React; smaller webview attack
  surface. A 3,000-row table is handled by a windowed virtual list, not the
  framework. (React would be chosen only if we needed its ecosystem; we don't.)

### Workspace crates

```
jetsam/
  crates/
    jetsam-core/     # alpm access, ownership index, classification, scoring, types
    jetsam-daemon/   # systemd user service: sampling, SQLite, rollups, notify, news
    jetsam-helper/   # polkit-gated privileged binary, fixed allowlist
    jetsam-app/      # Tauri v2 GUI (Rust side) + src-tauri
    jetsam-cli/      # scripting + Waybar/Polybar/i3blocks module; --json
  ui/                # Svelte 5 front end (thin client)
```

### Process model

```
        ┌──────────────┐   UDS JSON  ┌───────────────┐
        │  jetsam-app  │◀───────────▶│               │
        │  (GUI, user) │  $XDG_RUN   │  jetsamd      │  systemd --user
        └──────┬───────┘             │  (user svc)   │  always on
               │ Tauri cmd           │  SQLite+sampler│
        ┌──────▼───────┐             └───────┬───────┘
        │  jetsam-cli  │◀───────────────────▶┘
        │  (user)      │   same UDS protocol
        └──────┬───────┘
               │ D-Bus → polkit
        ┌──────▼────────────┐   fixed allowlist: remove_pkgs, full_upgrade
        │ jetsam-helper     │   (root, no shell, validated argv only)
        └───────────────────┘
```

- The GUI window is optional and short-lived; closing it leaves only `jetsamd`.
- The GUI and CLI never run as root and never shell out to `pacman`; privileged
  operations go only through `jetsam-helper` via polkit.
- Daemon ↔ clients speak a **versioned** protocol (`{"v":1,...}`) over a Unix
  domain socket in `$XDG_RUNTIME_DIR/jetsam/`; the wire format is also the
  `--json` contract for scripting.

### Data flow (usage)
1. `jetsamd` samples `/proc` at a configurable interval (default 5 s).
2. New *root* processes only are counted as launch events (child forks are
   attributed to the parent root, not counted again).
3. `exe` path is canonicalized (symlinks, shebangs → interpreter) and mapped to
   a package via a **file-ownership index** built from libalpm file lists and
   cached on disk (rebuilt when the local DB mtime changes).
4. PSS from `/proc/<pid>/smaps_rollup` accumulates per package; averaged over
   active samples.
5. Raw samples are rolled up daily; retention configurable; all in SQLite.

### Data model (SQLite, indicative)

```sql
package(id, name, version, repo, install_reason, install_date, size,
        description, class, protected)
usage_launch(pkg_id, ts, root_pid)          -- launch events
usage_daily(pkg_id, day, launches, uptime_s, pss_samples, pss_sum)  -- rollups
proc_sample(ts, pkg_id, pid, pss_kb, state) -- short-lived raw (pruned fast)
ignore(pkg_id, scope)                        -- per-package opt-out
snapshot(day, total_size, reclaimable, pkgs) -- for trends
meta(key, value)                             -- schema_version, optin, retention
```

### Front-end/back-end boundary
The webview receives only normalized view-models over Tauri commands. No alpm,
SQLite, network, or filesystem logic in TypeScript. All fetched content (news,
release notes, PKGBUILDs) is sanitized server-side before it crosses the bridge.

---

## 6. The cleanup score (transparent by construction)

For a candidate package `p`:

```
reclaim(p) = size(p) + Σ size(d)   for d in exclusive_orphans(p)
```
where `exclusive_orphans(p)` are dependencies that become orphans **iff** `p`
is removed (computed with libalpm's dependency graph, same semantics pacman
uses). Components are each normalized to `0..1` and shown per item in the UI:

```
space      = min(1, log1p(reclaim) / log1p(SPACE_CAP))
staleness  = min(1, days_since_last_use / STALE_CAP)
explicit   = 1.0 if explicit & unused   else 0.5
safety     = 1 - min(1, reverse_dep_count / REV_CAP)

score = 100·( w_s·space + w_t·staleness + w_e·explicit + w_s2·safety )
defaults: w_s=0.30  w_t=0.30  w_e=0.20  w_s2=0.20
```

- **Protected set** (base, `linux*`, bootloader, drivers/firmware, init,
  `filesystem`, `glibc`, pacman itself, anything on the critical path) can
  *never* be suggested; shown as visibly excluded with the reason.
- Every suggestion opens a **dry run** listing exactly the packages (and
  orphaned deps) that would be removed and bytes freed, before the helper is
  ever invoked.
- If usage data is absent for a package, `staleness` is `null`, not faked; the
  UI labels it "unknown" and it contributes via an explicit user setting (off
  by default), so a fresh install never mass-suggests everything.

---

## 7. Privacy & threat model

**Assets:** per-package usage history (behavioural), package inventory, user
identity, and — at the privileged boundary — the ability to alter the system.

**Trust boundaries:** (a) unprivileged user processes ↔ daemon; (b) unprivileged
↔ root helper via polkit; (c) Jetsam ↔ network (fetch only, untrusted input);
(d) backend ↔ webview (untrusted rendered content).

| Threat | Mitigation |
|---|---|
| Collector exfiltrates browsing/behaviour | Collector has **no** network code path; static + test assertion that `jetsamd` has no network deps. Local DB only. |
| GUI compromise → root | GUI never root, never shells out; only the helper is privileged, with a fixed allowlist and argv validation. |
| Malicious package name / argument injection | Package names/args validated against libalpm identifiers; **no string shell construction anywhere**; helper takes a validated list, not a command string. |
| Malicious news / release notes / PKGBUILD | Treated as untrusted: fetched in backend against a domain allowlist, sanitized (no HTML/JS) before rendering; PKGBUILDs shown as inert monospace text. |
| Webview supply-chain / remote content | Strict Tauri capabilities + CSP; no remote content loaded; only local assets. |
| Network MITM | HTTPS + pinned domain allowlist; no user-controlled URLs. |
| "Wipe data" not real | Wipe removes the SQLite DB and socket state; documented and tested. |

**Explicit non-goals for the collector:** no window titles, no keystrokes, no
network, no file contents. It records `exe → package` and timing only.

---

## 8. Top 5 risks (technical + product) and mitigations

1. **`exe ↔ package` mapping accuracy.** Shared binaries, symlink chains,
   interpreters (python/node/sh), scripts, AppImages, Flatpaks all break naive
   mapping. *Mitigation:* ownership index from libalpm file lists + path
   canonicalization + interpreter/shebang attribution + an explicit `unknown`
   bucket; the UI always shows *why* a package is credited; mapping tests
   against a fixture DB.
2. **Usage-signal quality.** Child processes, daemons, and terminal tools skew
   naive counts. *Mitigation:* count only *new root* processes as launches;
   daemons contribute uptime not launches; dedupe; survival of long-lived
   processes; clearly separate "launched" from "running".
3. **Privacy/trust backlash** ("why is a package tool watching my processes?").
   *Mitigation:* opt-in, local-only, transparent onboarding, per-package ignore,
   wipe, source-available, "no telemetry, ever" in README, collector demonstrably
   network-free.
4. **Arch safety:** mass-removal breakage and partial-upgrade temptation.
   *Mitigation:* full-system upgrades only (refuse selective), protected list,
   libalpm dependency checks before removal, mandatory dry run, offer a snapshot
   (v0.2), never auto-remove.
5. **Update/changelog data reliability:** repo lag, missing changelogs, GH
   rate limits, mapping failures. *Mitigation:* multi-source with confidence
   labels, aggressive caching + ETag/backoff, graceful "no changelog", sync into
   a temp DB copy so the real sync DB is never partially updated.

Product risks also tracked: empty-usage first impression (mitigated by
day-one value), scope creep into "generic monitor" (hard non-goals), Tauri
`webkit2gtk-4.1` runtime weight and Wayland fractional-scaling bugs (test
matrix), and distro fragmentation (neutral, `pacman.conf`-driven, no Manjaro
assumptions).

---

## 9. Licensing

**Project license: `GPL-3.0-or-later`.**

Reasoning: the `alpm` binding crate is **GPL-3.0** (verified; co-owned by Arch
Linux Technical User) and links `libalpm`/`pacman`, which are
**GPL-2.0-or-later**. Linking them produces a combined work that must be
GPL-3.0-or-later (GPL-2.0-or-later is GPL-3.0-compatible). This also matches
the ecosystem (`paru`, `yay` are GPL-3.0), so AUR users see nothing surprising.
Tauri (MIT/Apache-2.0) and front-end assets remain compatible as part of a
larger GPL work. Every dependency's license is checked in CI; anything
incompatible blocks the merge.

---

## 10. Engineering standards

- **MSRV:** `1.85.0` (required transitively by `alpm` 5.x, which uses edition
  2024; Jetsam's own crates use edition 2021 for stability). Tauri v2 needs
  ≥ 1.77. Pinned in `rust-toolchain.toml` and checked in CI.
- `cargo fmt` + `cargo clippy -- -D warnings` clean; **no `unwrap` in
  non-test code**; `thiserror` in libraries, `anyhow` at binary edges.
- Unit tests: scoring, parsing, ownership mapping, classification. Integration:
  fixture pacman DB. Front-end component tests: cleanup review + update gate.
- Config in TOML under XDG paths, zero-config first run, sane defaults.
- Graceful degradation everywhere: no usage data, no network, no snapshot tool,
  unreadable processes.
- SemVer; `CHANGELOG.md` in Keep a Changelog format.
- CI: build, test (per-OS matrix), lint, license check, release artifacts.

---

## 11. Roadmap

- **v0.1 (MVP):** inventory (libalpm), usage collector, cleanup advisor + score,
  orphans/cache/reclaimable, helper-based removal, update center (news gate,
  changelog tabs, full upgrade, post-upgrade report), settings/privacy, CLI/`--json`.
- **v0.2:** arch-audit vuln flags; snapshot-before-upgrade + cache downgrade;
  interactive disk map; pacman-log timeline; Waybar/Polybar/i3blocks module +
  notifications; pacman hook for instant install/remove events.
- **v0.3:** `PackageBackend` trait + Flatpak backend; richer trends; community
  cleanup-score tuning; translations.

Phases (working rule): (1) this doc + UI brief → (2) workspace + `core` →
(3) daemon + helper + security review → (4) Tauri app screens → (5) update
center → (6) CLI/CI/packaging/README/launch. Stop after each for review.
