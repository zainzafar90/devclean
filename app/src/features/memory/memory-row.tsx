import { MemoryStick } from "lucide-react";
import { Row } from "@/components/ui/grouped-list";
import { IconTile } from "@/components/ui/icon-tile";
import { LevelBar } from "@/components/ui/level-bar";
import { WarningText } from "@/components/ui/warning-text";
import { formatBytes } from "@/lib/format-bytes";
import { isMemoryLow, memoryUsedPercent, memoryWarning, pressureTone } from "@/lib/health-rules";
import type { MemoryInfo } from "@/lib/tauri-commands";
import { toneFill } from "@/lib/tone-styles";

function MemoryDetail({ memory }: { memory: MemoryInfo }) {
  const warning = memoryWarning(memory);
  if (warning !== null) {
    return <WarningText critical={memory.pressure === "critical"}>{warning}</WarningText>;
  }
  return (
    <span className="truncate" title={`${formatBytes(memory.available)} available of ${formatBytes(memory.total)}`}>
      {formatBytes(memory.available)} available · swap {formatBytes(memory.swapUsed)}
    </span>
  );
}

export function MemoryRow({ memory }: { memory: MemoryInfo }) {
  const tone = pressureTone(memory.pressure, isMemoryLow(memory));
  const free = memory.freePercent === null ? "?" : `${memory.freePercent}%`;
  return (
    <Row
      icon={<IconTile icon={MemoryStick} tint="#30b650" />}
      title="Memory"
      detail={<MemoryDetail memory={memory} />}
      trailing={<span className="text-[12px] text-secondary-label tabular-nums">{free} free</span>}
    >
      <LevelBar percent={memoryUsedPercent(memory)} label="Memory used" fillClassName={toneFill[tone]} />
    </Row>
  );
}
