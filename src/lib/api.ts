import { isTauri } from "@tauri-apps/api/core";
import { mockBackend } from "./backend/mock";
import { tauriBackend } from "./backend/tauri";
import type { Backend } from "./types";

/** The Tauri backend inside the app; a mock when the UI is opened in a plain browser for previewing. */
export const api: Backend = isTauri() ? tauriBackend : mockBackend;
