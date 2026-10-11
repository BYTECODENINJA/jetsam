export function formatBytes(bytes: number | null | undefined): string {
  if (bytes == null) return '—';
  const value = Math.max(0, bytes);
  if (value < 1024) return `${value} B`;
  const units = ['KiB', 'MiB', 'GiB', 'TiB', 'PiB'];
  let scaled = value / 1024;
  let unit = 0;
  while (scaled >= 1024 && unit < units.length - 1) {
    scaled /= 1024;
    unit += 1;
  }
  const digits = scaled >= 100 ? 0 : scaled >= 10 ? 1 : 2;
  return `${scaled.toFixed(digits)} ${units[unit]}`;
}

export function formatDuration(seconds: number | null | undefined): string {
  if (seconds == null) return '—';
  const s = Math.max(0, Math.floor(seconds));
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m`;
  const h = Math.floor(m / 60);
  if (h < 48) return `${h}h ${m % 60}m`;
  const d = Math.floor(h / 24);
  return `${d}d ${h % 24}h`;
}

export function formatDate(epoch: number | null | undefined): string {
  if (epoch == null) return '—';
  return new Date(epoch * 1000).toLocaleDateString();
}

export function formatDays(days: number | null | undefined): string {
  if (days == null) return 'no data';
  if (days < 1) return 'today';
  if (days < 2) return 'yesterday';
  if (days < 30) return `${Math.round(days)} days ago`;
  const months = days / 30.44;
  if (months < 24) return `${Math.round(months)} months ago`;
  return `${(months / 12).toFixed(1)} years ago`;
}

export function shortEpoch(epoch: number | null | undefined): string {
  if (epoch == null) return 'never';
  return new Date(epoch * 1000).toLocaleString();
}
