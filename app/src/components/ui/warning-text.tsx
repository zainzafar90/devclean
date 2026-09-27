import { cn } from "cn";
import { TriangleAlert } from "lucide-react";
import type { ReactNode } from "react";

/** System orange or red text with an exclamation-triangle glyph, the way macOS flags a problem inline. */
export function WarningText({ critical, children }: { critical: boolean; children: ReactNode }) {
  return (
    <span className={cn("inline-flex min-w-0 items-center gap-1", critical ? "text-system-red" : "text-system-orange")}>
      <TriangleAlert className="size-3 shrink-0" strokeWidth={2.5} aria-hidden />
      <span className="truncate">{children}</span>
    </span>
  );
}
