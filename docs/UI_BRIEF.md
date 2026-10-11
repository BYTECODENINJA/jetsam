# Jetsam — UI Design Brief (Phase 1)

Companion to `DESIGN.md`. Deliverable here is the *design system* (tokens),
*layout specs*, and the *component list* for every screen, including all states.
No implementation in this phase.

Design intent: **native-quality, calm, honest.** Jetsam is a tool for a decision
you only make occasionally (cleanup) plus a task you do weekly (update). It
should feel like a well-made system utility — closer to GNOME Disks / KDE
Partition Manager than to a dashboard. Numbers are the hero; chrome is quiet.

---

## 1. Design tokens

### Color
Two themes (light/dark) selected from system preference, overridable. Accent is
user-selectable from a small curated palette.

```
--surface-0   page background
--surface-1   cards / panels
--surface-2   raised (modals, hover rows)
--border      hairline separators
--text-hi     primary text        (AA: ≥ 7:1 on surface-0)
--text-lo     secondary text      (AA: ≥ 4.5:1 on surface-0)
--text-dim    disabled / meta
--accent      user accent (focus ring, selection, links)
--accent-fg   text/icon on accent

Semantic (used for the cleanup tiers, never as the only cue):
--safe        #2e7d32-ish   green  — clearly unused, low risk
--caution     #b26a00-ish   amber  — judge needed
--risk        #c62828-ish   red    — reverse deps / borderline
--info        #1565c0-ish   blue   — neutral/update/news
```

Rules: never encode meaning by color alone (pair with icon + label + number);
recompute contrast so both themes pass WCAG AA; accent must hit 3:1 against
surface for focus rings. Respect `prefers-color-scheme` and a manual override.

### Typography (system stack; ship no fonts)
```
family: system-ui, -apple-system, "Inter", "Cantarell", "Noto Sans", sans-serif
mono:   ui-monospace, "JetBrains Mono", "Source Code Pro", monospace

--fs-display  40/44  (hero reclaimable number)
--fs-h1       24/30
--fs-h2       18/24
--fs-body     14/20
--fs-sm       12/16
--fs-meta     11/14
```
Numbers in tables use `font-variant-numeric: tabular-nums`.

### Spacing (4 px base)
`--sp-1 4  --sp-2 8  --sp-3 12  --sp-4 16  --sp-5 24  --sp-6 32  --sp-7 48`

### Radius / elevation / motion
```
--r-sm 6   --r-md 10  --r-lg 14  --r-full 999
--elev-1  0 1px 2px rgba(0,0,0,.12)
--elev-2  0 4px 16px rgba(0,0,0,.18)
--elev-3  0 12px 40px rgba(0,0,0,.28)

--dur-fast 120ms  --dur 180ms  --ease cubic-bezier(.2,.7,.2,1)
@media (prefers-reduced-motion: reduce) { all durations → 0ms }
```

### App shell layout
```
┌─────────────────────────────────────────────────────────────┐
│  top bar: app name · [sync status] · [offline?] · ⌘K · ⚙   │ 56px
├──────────┬──────────────────────────────────────────────────┤
│ sidebar  │  content                                         │
│ 232px    │  (max content width 1180px, centered)            │
│ Dashboard│                                                  │
│ Packages │                                                  │
│ Cleanup  │                                                  │
│ Updates  │                                                  │
│ Settings │                                                  │
└──────────┴──────────────────────────────────────────────────┘
```
Sidebar collapses to icons < 900 px. Full keyboard navigation, focus-visible
rings on every interactive element. Command palette (`Ctrl/Cmd+K`) is global:
jump to screens, search packages, run actions ("Update system", "Open cleanup").

---

## 2. Screen specs

Each screen: purpose · primary action · key components · states.

### 2.1 Dashboard
- **Purpose:** answer "what should I do?" in one look.
- **Primary action:** Update system (if pending) or Review cleanup (if reclaimable).
- **Hero:** reclaimable space (`--fs-display`, tabular), with a one-line
  breakdown "= 412 MB unused + 1.1 GB orphans + 3.2 GB cache".
