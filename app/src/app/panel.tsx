import { useRef } from "react";
import { Section } from "@/components/ui/grouped-list";
import { Spinner } from "@/components/ui/spinner";
import { AboutFooter } from "@/features/about/about-footer";
import { CacheSection } from "@/features/caches/cache-section";
import { DiskRow } from "@/features/disk/disk-row";
import { MemoryRow } from "@/features/memory/memory-row";
import { TopApps } from "@/features/memory/top-apps";
import { useSnapshot } from "@/features/memory/use-snapshot";
import { ScheduleSection } from "@/features/schedule/schedule-section";
import { PanelHeader } from "./panel-header";
import { useFitHeight, usePanelShortcuts } from "./use-panel-window";

function SystemSections() {
  const snapshot = useSnapshot();
  if (snapshot.error) {
    return <p className="px-2 text-[11px] text-system-red">{String(snapshot.error)}</p>;
  }
  if (snapshot.data === undefined) {
    return (
      <p className="flex items-center gap-1.5 px-2 text-[11px] text-secondary-label">
        <Spinner label="Reading memory and disk" className="size-3" />
        Reading memory and disk…
      </p>
    );
  }
  return (
    <>
      <Section title="System">
        <MemoryRow memory={snapshot.data.memory} />
        <DiskRow disk={snapshot.data.disk} />
      </Section>
      <TopApps apps={snapshot.data.topApps} />
    </>
  );
}

export function Panel() {
  const frameRef = useRef<HTMLDivElement>(null);
  const scrollerRef = useRef<HTMLElement>(null);
  useFitHeight(frameRef, scrollerRef);
  usePanelShortcuts();
  return (
    <div ref={frameRef} className="flex max-h-full flex-col">
      <PanelHeader />
      <main ref={scrollerRef} className="min-h-0 flex-1 overflow-y-auto overscroll-contain px-2 pb-2.5">
        <div className="flex flex-col gap-3.5">
          <SystemSections />
          <CacheSection />
          <ScheduleSection />
        </div>
      </main>
      <AboutFooter />
    </div>
  );
}
