import { useMutation, useQueries, useQuery, useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { SNAPSHOT_KEY } from "@/features/memory/use-snapshot";
import {
  type AreaInfo,
  type AreaName,
  CLEAN_PROGRESS_EVENT,
  type CleanOutcome,
  type CleanProgress,
  commands,
} from "@/lib/tauri-commands";

export const AREAS_KEY = ["areas"];

export interface AreaEntry extends AreaInfo {
  bytes: number | undefined;
  measureError: string | null;
}

/** Every area at once from the registry; each size streams in from its own command as it is measured. */
export function useAreas() {
  const list = useQuery({ queryKey: [...AREAS_KEY, "list"], queryFn: commands.areas, staleTime: Infinity });
  const infos = list.data ?? [];
  const sizes = useQueries({
    queries: infos.map((info) => ({
      queryKey: [...AREAS_KEY, "size", info.area],
      queryFn: () => commands.areaSize(info.area),
      staleTime: 60_000,
    })),
  });
  const areas: AreaEntry[] = infos.map((info, index) => {
    const size = sizes[index];
    return { ...info, bytes: size?.data, measureError: size?.error ? String(size.error) : null };
  });
  return { areas, error: list.error === null ? null : String(list.error) };
}

function useCleanProgress() {
  const [progress, setProgress] = useState<CleanProgress | null>(null);
  useEffect(() => {
    const unlisten = listen<CleanProgress>(CLEAN_PROGRESS_EVENT, (event) => setProgress(event.payload));
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, []);
  return [progress, setProgress] as const;
}

type Outcomes = Partial<Record<AreaName, CleanOutcome>>;

export function useCleaning() {
  const queryClient = useQueryClient();
  const [progress, setProgress] = useCleanProgress();
  const [outcomes, setOutcomes] = useState<Outcomes>({});
  const mutation = useMutation({
    mutationFn: commands.clean,
    onSuccess: (results) => {
      setOutcomes((previous) => ({ ...previous, ...Object.fromEntries(results.map((r) => [r.area, r])) }));
    },
    onSettled: () => {
      setProgress(null);
      void queryClient.invalidateQueries({ queryKey: AREAS_KEY });
      void queryClient.invalidateQueries({ queryKey: SNAPSHOT_KEY });
    },
  });
  return {
    clean: mutation.mutate,
    isCleaning: mutation.isPending,
    cleaningAreas: mutation.isPending ? (mutation.variables ?? []) : [],
    progress,
    outcomes,
    error: mutation.error === null ? null : String(mutation.error),
  };
}
