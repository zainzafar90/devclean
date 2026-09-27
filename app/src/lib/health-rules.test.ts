import { describe, expect, it } from "vitest";
import {
  diskWarning,
  freedBytes,
  isMeasuring,
  memoryWarning,
  isDiskLow,
  isMemoryLow,
  memoryUsedPercent,
  needsConfirmation,
  pressureTone,
  safeAreas,
  totalBytes,
} from "./health-rules";
import type { AreaSize, MemoryInfo } from "./tauri-commands";

const memory = (freePercent: number | null, pressure: MemoryInfo["pressure"] = "normal"): MemoryInfo => ({
  total: 100,
  used: 60,
  available: 40,
  freePercent,
  pressure,
  swapTotal: 0,
  swapUsed: 0,
});

const area = (name: AreaSize["area"], safe: boolean, bytes: number): AreaSize => ({
  area: name,
  label: name,
  safe,
  bytes,
});

describe("health rules", () => {
  it("flags memory under 20% free and disk under 10% free", () => {
    expect(isMemoryLow(memory(19))).toBe(true);
    expect(isMemoryLow(memory(20))).toBe(false);
    expect(isMemoryLow(memory(null))).toBe(false);
    expect(isDiskLow({ total: 100, free: 9, freePercent: 9 })).toBe(true);
    expect(isDiskLow({ total: 100, free: 10, freePercent: 10 })).toBe(false);
  });

  it("maps pressure to a tone", () => {
    expect(pressureTone("critical", false)).toBe("critical");
    expect(pressureTone("warning", false)).toBe("warn");
    expect(pressureTone("normal", true)).toBe("warn");
    expect(pressureTone("normal", false)).toBe("ok");
  });

  it("derives used percent", () => {
    expect(memoryUsedPercent(memory(47))).toBe(53);
    expect(memoryUsedPercent(memory(null))).toBe(60);
  });

  it("asks before cleaning areas that are not safe", () => {
    expect(needsConfirmation(area("worktrees", false, 1))).toBe(true);
    expect(needsConfirmation(area("docker", true, 1))).toBe(false);
  });

  it("sums and filters areas", () => {
    const areas = [area("docker", true, 10), area("playwright", false, 5)];
    expect(safeAreas(areas).map((a) => a.area)).toEqual(["docker"]);
    expect(totalBytes(areas)).toBe(15);
    expect(freedBytes({ area: "docker", before: 10, after: 12, dryRun: false, failures: [] })).toBe(0);
  });

  it("counts sizes as they arrive", () => {
    const partial = [{ bytes: 10 }, { bytes: undefined }];
    expect(totalBytes(partial)).toBe(10);
    expect(isMeasuring(partial)).toBe(true);
    expect(isMeasuring([{ bytes: 0 }])).toBe(false);
  });

  it("words memory and disk warnings, most severe first", () => {
    expect(memoryWarning(memory(10, "critical"))).toBe("Memory pressure is critical");
    expect(memoryWarning(memory(50, "warning"))).toBe("Memory pressure is high");
    expect(memoryWarning(memory(12))).toBe("Only 12% of memory free");
    expect(memoryWarning(memory(47))).toBeNull();
    expect(diskWarning({ total: 100, free: 5, freePercent: 5 })).toBe("Only 5% of disk free");
    expect(diskWarning({ total: 100, free: 50, freePercent: 50 })).toBeNull();
  });
});
