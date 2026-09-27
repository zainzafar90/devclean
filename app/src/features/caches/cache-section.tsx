import { Button } from "@/components/ui/button";
import { Section } from "@/components/ui/grouped-list";
import { Spinner } from "@/components/ui/spinner";
import { formatBytes } from "@/lib/format-bytes";
import { isMeasuring, safeAreas, totalBytes } from "@/lib/health-rules";
import { CacheRow } from "./cache-row";
import { type AreaEntry, useAreas, useCleaning } from "./use-cleaning";

function CachesTotal({ areas }: { areas: AreaEntry[] }) {
  if (areas.length === 0) {
    return null;
  }
  return (
    <>
      {formatBytes(totalBytes(areas))}
      {isMeasuring(areas) ? <Spinner label="Measuring caches" className="size-3" /> : null}
    </>
  );
}

function CleanStatus({ line, error }: { line: string | undefined; error: string | null }) {
  if (error !== null) {
    return <p className="text-system-red">{error}</p>;
  }
  if (line === undefined) {
    return null;
  }
  return <p className="truncate">{line}</p>;
}

export function CacheSection() {
  const { areas, error } = useAreas();
  const cleaning = useCleaning();
  const safe = safeAreas(areas);
  return (
    <Section
      title="Caches"
      detail={<CachesTotal areas={areas} />}
      trailing={
        <Button
          variant="prominent"
          size="small"
          disabled={cleaning.isCleaning || safe.length === 0}
          onClick={() => cleaning.clean(safe.map((area) => area.area))}
        >
          Clean Safe Items
        </Button>
      }
      footer={<CleanStatus line={cleaning.progress?.line} error={cleaning.error ?? error} />}
    >
      {areas.map((area) => (
        <CacheRow
          key={area.area}
          area={area}
          outcome={cleaning.outcomes[area.area]}
          busy={cleaning.cleaningAreas.includes(area.area)}
          disabled={cleaning.isCleaning}
          onClean={() => cleaning.clean([area.area])}
        />
      ))}
    </Section>
  );
}