- **Blocks:** update status card; Arch-news banner (only when unacknowledged
  manual intervention applies); top-5 unused packages (mini rows → detail);
  disk breakdown (packages / cache / orphans / foreign); last sync time.
- **States:** loading skeleton; **empty usage data** ("Usage data: accumulating —
  day 1 of ~14. These numbers are from package metadata only."); offline
  (banner, cached data); error (retry); all-clear ("Nothing to do. Last sync …").

### 2.2 Packages
- **Purpose:** the honest inventory; everything drillable.
- **Primary action:** open a package; bulk-select → add to ignore / detail.
- **Components:** `VirtualTable` (windowed, smooth ≥ 3,000 rows), column
  sort/filter, search (name/desc/repo), saved filters, bulk selection bar.
- **Columns:** name (+ reason icon), version, repo (core/extra/**foreign**),
  size, install date, **last used**, **launches**, **avg PSS**, revdeps.
- **States:** no data; no usage columns populated (dimmed with tooltip "no
  history yet"); filtered-empty; very large set (virtualization proof).

### 2.3 Package detail
- **Purpose:** justify the score and make removal concrete.
- **Primary action:** Remove (opens dry run) or Ignore.
- **Components:** header (name, repo, class badge, protected badge); usage
  timeline (sparkline of launches) + RAM sparkline (avg PSS); dependency view;
  reverse-dependency view; file list (read-only, capped + searchable);
  **"Removing this frees X"** panel showing exclusive orphans; score breakdown
  with the actual per-component numbers (§6 of DESIGN).
- **States:** library package (different copy: "judged by dependents, not
  launches"); protected (Remove disabled + reason); no usage data; unknown owner.

### 2.4 Disk map (v0.2)
- **Purpose:** spatial intuition for what's big and what's stale.
- **Primary action:** click a tile → detail.
- **Components:** treemap by size, tiles colored by recency (fresh → dormant),
  legend, hover tooltip (size, last used), breadcrumb zoom.
- **States:** no size data; huge tree (aggregation); reduced-motion (no zoom
  animation, instant jumps); color-blind safe alternate pattern mapping.

### 2.5 Cleanup (flagship)
- **Purpose:** safe reclamation, trust first.
- **Primary action:** Review selected → **DryRunModal** → confirm (helper).
- **Components:** ranked suggestion list; per row: score, size reclaimable,
  class, days-since-use, revdeps, and an expandable **score breakdown**; a
  visibly separate **"Protected — never suggested"** section with reasons;
  selection with select-all-tier; `DryRunModal` (exact remove list + bytes).
- **States:** no usage data (staleness disabled — explained); nothing reclaimable
  (positive empty state); analysis running; error; dry-run empty (nothing to do).
- **Non-negotiables:** no color-only risk cue; no destructive action without the
  dry run; protected items cannot be selected.

### 2.6 Updates
- **Purpose:** advance safely.
- **Primary action:** **Update system** (single button; the only upgrade path).
- **Components:** pending list (per-package version diff; **no per-row update
  button**); **NewsGate** (unacknowledged manual-intervention notice blocks the
  button until acknowledged); changelog tabs per package (Upstream release ·
  Arch packaging commits · `pacman -Qc`); progress view; **PostUpgradeReport**
  (.pacnew/.pacsave, services needing restart).
- **States:** up to date; sync failed (retry, cached); news present (gate);
  no changelog (explicit "not available"); AUR pending (PKGBUILD diff + delegate
  to paru/yay + explicit confirm); partial-upgrade attempt → refused with
  explanation; upgrade failed (structured error, no half state).
- **Non-negotiable:** exactly one upgrade action; never offer selective updates.

### 2.7 Settings & Privacy
- **Purpose:** control and transparency.
- **Primary action:** toggle tracking / wipe data.
- **Components:** usage-tracking opt-in (with what/where/why copy); ignore list;
  retention; sample interval; **Wipe all usage data**; snapshot integration
  (auto-detect snapper/timeshift/btrfs); notification prefs; theme + accent;
  update/news preferences; helper status.
- **States:** tracking off (explain what's lost, keep metadata features); wipe
  confirm; no snapshot tool (explained); polkit/helper unavailable.

### 2.8 First-run & empty states
- **Purpose:** earn trust before collecting anything.
- **Primary action:** Enable tracking / Continue without tracking.
- **Components:** onboarding stepper: (1) what Jetsam is, (2) exactly what is
  recorded (exe→package, timing, PSS; **explicitly not** window titles, keys,
  file contents, network), (3) where it's stored (path shown), (4) opt-in,
  (5) done + "day one" value recap.
- **States:** no systemd (manual-start instructions); no libalpm version match
  (clear blocking screen with fix guidance); no network.

**Global states (every screen):** loading (skeletons, no spinners if avoidable),
empty, error (message + retry + log path), offline (banner), no-usage-data
(banner + link to onboarding), daemon-down (banner + "Start daemon" action),
partial data (dimmed columns).

---

## 3. Component list

```
Shell:        AppShell, Sidebar, TopBar, SyncStatusChip, OfflineBanner,
              DaemonDownBanner, CommandPalette, ThemeToggle, Toast/ToastStack
Table:        VirtualTable, ColumnHeader, SortMenu, FilterBar, SearchInput,
              BulkSelectionBar, RowMenu, PkgNameCell, ReasonBadge, RepoBadge
Data viz:     ReclaimHero, SizeBadge, UsageSparkline, PssSparkline, Timeline,
              DiskTreemap(v0.2), Legend
Cleanup:      SuggestionList, SuggestionRow, ScoreBreakdown, ScoreBar,
              ProtectedSection, ProtectedBadge, DryRunModal, ReclaimPanel,
              ConfirmRemoveDialog
Updates:      PendingList, VersionDiff, NewsBanner, NewsGate,
              ChangelogTabs, ChangelogMissing, AurPkgbuildDiff,
              UpdateSystemButton, UpgradeProgress, PostUpgradeReport,
              PacnewList, ServiceRestartList
Settings:     OptInCard, IgnoreListEditor, RetentionSelect, SampleIntervalSelect,
              WipeDataDialog, SnapshotIntegrationCard, NotificationPrefs,
              AccentPicker, HelperStatusCard
Onboarding:   OnboardingStepper, PrivacyDisclosure, StoragePathNote,
              DayOneSummary
Primitives:   Button, IconButton, Input, Select, Checkbox, Switch, Badge, Chip,
              Tooltip, Modal/Dialog, Sheet, Tabs, ProgressBar, Skeleton,
              EmptyState, ErrorState, Banner, Icon set
```

---

## 4. Accessibility & platform

- **WCAG AA** contrast in both themes; focus-visible everywhere; everything
  operable by keyboard; logical tab order; `Esc` closes modals; `Ctrl+K` palette.
- **Reduced motion** honored globally; treemap/sparklines degrade to static.
- **Screen readers:** roles/labels on table, score bars expose numeric values
  (e.g. "score 72 of 100 — space 0.8, staleness 1.0, explicit 1.0, safety 0.6").
- **HiDPI & fractional scaling** tested on Hyprland (Wayland), GNOME, KDE; X11
  fallback; native window controls per desktop where possible.
- **i18n-ready** strings from day one; no concatenated sentences; RTL-safe
  layout (no absolute text positioning).

---

## 5. Definition of done for the UI (later phases)

- Cold start < 1 s; package table smooth at 3,000+ rows (virtualized).
- Every screen implements all states above, not just the happy path.
- No color-only signals; all scores explain themselves from data.
- No destructive action without a dry run; protected packages unremovable.
- Exactly one upgrade path; no per-package update control anywhere.
