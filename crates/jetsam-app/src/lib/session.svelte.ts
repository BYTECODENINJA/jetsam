import {
  api,
  type AppStatus,
  type CleanupView,
  type InventorySummary,
  type PackageRecord,
  type UsageInfo
} from './api';

export const session = $state({
  status: null as AppStatus | null,
  summary: null as InventorySummary | null,
  packages: null as PackageRecord[] | null,
  usage: {} as Record<string, UsageInfo>,
  ignored: [] as string[],
  cleanup: null as CleanupView | null,
  loading: false,
  error: null as string | null,
  lastSync: null as number | null
});

export function daemonUp(): boolean {
  return !!session.status?.daemon;
}

export function isIgnored(name: string): boolean {
  return session.ignored.includes(name);
}

export async function refreshStatus(): Promise<void> {
  try {
    session.status = await api.appStatus();
    session.error = null;
  } catch (err) {
    session.error = String(err);
  }
}

export async function loadInventory(force = false): Promise<void> {
  if (session.packages && !force) return;
  session.loading = true;
  session.error = null;
  try {
    const [summary, packages] = await Promise.all([
      api.loadInventory(force),
      api.listPackages()
    ]);
    session.summary = summary;
    session.packages = packages;
    session.lastSync = Date.now();
  } catch (err) {
    session.error = String(err);
  } finally {
    session.loading = false;
  }
}

export async function loadUsage(): Promise<void> {
  try {
    session.usage = await api.usageMap();
  } catch {
    session.usage = {};
  }
}

export async function loadIgnored(): Promise<void> {
  try {
    session.ignored = await api.ignoreList();
  } catch {
    session.ignored = [];
  }
}

export async function loadCleanup(): Promise<void> {
  try {
    session.cleanup = await api.cleanupSuggestions();
  } catch (err) {
    session.error = String(err);
  }
}

export async function initialize(): Promise<void> {
  await refreshStatus();
  await Promise.all([loadUsage(), loadIgnored()]);
}

export function usageFor(name: string): UsageInfo | undefined {
  return session.usage[name];
}

export function nowEpoch(): number {
  return Math.floor(Date.now() / 1000);
}

export function daysSince(epoch: number | null | undefined): number | null {
  if (epoch == null) return null;
  return Math.max(0, (Date.now() / 1000 - epoch) / 86400);
}
