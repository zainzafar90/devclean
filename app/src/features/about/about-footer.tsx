import { useQuery } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { commands } from "@/lib/tauri-commands";

/** An NSMenu item: accent highlight with white text on hover or keyboard focus. */
function MenuItem({ label, shortcut, onSelect }: { label: string; shortcut: ReactNode; onSelect: () => void }) {
  return (
    <button
      type="button"
      onClick={onSelect}
      className="group flex h-[22px] w-full items-center justify-between rounded-[5px] px-2.5 text-left text-[13px] text-label outline-none hover:bg-accent hover:text-white focus-visible:bg-accent focus-visible:text-white"
    >
      {label}
      <span className="text-secondary-label tabular-nums group-hover:text-white/75 group-focus-visible:text-white/75">
        {shortcut}
      </span>
    </button>
  );
}

export function AboutFooter() {
  const info = useQuery({ queryKey: ["app-info"], queryFn: commands.appInfo, staleTime: Infinity });
  const name = info.data?.name ?? "devclean";
  return (
    <footer className="flex flex-col px-1.5 pb-1.5">
      <div className="mx-2.5 mb-1 border-t-[0.5px] border-separator" />
      <MenuItem label={`About ${name}`} shortcut={info.data?.version} onSelect={() => void commands.openRepo()} />
      <MenuItem label={`Quit ${name}`} shortcut="⌘Q" onSelect={() => void commands.quit()} />
      <p className="px-2.5 pt-1 text-[10px] text-tertiary-label">Built by {info.data?.author ?? "Zain Zafar"}</p>
    </footer>
  );
}
