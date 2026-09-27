const UNITS = ["B", "KB", "MB", "GB", "TB"];

/** Binary units; GB and TB get one decimal, smaller units are whole numbers (same as the CLI). */
export function formatBytes(bytes: number): string {
  let value = Math.max(0, bytes);
  let unit = 0;
  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024;
    unit += 1;
  }
  if (unit >= 3) {
    return `${value.toFixed(1)} ${UNITS[unit]}`;
  }
  return `${Math.floor(value)} ${UNITS[unit]}`;
}
