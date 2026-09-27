import { IconTile } from "@/components/ui/icon-tile";
import type { AreaName } from "@/lib/tauri-commands";
import { AREA_LOOKS } from "./area-looks";

export function AreaTile({ area }: { area: AreaName }) {
  return <IconTile icon={AREA_LOOKS[area].icon} tint={AREA_LOOKS[area].tint} />;
}
