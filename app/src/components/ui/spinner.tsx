import { cn } from "cn";

const SPOKES = [0, 1, 2, 3, 4, 5, 6, 7];

/** NSProgressIndicator spinning style: eight spokes stepping round, brightest at the head. */
export function Spinner({ label, className }: { label: string; className?: string }) {
  return (
    <svg viewBox="0 0 16 16" role="img" aria-label={label} className={cn("size-3.5 animate-mac-spin", className)}>
      {SPOKES.map((spoke) => (
        <rect
          key={spoke}
          x="7.2"
          y="1"
          width="1.6"
          height="4.4"
          rx="0.8"
          fill="currentColor"
          opacity={1 - (((8 - spoke) % 8) / 8) * 0.85}
          transform={`rotate(${spoke * 45} 8 8)`}
        />
      ))}
    </svg>
  );
}
