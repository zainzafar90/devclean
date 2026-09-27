import { QueryClientProvider } from "@tanstack/react-query";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Panel } from "./app/panel";
import { queryClient } from "./app/query-client";
import "./index.css";

const root = document.getElementById("root");
if (root === null) {
  throw new Error("#root element missing from index.html");
}

createRoot(root).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <Panel />
    </QueryClientProvider>
  </StrictMode>,
);
