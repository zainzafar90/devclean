import { useQuery } from "@tanstack/react-query";
import { commands } from "@/lib/tauri-commands";

export const SNAPSHOT_KEY = ["snapshot"];
const REFRESH_MS = 5000;

export function useSnapshot() {
  return useQuery({ queryKey: SNAPSHOT_KEY, queryFn: commands.snapshot, refetchInterval: REFRESH_MS });
}
