// A stand-in backend for previewing the UI in a plain browser (`pnpm dev`). Nothing touches the disk.
// Add `?ffmpeg=missing` to the URL to preview the ffmpeg download banner.

import { fileName } from "../paths";
import type { Backend, FileResult, JobId, JobProgress, Summary } from "../types";

const SETTINGS_KEY = "prepkit-preview-settings";
const progressHandlers = new Set<(progress: JobProgress) => void>();
const cancelled = new Set<JobId>();
let ffmpegMissing = new URLSearchParams(window.location.search).get("ffmpeg") === "missing";

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

const SPRITES = ["hero_idle", "coin", "button_play", "panel_frame", "icon_shop", "badge_gold", "spark", "chest", "gem"];
const SOUNDS = ["jump", "coin_pickup", "hit", "ui_click", "explosion", "splash"];

/** Folders become a few made-up files (plus a sub-folder's when recursive); files stay as they are. */
function expand(paths: string[], names: string[], extension: string, recursive = false): string[] {
  return paths.flatMap((path) => {
    if (/\.\w{2,4}$/.test(path)) return [path];
    const own = names.map((name) => `${path}\\${name}.${extension}`);
    const nested = recursive ? names.slice(0, 4).map((name) => `${path}\\icons\\${name}_small.${extension}`) : [];
    return [...own, ...nested];
  });
}

/** A small made-up sprite (a colored shape on transparency) so previews have something to show. */
function placeholder(path: string): string {
  const hash = [...fileName(path)].reduce((sum, char) => sum + char.charCodeAt(0), 0);
  const hue = (hash * 47) % 360;
  const shape =
    hash % 3 === 0
      ? `<circle cx="32" cy="32" r="20" fill="hsl(${hue} 70% 55%)"/>`
      : hash % 3 === 1
        ? `<rect x="14" y="14" width="36" height="36" rx="6" fill="hsl(${hue} 65% 52%)"/>`
        : `<path d="M32 10l22 40H10z" fill="hsl(${hue} 70% 58%)"/>`;
  return `data:image/svg+xml,${encodeURIComponent(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">${shape}</svg>`)}`;
}

async function simulate(job: JobId, files: string[], describe: (path: string, i: number) => FileResult) {
  cancelled.delete(job);
  const results: FileResult[] = [];
  for (const [index, path] of files.entries()) {
    if (cancelled.has(job)) break;
    await wait(150);
    const result = describe(path, index);
    results.push(result);
    for (const handler of progressHandlers) handler({ job, result, done: index + 1, total: files.length });
  }
  const count = (status: FileResult["status"]) => results.filter((r) => r.status === status).length;
  const summary: Summary = {
    total: files.length,
    written: count("written"),
    skipped: count("skipped"),
    failed: count("failed"),
    cancelled: cancelled.has(job),
    results,
  };
  return summary;
}

export const mockBackend: Backend = {
  runPixelFix: (paths) =>
    simulate("pixel-fix", paths, (path, i) => ({
      path,
      output: null,
      status: i === 3 ? "skipped" : "written",
      message: i === 3 ? "No transparent pixels to fix" : "Fixed",
    })),
  runAudioConvert: (paths, options) =>
    simulate("audio-convert", paths, (path, i) => ({
      path,
      output: path.replace(/([^\\]+)\.\w+$/, `normalized\\$1${options.suffix}.${options.convert ?? "wav"}`),
      status: i === 4 ? "failed" : "written",
      message: i === 4 ? "ffmpeg failed: Invalid data found when processing input" : `+${(3 + i * 1.7).toFixed(1)} dB`,
    })),
  cancel: async (job) => {
    cancelled.add(job);
  },
  listImages: async (paths, options) => expand(paths, SPRITES, "png", options.recursive),
  listAudio: async (paths) => expand(paths, SOUNDS, "wav"),
  fileSrc: placeholder,
  ffmpegStatus: async () => (ffmpegMissing ? null : { path: "C:\\ffmpeg\\bin\\ffmpeg.exe", version: "7.1" }),
  installFfmpeg: async () => {
    await wait(1500);
    ffmpegMissing = false;
    return { path: "C:\\Users\\you\\AppData\\Local\\Programs\\ffmpeg\\bin\\ffmpeg.exe", version: "nightly 2026-10-20" };
  },
  loadSettings: async () => JSON.parse(localStorage.getItem(SETTINGS_KEY) ?? "null"),
  saveSettings: async (settings) => localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings)),
  pickFolders: async () => ["D:\\Game\\assets\\ui"],
  pickFiles: async () => ["D:\\Game\\assets\\ui\\logo.png", "D:\\Game\\assets\\ui\\star.png"],
  onProgress: async (handler) => {
    progressHandlers.add(handler);
    return () => progressHandlers.delete(handler);
  },
  onInstallProgress: async () => () => {},
  onDrop: async () => () => {},
  window: {
    minimize: async () => {},
    toggleMaximize: async () => {},
    close: async () => {},
  },
};
