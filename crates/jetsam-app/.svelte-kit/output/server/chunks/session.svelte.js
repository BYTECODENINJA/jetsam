import "clsx";
import { a as api } from "./api.js";
const session = {
  status: null,
  summary: null,
  packages: null,
  usage: {},
  ignored: [],
  cleanup: null,
  loading: false,
  error: null,
  lastSync: null
};
function isIgnored(name) {
  return session.ignored.includes(name);
}
async function refreshStatus() {
  try {
    session.status = await api.appStatus();
    session.error = null;
  } catch (err) {
    session.error = String(err);
  }
}
async function loadInventory(force = false) {
  if (session.packages && !force) return;
  session.loading = true;
  session.error = null;
  try {
    const [summary, packages] = await Promise.all([api.loadInventory(force), api.listPackages()]);
    session.summary = summary;
    session.packages = packages;
    session.lastSync = Date.now();
  } catch (err) {
    session.error = String(err);
  } finally {
    session.loading = false;
  }
}
async function loadUsage() {
  try {
    session.usage = await api.usageMap();
  } catch {
    session.usage = {};
  }
}
async function loadIgnored() {
  try {
    session.ignored = await api.ignoreList();
  } catch {
    session.ignored = [];
  }
}
async function loadCleanup() {
  try {
    session.cleanup = await api.cleanupSuggestions();
  } catch (err) {
    session.error = String(err);
  }
}
function usageFor(name) {
  return session.usage[name];
}
function daysSince(epoch) {
  if (epoch == null) return null;
  return Math.max(0, (Date.now() / 1e3 - epoch) / 86400);
}
export {
  loadCleanup as a,
  loadIgnored as b,
  loadUsage as c,
  daysSince as d,
  isIgnored as i,
  loadInventory as l,
  refreshStatus as r,
  session as s,
  usageFor as u
};
