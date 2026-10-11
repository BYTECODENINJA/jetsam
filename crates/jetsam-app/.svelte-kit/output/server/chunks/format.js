function formatBytes(bytes) {
  if (bytes == null) return "—";
  const value = Math.max(0, bytes);
  if (value < 1024) return `${value} B`;
  const units = ["KiB", "MiB", "GiB", "TiB", "PiB"];
  let scaled = value / 1024;
  let unit = 0;
  while (scaled >= 1024 && unit < units.length - 1) {
    scaled /= 1024;
    unit += 1;
  }
  const digits = scaled >= 100 ? 0 : scaled >= 10 ? 1 : 2;
  return `${scaled.toFixed(digits)} ${units[unit]}`;
}
function formatDate(epoch) {
  if (epoch == null) return "—";
  return new Date(epoch * 1e3).toLocaleDateString();
}
function formatDays(days) {
  if (days == null) return "no data";
  if (days < 1) return "today";
  if (days < 2) return "yesterday";
  if (days < 30) return `${Math.round(days)} days ago`;
  const months = days / 30.44;
  if (months < 24) return `${Math.round(months)} months ago`;
  return `${(months / 12).toFixed(1)} years ago`;
}
export {
  formatDays as a,
  formatDate as b,
  formatBytes as f
};
