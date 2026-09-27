import { Row } from "@/components/ui/grouped-list";
import { Spinner } from "@/components/ui/spinner";
import { formatBytes } from "@/lib/format-bytes";
import { freedBytes } from "@/lib/health-rules";
import type { CleanOutcome } from "@/lib/tauri-commands";
import { areaName } from "./area-looks";
import { AreaTile } from "./area-tile";
import { CleanButton } from "./clean-button";
import type { AreaEntry } from "./use-cleaning";

interface CacheRowProps {
  area: AreaEntry;
  outcome: CleanOutcome | undefined;
  busy: boolean;
  disabled: boolean;
  onClean: () => void;
}

function RowDetail({ area, outcome, busy }: { area: AreaEntry; outcome: CleanOutcome | undefined; busy: boolean }) {
  if (busy) {
    return <span className="truncate">Cleaning…</span>;
  }
  if (outcome === undefined) {
    return (
      <span className="truncate" title={area.label}>
        {area.label}
      </span>
    );
  }
  if (outcome.failures.length > 0) {
    return (
      <span className="truncate text-system-red" title={outcome.failures.join("\n")}>
        {outcome.failures.length === 1 ? "1 error" : `${outcome.failures.length} errors`}
      </span>
    );
  }
  return <span className="truncate text-system-green">Freed {formatBytes(freedBytes(outcome))}</span>;
}

function AreaBytes({ area }: { area: AreaEntry }) {
  if (area.measureError !== null) {
    return (
      <span className="text-tertiary-label" title={area.measureError}>
        —
      </span>
    );
  }
  if (area.bytes === undefined) {
    return <Spinner label={`Measuring ${areaName(area.area)}`} className="text-secondary-label" />;
  }
  return <>{formatBytes(area.bytes)}</>;
}

export function CacheRow({ area, outcome, busy, disabled, onClean }: CacheRowProps) {
  const empty = area.bytes === undefined || area.bytes === 0;
  return (
    <Row
      icon={<AreaTile area={area.area} />}
      title={areaName(area.area)}
      detail={<RowDetail area={area} outcome={outcome} busy={busy} />}
      trailing={
        <>
          <span className="flex w-14 shrink-0 justify-end text-[12px] text-secondary-label tabular-nums">
            <AreaBytes area={area} />
          </span>
          <CleanButton area={area} busy={busy} disabled={disabled || empty} onClean={onClean} />
        </>
      }
    />
  );
}
