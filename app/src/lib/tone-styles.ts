import type { Tone } from "./health-rules";

export const toneFill: Record<Tone, string> = {
  ok: "bg-secondary-label",
  warn: "bg-system-orange",
  critical: "bg-system-red",
};
