import { useQueryClient } from "@tanstack/react-query";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { type RefObject, useEffect } from "react";
import { commands } from "@/lib/tauri-commands";
import { fittedHeight, panelShortcut } from "@/lib/panel-keys";

function reportFailure(action: string) {
  return (error: unknown) => console.error(`devclean panel: ${action} failed`, error);
}

/** Keeps the window as tall as its content (up to a cap) so the menu extra never shows empty space. */
export function useFitHeight(frameRef: RefObject<HTMLElement | null>, scrollerRef: RefObject<HTMLElement | null>) {
  useEffect(() => {
    const frame = frameRef.current;
    const scroller = scrollerRef.current;
    const content = scroller?.firstElementChild;
    if (!frame || !scroller || !content) {
      return;
    }
    let applied = 0;
    const fit = () => {
      const height = fittedHeight({
        frameHeight: frame.offsetHeight,
        scrollerVisible: scroller.clientHeight,
        scrollerContent: scroller.scrollHeight,
        screenHeight: window.screen.availHeight,
      });
      if (height !== applied) {
        applied = height;
        getCurrentWindow().setSize(new LogicalSize(window.innerWidth, height)).catch(reportFailure("resize"));
      }
    };
    const observer = new ResizeObserver(fit);
    observer.observe(frame);
    observer.observe(content);
    return () => observer.disconnect();
  }, [frameRef, scrollerRef]);
}

export function usePanelShortcuts() {
  const queryClient = useQueryClient();
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const shortcut = panelShortcut(event);
      const dialogOpen = document.querySelector('[role="alertdialog"]') !== null;
      if (shortcut === null || (shortcut === "close" && dialogOpen)) {
        return;
      }
      event.preventDefault();
      if (shortcut === "close") {
        getCurrentWindow().hide().catch(reportFailure("hide"));
      } else if (shortcut === "quit") {
        commands.quit().catch(reportFailure("quit"));
      } else {
        void queryClient.invalidateQueries();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [queryClient]);
}
