import { isTauri } from "@tauri-apps/api/core";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./app/App";
import "./styles/tokens.css";
import "./styles/base.css";
import "./styles/layout.css";
import "./styles/controls.css";
import "./styles/files.css";
import "./styles/panels.css";

// In a plain browser (UI preview) there's no transparent window around the app, so give the page a dark backdrop.
if (!isTauri()) document.documentElement.style.background = "#202024";

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}
