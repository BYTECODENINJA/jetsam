import { invoke } from "@tauri-apps/api/core";
const api = {
  loadInventory: (force = false) => invoke("load_inventory", { force }),
  listPackages: () => invoke("list_packages"),
  packageDetail: (name) => invoke("package_detail", { name }),
  cleanupSuggestions: () => invoke("cleanup_suggestions"),
  usageMap: () => invoke("usage_map"),
  ignoreList: () => invoke("ignore_list"),
  setIgnored: (name, ignored) => invoke("set_ignored", { name, ignored }),
  setOptIn: (enabled) => invoke("set_opt_in", { enabled }),
  wipeData: () => invoke("wipe_data"),
  appStatus: () => invoke("app_status"),
  dryRun: (names) => invoke("dry_run", { names }),
  applyRemove: (names) => invoke("apply_remove", { names }),
  runUpgrade: () => invoke("run_upgrade"),
  checkUpdates: () => invoke("check_updates"),
  startDaemon: () => invoke("start_daemon"),
  helperAvailable: () => invoke("helper_available")
};
function repoLabel(repo) {
  if (typeof repo === "string") return repo;
  return repo.other;
}
export {
  api as a,
  repoLabel as r
};
