/**
 * Format byte size to human-readable string.
 * Auto-scales through B / KB / MB / GB / TB, 1 decimal place.
 */
export function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B';

  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const k = 1024;
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(k)), units.length - 1);
  const value = bytes / Math.pow(k, i);

  return `${value.toFixed(1)} ${units[i]}`;
}

/**
 * Format a Unix timestamp (seconds or milliseconds) to "YYYY-MM-DD HH:mm:ss".
 */
export function formatTime(timestamp: number): string {
  // Rust may send seconds; JS expects milliseconds.
  const ms = timestamp < 1e12 ? timestamp * 1000 : timestamp;
  const d = new Date(ms);

  const pad = (n: number) => String(n).padStart(2, '0');
  const Y = d.getFullYear();
  const M = pad(d.getMonth() + 1);
  const D = pad(d.getDate());
  const h = pad(d.getHours());
  const m = pad(d.getMinutes());
  const s = pad(d.getSeconds());

  return `${Y}-${M}-${D} ${h}:${m}:${s}`;
}

/**
 * Format a Unix timestamp (seconds or milliseconds) to a relative time string
 * such as "3 分钟前", "2 小时前", "3 天前".
 */
export function formatRelativeTime(timestamp: number): string {
  const ms = timestamp < 1e12 ? timestamp * 1000 : timestamp;
  const now = Date.now();
  const diff = now - ms;

  if (diff < 0) return '刚刚';

  const seconds = Math.floor(diff / 1000);
  if (seconds < 60) return '刚刚';

  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes} 分钟前`;

  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours} 小时前`;

  const days = Math.floor(hours / 24);
  if (days < 30) return `${days} 天前`;

  const months = Math.floor(days / 30);
  if (months < 12) return `${months} 个月前`;

  return `${Math.floor(months / 12)} 年前`;
}

/**
 * Return an emoji icon for the given file type string.
 */
export function getFileTypeIcon(fileType: string | null): string {
  switch (fileType) {
    case 'VIDEO':
      return '🎬';
    case 'IMAGE':
      return '🖼';
    case 'DOCUMENT':
      return '📄';
    case 'ARCHIVE':
      return '📦';
    case 'OTHER':
      return '📁';
    default:
      return '📄';
  }
}

/**
 * Return the percentage string given a part and total.
 */
export function formatPercent(part: number, total: number): string {
  if (total === 0) return '0.0%';
  return `${((part / total) * 100).toFixed(1)}%`;
}