import { invoke } from "@tauri-apps/api/core";

export type Pressure = "normal" | "warning" | "critical" | "unknown";

export interface MemoryInfo {
  total: number;
  used: number;
  available: number;
  freePercent: number | null;
  pressure: Pressure;
  swapTotal: number;
  swapUsed: number;
}

export interface ProcessUsage {
  name: string;
  bytes: number;
}

export interface DiskInfo {
  total: number;
  free: number;
  freePercent: number;
}

export interface Snapshot {
  memory: MemoryInfo;
  topApps: ProcessUsage[];
  disk: DiskInfo;
}

export type AreaName =
  | "docker"
  | "pnpm"
  | "uv"
  | "npm"
  | "yarn"
  | "brew"
  | "pip"
  | "xcode"
  | "simulators"
  | "playwright"
  | "cocoapods"
  | "worktrees";

export interface AreaInfo {
  area: AreaName;
  label: string;
  safe: boolean;
}

export interface AreaSize extends AreaInfo {
  bytes: number;
}

export interface CleanOutcome {
  area: AreaName;
  before: number;
  after: number;
  dryRun: boolean;
  failures: string[];
}

export interface CleanProgress {
  area: AreaName;
  line: string;
}

export interface LastRun {
  started: string;
  finished: string | null;
  freed: string | null;
}

export interface ScheduleStatus {
  enabled: boolean;
  logFile: string;
  recentLines: string[];
  lastRun: LastRun | null;
}

export interface AppInfo {
  name: string;
  version: string;
  author: string;
  repo: string;
}

export const commands = {
  snapshot: () => invoke<Snapshot>("snapshot"),
  areaSizes: () => invoke<AreaSize[]>("area_sizes"),
  areas: () => invoke<AreaInfo[]>("areas"),
  areaSize: (area: AreaName) => invoke<number>("area_size", { area }),
  clean: (areas: AreaName[]) => invoke<CleanOutcome[]>("clean", { areas }),
  scheduleStatus: () => invoke<ScheduleStatus>("schedule_status"),
  setSchedule: (enabled: boolean) => invoke<ScheduleStatus>("set_schedule", { enabled }),
  appInfo: () => invoke<AppInfo>("app_info"),
  openRepo: () => invoke<void>("open_repo"),
  quit: () => invoke<void>("quit"),
};

export const CLEAN_PROGRESS_EVENT = "clean-progress";
