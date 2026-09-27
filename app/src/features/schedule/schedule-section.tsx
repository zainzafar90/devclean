import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { CalendarClock } from "lucide-react";
import { Row, Section } from "@/components/ui/grouped-list";
import { IconTile } from "@/components/ui/icon-tile";
import { Switch } from "@/components/ui/switch";
import { commands, type LastRun, type ScheduleStatus } from "@/lib/tauri-commands";

const SCHEDULE_KEY = ["schedule"];

function lastRunText(lastRun: LastRun | null): string {
  if (lastRun === null) {
    return "No scheduled run logged yet";
  }
  if (lastRun.finished === null) {
    return `Last run started ${lastRun.started}, not finished`;
  }
  const freed = lastRun.freed === null ? "" : ` · freed ${lastRun.freed}`;
  return `Last run ${lastRun.finished}${freed}`;
}

export function ScheduleSection() {
  const queryClient = useQueryClient();
  const status = useQuery({ queryKey: SCHEDULE_KEY, queryFn: commands.scheduleStatus });
  const toggle = useMutation({
    mutationFn: commands.setSchedule,
    onSuccess: (next: ScheduleStatus) => queryClient.setQueryData(SCHEDULE_KEY, next),
  });
  const error = toggle.error ?? status.error;
  return (
    <Section
      title="Schedule"
      footer={error ? <span className="text-system-red">{String(error)}</span> : lastRunText(status.data?.lastRun ?? null)}
    >
      <Row
        icon={<IconTile icon={CalendarClock} tint="#5e5ce6" />}
        title={<label htmlFor="schedule-switch">Daily clean at 03:00</label>}
        detail={<span className="truncate">Docker leftovers only, via launchd</span>}
        trailing={
          <Switch
            id="schedule-switch"
            checked={status.data?.enabled ?? false}
            disabled={status.isPending || toggle.isPending}
            onCheckedChange={(enabled) => toggle.mutate(enabled)}
          />
        }
      />
    </Section>
  );
}
