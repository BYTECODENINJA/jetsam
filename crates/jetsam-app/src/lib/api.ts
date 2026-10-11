import { invoke } from '@tauri-apps/api/core';

export type RepoKind = string | { other: string };

export interface PackageRecord {
  name: string;
  version: string;
  repo: RepoKind;
  reason: 'explicit' | 'dependency';
  install_date: number | null;
  installed_size: number;
  package_size: number;
  description: string | null;
  url: string | null;
  licenses: string[];
  groups: string[];
  provides: string[];
  depends: string[];
  class: string;
  protected: boolean;
  protected_reason: string | null;
  reverse_dep_count: number;
  file_count: number;
}

export interface PackageDetail {
  record: PackageRecord;
  files: string[];
  required_by: string[];
}

export interface UsageInfo {
  name: string;
  launches: number;
  uptime_secs: number;
  last_used_epoch: number | null;
  first_seen_epoch: number | null;
  pss_samples: number;
  avg_pss_bytes: number | null;
}

export interface DaemonStatus {
  daemon_version: string;
  protocol_version: number;
  opted_in: boolean;
  sampling_interval_secs: number;
  pss_interval_secs: number;
  packages_observed: number;
  ignored_count: number;
  first_sample_epoch: number | null;
  last_sample_epoch: number | null;
  days_of_history: number;
  started_at_epoch: number;
  uptime_secs: number;
}

export interface AppStatus {
  daemon: DaemonStatus | null;
  packages_loaded: boolean;
  packages: number;
  helper_available: boolean;
  helper_path: string | null;
}

export interface InventorySummary {
  packages: number;
  orphans: number;
  protected: number;
  total_bytes: number;
  orphan_bytes: number;
  foreign_bytes: number;
  build_ms: number;
}

export interface ScoreComponents {
  space: number;
  staleness: number | null;
  explicit: number;
  safety: number;
}

export interface Candidate {
  name: string;
  class: string | null;
  explicit: boolean;
  reverse_dep_count: number;
  days_since_last_use: number | null;
  reclaimable_bytes: number;
  exclusive_orphans: string[];
  breaks: [string, string][];
  score: { total: number; components: ScoreComponents };
}

export interface ProtectedItem {
  name: string;
  reason: string;
}

export interface CleanupView {
  candidates: Candidate[];
  protected: ProtectedItem[];
  usage_known: boolean;
}

export interface RemovalPlan {
  remove: string[];
  exclusive_orphans: string[];
  freed_bytes: number;
  breaks: [string, string][];
}

export interface HelperOutcome {
  action: string;
  ok: boolean;
  exit_code: number | null;
  stdout: string;
  stderr: string;
}

export interface PendingUpdate {
  name: string;
  installed: string;
  available: string;
}

export const api = {
  loadInventory: (force = false) => invoke<InventorySummary>('load_inventory', { force }),
  listPackages: () => invoke<PackageRecord[]>('list_packages'),
  packageDetail: (name: string) => invoke<PackageDetail>('package_detail', { name }),
  cleanupSuggestions: () => invoke<CleanupView>('cleanup_suggestions'),
  usageMap: () => invoke<Record<string, UsageInfo>>('usage_map'),
  ignoreList: () => invoke<string[]>('ignore_list'),
  setIgnored: (name: string, ignored: boolean) => invoke<void>('set_ignored', { name, ignored }),
  setOptIn: (enabled: boolean) => invoke<void>('set_opt_in', { enabled }),
  wipeData: () => invoke<void>('wipe_data'),
  appStatus: () => invoke<AppStatus>('app_status'),
  dryRun: (names: string[]) => invoke<RemovalPlan>('dry_run', { names }),
  applyRemove: (names: string[]) => invoke<HelperOutcome>('apply_remove', { names }),
  runUpgrade: () => invoke<HelperOutcome>('run_upgrade'),
  checkUpdates: () => invoke<PendingUpdate[]>('check_updates'),
  startDaemon: () => invoke<boolean>('start_daemon'),
  helperAvailable: () => invoke<boolean>('helper_available')
};

export function repoLabel(repo: RepoKind): string {
  if (typeof repo === 'string') return repo;
  return repo.other;
}
