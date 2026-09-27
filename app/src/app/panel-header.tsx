import { useQueryClient } from "@tanstack/react-query";
import { RotateCw } from "lucide-react";
import { Button } from "@/components/ui/button";

export function PanelHeader() {
  const queryClient = useQueryClient();
  return (
    <header className="flex items-center justify-between py-2.5 pr-2 pl-4">
      <h1 className="text-[13px] font-semibold">devclean</h1>
      <Button
        variant="borderless"
        size="icon"
        aria-label="Refresh"
        title="Refresh (⌘R)"
        onClick={() => void queryClient.invalidateQueries()}
      >
        <RotateCw strokeWidth={2} />
      </Button>
    </header>
  );
}
