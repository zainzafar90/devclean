import type { LucideIcon } from "lucide-react";

/** White glyph on a tinted rounded square, like the System Settings sidebar. */
export function IconTile({ icon: Icon, tint }: { icon: LucideIcon; tint: string }) {
  return (
    <span
      className="flex size-[22px] shrink-0 items-center justify-center rounded-[6px] bg-linear-to-b from-white/20 to-transparent text-white shadow-[inset_0_0_0_0.5px_rgb(0_0_0/0.08)]"
      style={{ backgroundColor: tint }}
      aria-hidden
    >
      <Icon className="size-[13px]" strokeWidth={2.25} />
    </span>
  );
}
