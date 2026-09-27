import { cn } from "cn";

/** Thin level indicator like the Battery menu's; `fillClassName` carries the tone. */
export function LevelBar({ percent, label, fillClassName }: { percent: number; label: string; fillClassName: string }) {
  const value = Math.min(100, Math.max(0, Math.round(percent)));
  return (
    <div
      role="meter"
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={value}
      className="h-1 w-full overflow-hidden rounded-full bg-track"
    >
      <div className={cn("h-full rounded-full transition-[width] duration-300", fillClassName)} style={{ width: `${value}%` }} />
    </div>
  );
}
