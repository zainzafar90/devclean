import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { formatBytes } from "@/lib/format-bytes";
import { needsConfirmation } from "@/lib/health-rules";
import appIcon from "../../../src-tauri/icons/128x128.png";
import { areaName, confirmNote } from "./area-looks";
import type { AreaEntry } from "./use-cleaning";

interface CleanButtonProps {
  area: AreaEntry;
  busy: boolean;
  disabled: boolean;
  onClean: () => void;
}

/** "Clean…" carries the macOS ellipsis when the button asks before acting. */
function ButtonLabel({ busy, asks }: { busy: boolean; asks: boolean }) {
  if (busy) {
    return <Spinner label="Cleaning" className="size-3" />;
  }
  return <>{asks ? "Clean…" : "Clean"}</>;
}

export function CleanButton({ area, busy, disabled, onClean }: CleanButtonProps) {
  const asks = needsConfirmation(area);
  const button = (
    <Button size="small" disabled={disabled} onClick={asks ? undefined : onClean}>
      <ButtonLabel busy={busy} asks={asks} />
    </Button>
  );
  if (!asks) {
    return button;
  }
  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>{button}</AlertDialogTrigger>
      <AlertDialogContent>
        <img src={appIcon} alt="" className="size-12" />
        <AlertDialogTitle>Clean {areaName(area.area)}?</AlertDialogTitle>
        <AlertDialogDescription>
          {area.label}. About {formatBytes(area.bytes ?? 0)} will be removed{confirmNote(area.area)}
        </AlertDialogDescription>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction onClick={onClean}>Clean</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
