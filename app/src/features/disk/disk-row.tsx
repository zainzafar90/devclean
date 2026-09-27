import { HardDrive } from "lucide-react";
import { Row } from "@/components/ui/grouped-list";
import { IconTile } from "@/components/ui/icon-tile";
import { LevelBar } from "@/components/ui/level-bar";
import { WarningText } from "@/components/ui/warning-text";
import { formatBytes } from "@/lib/format-bytes";
import { diskWarning } from "@/lib/health-rules";
import type { DiskInfo } from "@/lib/tauri-commands";
import { toneFill } from "@/lib/tone-styles";

function DiskDetail({ disk }: { disk: DiskInfo }) {
  const warning = diskWarning(disk);
  if (warning !== null) {
    return <WarningText critical>{warning}</WarningText>;
  }
  return (
    <span className="truncate">
      {formatBytes(disk.free)} free of {formatBytes(disk.total)}
    </span>
  );
}

export function DiskRow({ disk }: { disk: DiskInfo }) {
  const warning = diskWarning(disk);
  return (
    <Row
      icon={<IconTile icon={HardDrive} tint="#8e8e93" />}
      title="Disk"
      detail={<DiskDetail disk={disk} />}
      trailing={<span className="text-[12px] text-secondary-label tabular-nums">{disk.freePercent}% free</span>}
    >
      <LevelBar
        percent={100 - disk.freePercent}
        label="Disk used"
        fillClassName={toneFill[warning === null ? "ok" : "critical"]}
      />
    </Row>
  );
}
