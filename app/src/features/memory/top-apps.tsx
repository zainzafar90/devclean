import { Section } from "@/components/ui/grouped-list";
import { formatBytes } from "@/lib/format-bytes";
import type { ProcessUsage } from "@/lib/tauri-commands";
import { cn } from "@/lib/utils";

const HEAVY_BYTES = 2 * 1024 ** 3;

export function TopApps({ apps }: { apps: ProcessUsage[] }) {
  if (apps.length === 0) {
    return null;
  }
  return (
    <Section title="Top Apps by Memory">
      <li className="flex flex-col py-1.5">
        {apps.map((app) => (
          <div key={app.name} className="flex items-center justify-between gap-3 py-[2px] text-[11px] leading-[14px]">
            <span className="truncate text-label" title={app.name}>
              {app.name}
            </span>
            <span
              className={cn("shrink-0 tabular-nums", app.bytes > HEAVY_BYTES ? "text-system-orange" : "text-secondary-label")}
            >
              {formatBytes(app.bytes)}
            </span>
          </div>
        ))}
      </li>
    </Section>
  );
}
