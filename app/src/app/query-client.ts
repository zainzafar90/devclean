import { QueryClient, focusManager } from "@tanstack/react-query";
import { getCurrentWindow } from "@tauri-apps/api/window";

export const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: 1, refetchOnWindowFocus: true } },
});

// Polling follows the panel window: it hides on blur, so focus means "open".
focusManager.setEventListener((setFocused) => {
  const unlisten = getCurrentWindow().onFocusChanged(({ payload }) => setFocused(payload));
  return () => {
    void unlisten.then((stop) => stop());
  };
});
