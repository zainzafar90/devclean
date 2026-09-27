import {
  Beer,
  Boxes,
  Container,
  GitBranch,
  Hammer,
  type LucideIcon,
  Package,
  PackageOpen,
  Smartphone,
  Theater,
  Zap,
} from "lucide-react";
import type { AreaName } from "@/lib/tauri-commands";

export interface AreaLook {
  name: string;
  icon: LucideIcon;
  tint: string;
}

export const AREA_LOOKS: Record<AreaName, AreaLook> = {
  docker: { name: "Docker", icon: Container, tint: "#1d8cf8" },
  pnpm: { name: "pnpm", icon: Package, tint: "#f69220" },
  uv: { name: "uv", icon: Zap, tint: "#5e5ce6" },
  npm: { name: "npm", icon: Package, tint: "#e5383b" },
  yarn: { name: "Yarn", icon: Package, tint: "#2c8ebb" },
  brew: { name: "Homebrew", icon: Beer, tint: "#f4a000" },
  pip: { name: "pip", icon: PackageOpen, tint: "#3b7bbf" },
  xcode: { name: "Xcode", icon: Hammer, tint: "#147efb" },
  simulators: { name: "Simulators", icon: Smartphone, tint: "#8e8e93" },
  playwright: { name: "Playwright", icon: Theater, tint: "#2ead33" },
  cocoapods: { name: "CocoaPods", icon: Boxes, tint: "#ee3a52" },
  worktrees: { name: "Worktrees", icon: GitBranch, tint: "#a550d8" },
};

export function areaName(area: AreaName): string {
  return AREA_LOOKS[area].name;
}

/** What else the confirmation should warn about once the size has been stated. */
export function confirmNote(area: AreaName): string {
  return area === "worktrees"
    ? ". Files git ignores inside them, such as .env, are deleted too."
    : " and may need a re-download later.";
}
