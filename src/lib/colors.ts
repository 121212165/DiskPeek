export const TYPE_COLORS: Record<string, string> = {
  ".exe": "#e74c3c",
  ".dll": "#e74c3c",
  ".sys": "#c0392b",
  ".zip": "#f39c12",
  ".rar": "#f39c12",
  ".7z": "#f39c12",
  ".tar": "#f39c12",
  ".gz": "#f39c12",
  ".png": "#2ecc71",
  ".jpg": "#2ecc71",
  ".jpeg": "#2ecc71",
  ".gif": "#2ecc71",
  ".bmp": "#2ecc71",
  ".svg": "#2ecc71",
  ".mp4": "#3498db",
  ".avi": "#3498db",
  ".mkv": "#3498db",
  ".mov": "#3498db",
  ".mp3": "#9b59b6",
  ".wav": "#9b59b6",
  ".flac": "#9b59b6",
  ".pdf": "#e67e22",
  ".doc": "#2980b9",
  ".docx": "#2980b9",
  ".xls": "#27ae60",
  ".xlsx": "#27ae60",
  ".ppt": "#d35400",
  ".pptx": "#d35400",
  ".html": "#1abc9c",
  ".css": "#1abc9c",
  ".js": "#f1c40f",
  ".ts": "#2c3e50",
  ".json": "#7f8c8d",
  ".xml": "#7f8c8d",
  ".txt": "#95a5a6",
  ".md": "#95a5a6",
};

export function getNodeColor(name: string, isDir: boolean): string {
  if (isDir) return "#5b6abf";
  const ext = name.slice(name.lastIndexOf(".")).toLowerCase();
  return TYPE_COLORS[ext] || "#7f8c8d";
}

export function formatSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  return (bytes / Math.pow(1024, i)).toFixed(i === 0 ? 0 : 1) + " " + units[i];
}