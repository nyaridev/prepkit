// What the window remembers between runs. Folders and files are never part of it.

import type { AudioOptions, JobId, OutputMode, PixelFixOptions } from "./types";

export const AUDIO_FORMATS = ["ogg", "mp3", "wav", "flac", "m4a", "aac", "opus", "aiff", "wma"] as const;
export type AudioFormat = (typeof AUDIO_FORMATS)[number];

export const IMAGE_EXTENSIONS = ["png", "webp", "bmp", "gif", "jpg", "jpeg", "tif", "tiff", "apng"] as const;
export const AUDIO_EXTENSIONS = ["wav", "ogg", "mp3", "flac", "aac", "m4a", "wma", "aiff", "aif", "opus"] as const;

export type FileView = "list" | "grid";

export interface Settings {
  lastTab: JobId;
  pixelFix: { recursive: boolean; output: OutputMode; folder: string; prefix: string; view: FileView };
  audio: {
    normalize: boolean;
    mode: "peak" | "rms";
    targetPeak: number;
    targetRms: number;
    /** `keep` leaves each file in its own format. */
    format: "keep" | AudioFormat;
    suffix: string;
  };
}

export const DEFAULT_SETTINGS: Settings = {
  lastTab: "pixel-fix",
  pixelFix: { recursive: false, output: "replace", folder: "fixed", prefix: "fixed_", view: "list" },
  audio: { normalize: true, mode: "rms", targetPeak: -1, targetRms: -14, format: "keep", suffix: "_n" },
};

export const TARGET_RANGE = { min: -60, max: 0 } as const;

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

const bool = (value: unknown, fallback: boolean) => (typeof value === "boolean" ? value : fallback);

const oneOf = <T extends string>(value: unknown, options: readonly T[], fallback: T): T =>
  options.includes(value as T) ? (value as T) : fallback;

const target = (value: unknown, fallback: number) =>
  typeof value === "number" && Number.isFinite(value)
    ? Math.min(TARGET_RANGE.max, Math.max(TARGET_RANGE.min, value))
    : fallback;

/** Keeps a suffix, prefix or folder name safe to put in a file name. */
export function cleanName(value: string): string {
  return value.replace(/[\\/:*?"<>|]/g, "").slice(0, 32);
}

const name = (value: unknown, fallback: string) => (typeof value === "string" ? cleanName(value) : fallback);

/** Reads saved settings, falling back to the default for anything missing or invalid. */
export function mergeSettings(raw: unknown): Settings {
  const saved = isRecord(raw) ? raw : {};
  const pixelFix = isRecord(saved.pixelFix) ? saved.pixelFix : {};
  const audio = isRecord(saved.audio) ? saved.audio : {};
  const pixelDefaults = DEFAULT_SETTINGS.pixelFix;
  const audioDefaults = DEFAULT_SETTINGS.audio;

  // Older versions called the audio tab "audio-normalize" and had a separate convert switch.
  const lastTab =
    saved.lastTab === "audio-convert" || saved.lastTab === "audio-normalize" ? "audio-convert" : "pixel-fix";
  const legacyFormat =
    audio.convertEnabled === true ? audio.convertFormat : audio.convertEnabled === false ? "keep" : undefined;

  return {
    lastTab,
    pixelFix: {
      recursive: bool(pixelFix.recursive, pixelDefaults.recursive),
      output: oneOf(pixelFix.output, ["replace", "folder", "prefix"], pixelDefaults.output),
      folder: name(pixelFix.folder, pixelDefaults.folder),
      prefix: name(pixelFix.prefix, pixelDefaults.prefix),
      view: oneOf(pixelFix.view, ["list", "grid"], pixelDefaults.view),
    },
    audio: {
      normalize: bool(audio.normalize, audioDefaults.normalize),
      mode: oneOf(audio.mode, ["peak", "rms"], audioDefaults.mode),
      targetPeak: target(audio.targetPeak, audioDefaults.targetPeak),
      targetRms: target(audio.targetRms, audioDefaults.targetRms),
      format: oneOf(audio.format ?? legacyFormat, ["keep", ...AUDIO_FORMATS], audioDefaults.format),
      suffix: name(audio.suffix, audioDefaults.suffix),
    },
  };
}

/** The options the Pixel Fix tool runs (and lists files) with. */
export function pixelFixOptions(pixelFix: Settings["pixelFix"]): PixelFixOptions {
  return { recursive: pixelFix.recursive, output: pixelFix.output, folder: pixelFix.folder, prefix: pixelFix.prefix };
}

/** The options the audio tool runs with. */
export function audioOptions(audio: Settings["audio"]): AudioOptions {
  return {
    normalize: audio.normalize,
    mode: audio.mode,
    targetPeak: audio.targetPeak,
    targetRms: audio.targetRms,
    convert: audio.format === "keep" ? null : audio.format,
    suffix: audio.suffix,
    overwrite: false,
  };
}
