// Shapes shared with the Rust side (src-tauri/src, crates/prepkit-core).

export type JobId = "pixel-fix" | "audio-convert";

export type FileStatus = "written" | "skipped" | "failed";

export interface FileResult {
  path: string;
  output: string | null;
  status: FileStatus;
  message: string;
}

export interface Summary {
  total: number;
  written: number;
  skipped: number;
  failed: number;
  cancelled: boolean;
  results: FileResult[];
}

export interface JobProgress {
  job: JobId;
  result: FileResult;
  done: number;
  total: number;
}

export interface FfmpegInfo {
  path: string;
  version: string;
}

export type InstallStage = "downloading" | "verifying" | "extracting" | "addingToPath";

export interface InstallProgress {
  stage: InstallStage;
  downloaded: number;
  total: number | null;
}

export type OutputMode = "replace" | "folder" | "prefix";

export interface PixelFixOptions {
  recursive: boolean;
  output: OutputMode;
  folder: string;
  prefix: string;
}

export interface AudioOptions {
  normalize: boolean;
  mode: "peak" | "rms";
  targetPeak: number;
  targetRms: number;
  convert: string | null;
  suffix: string;
  overwrite: boolean;
}

export type DropEvent = { type: "enter" | "over" | "leave" } | { type: "drop"; paths: string[] };

/** Everything the window needs from the app shell; implemented by Tauri, or by a mock in a plain browser. */
export interface Backend {
  runPixelFix(paths: string[], options: PixelFixOptions): Promise<Summary>;
  runAudioConvert(paths: string[], options: AudioOptions): Promise<Summary>;
  cancel(job: JobId): Promise<void>;
  /** The images a Pixel Fix run would touch (folders expanded). */
  listImages(paths: string[], options: PixelFixOptions): Promise<string[]>;
  /** The audio files an Audio Convert run would process. */
  listAudio(paths: string[]): Promise<string[]>;
  /** A URL the window can show a listed image from. */
  fileSrc(path: string): string;
  ffmpegStatus(): Promise<FfmpegInfo | null>;
  installFfmpeg(): Promise<FfmpegInfo>;
  loadSettings(): Promise<unknown>;
  saveSettings(settings: object): Promise<void>;
  pickFolders(): Promise<string[]>;
  pickFiles(name: string, extensions: readonly string[]): Promise<string[]>;
  onProgress(handler: (progress: JobProgress) => void): Promise<() => void>;
  onInstallProgress(handler: (progress: InstallProgress) => void): Promise<() => void>;
  onDrop(handler: (event: DropEvent) => void): Promise<() => void>;
  window: {
    minimize(): Promise<void>;
    toggleMaximize(): Promise<void>;
    close(): Promise<void>;
  };
}
