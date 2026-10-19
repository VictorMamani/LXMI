const MB = 1024 * 1024;

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < MB) return `${Math.round(bytes / 1024)} KB`;
  return `${(bytes / MB).toLocaleString("es", { maximumFractionDigits: bytes < 10 * MB ? 1 : 0 })} MB`;
}

export function plural(
  count: number,
  singular: string,
  pluralForm = `${singular}s`,
) {
  return `${count} ${count === 1 ? singular : pluralForm}`;
}

const relative = new Intl.RelativeTimeFormat("es", { numeric: "auto" });

/** "hace 2 h", "hace 3 días"… */
export function timeAgo(timestamp: number, now = Date.now()): string {
  const minutes = Math.round((timestamp - now) / 60000);
  if (Math.abs(minutes) < 60) return relative.format(minutes, "minute");
  const hours = Math.round(minutes / 60);
  if (Math.abs(hours) < 24) return relative.format(hours, "hour");
  return relative.format(Math.round(hours / 24), "day");
}

/** Two-digit counter used by the launch progress ("03 / 06"). */
export function pad2(value: number): string {
  return String(value).padStart(2, "0");
}
