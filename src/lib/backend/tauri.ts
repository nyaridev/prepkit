// The real backend: Tauri commands, events, the native file pickers and window controls.

import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import type { Backend, DropEvent, InstallProgress, JobProgress } from "../types";

function asList(selection: string | string[] | null): string[] {
  if (selection === null) return [];
  return Array.isArray(selection) ? selection : [selection];
}

export const tauriBackend: Backend = {
  runPixelFix: (paths, options) => invoke("run_pixel_fix", { paths, options }),
  runAudioConvert: (paths, options) => invoke("run_audio_convert", { paths, options }),
  cancel: (job) => invoke("cancel_job", { job }),
  listImages: (paths, options) => invoke("list_images", { paths, options }),
  listAudio: (paths) => invoke("list_audio", { paths }),
  fileSrc: (path) => convertFileSrc(path),
  ffmpegStatus: () => invoke("ffmpeg_status"),
  installFfmpeg: () => invoke("install_ffmpeg"),
  loadSettings: () => invoke("load_settings"),
  saveSettings: (settings) => invoke("save_settings", { settings }),

  pickFolders: async () => asList(await open({ directory: true, multiple: true })),
  pickFiles: async (name, extensions) =>
    asList(await open({ multiple: true, filters: [{ name, extensions: [...extensions] }] })),

  onProgress: (handler) => listen<JobProgress>("job-progress", (event) => handler(event.payload)),
  onInstallProgress: (handler) => listen<InstallProgress>("ffmpeg-install-progress", (event) => handler(event.payload)),
  onDrop: (handler) =>
    getCurrentWebview().onDragDropEvent((event) => {
      const payload = event.payload;
      const drop: DropEvent = payload.type === "drop" ? { type: "drop", paths: payload.paths } : { type: payload.type };
      handler(drop);
    }),

  window: {
    minimize: () => getCurrentWindow().minimize(),
    toggleMaximize: () => getCurrentWindow().toggleMaximize(),
    close: () => getCurrentWindow().close(),
  },
};
