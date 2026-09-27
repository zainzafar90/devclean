import type { AreaInfo, CleanOutcome, DiskInfo, MemoryInfo, Pressure } from "./tauri-commands";

export type Tone = "ok" | "warn" | "critical";

export const LOW_MEMORY_PERCENT = 20;
export const LOW_DISK_PERCENT = 10;

export function isMemoryLow(memory: MemoryInfo): boolean {
  return memory.freePercent !== null && memory.freePercent < LOW_MEMORY_PERCENT;
}

export function isDiskLow(disk: DiskInfo): boolean {
  return disk.freePercent < LOW_DISK_PERCENT;
}

export function pressureTone(pressure: Pressure, memoryLow: boolean): Tone {
  if (pressure === "critical") {
    return "critical";
  }
  if (pressure === "warning" || memoryLow) {
    return "warn";
  }
  return "ok";
}

/** Share of memory in use as macOS counts it: 100 minus the free percentage. */
export function memoryUsedPercent(memory: MemoryInfo): number {
  if (memory.freePercent === null) {
    return memory.total > 0 ? Math.round((memory.used / memory.total) * 100) : 0;
  }
  return 100 - memory.freePercent;
}

/** The inline warning under the memory row, or null when memory is fine. */
export function memoryWarning(memory: MemoryInfo): string | null {
  if (memory.pressure === "critical") {
    return "Memory pressure is critical";
  }
  if (memory.pressure === "warning") {
    return "Memory pressure is high";
  }
  if (isMemoryLow(memory)) {
    return `Only ${memory.freePercent}% of memory free`;
  }
  return null;
}

export function diskWarning(disk: DiskInfo): string | null {
  return isDiskLow(disk) ? `Only ${disk.freePercent}% of disk free` : null;
}

export function needsConfirmation(area: AreaInfo): boolean {
  return !area.safe;
}

export function freedBytes(outcome: CleanOutcome): number {
  return Math.max(0, outcome.before - outcome.after);
}

export function safeAreas<T extends AreaInfo>(areas: T[]): T[] {
  return areas.filter((area) => area.safe);
}

/** Sum of the sizes measured so far; areas still measuring count as zero. */
export function totalBytes(areas: Array<{ bytes: number | undefined }>): number {
  return areas.reduce((sum, area) => sum + (area.bytes ?? 0), 0);
}

export function isMeasuring(areas: Array<{ bytes: number | undefined }>): boolean {
  return areas.some((area) => area.bytes === undefined);
}
