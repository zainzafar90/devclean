export type PanelShortcut = "close" | "quit" | "refresh";

/** Menu-extra keys: Esc closes the panel, ⌘Q quits, ⌘R measures again. */
export function panelShortcut(event: Pick<KeyboardEvent, "key" | "metaKey">): PanelShortcut | null {
  if (event.key === "Escape") {
    return "close";
  }
  if (!event.metaKey) {
    return null;
  }
  const key = event.key.toLowerCase();
  if (key === "q") {
    return "quit";
  }
  if (key === "r") {
    return "refresh";
  }
  return null;
}

export const MAX_PANEL_HEIGHT = 720;
const SCREEN_MARGIN = 48;

interface PanelLayout {
  frameHeight: number;
  scrollerVisible: number;
  scrollerContent: number;
  screenHeight: number;
}

/** Window height that shows all content, capped by the screen and a menu-sized maximum; the list scrolls past it. */
export function fittedHeight({ frameHeight, scrollerVisible, scrollerContent, screenHeight }: PanelLayout): number {
  const natural = frameHeight - scrollerVisible + scrollerContent;
  const cap = Math.min(MAX_PANEL_HEIGHT, screenHeight - SCREEN_MARGIN);
  return Math.ceil(Math.max(0, Math.min(natural, cap)));
}
