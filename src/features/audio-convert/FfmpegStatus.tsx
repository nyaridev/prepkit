import { DownloadIcon } from "../../components/icons";
import type { InstallProgress } from "../../lib/types";
import type { FfmpegState } from "./useFfmpeg";

const STAGE_LABEL: Record<InstallProgress["stage"], string> = {
  downloading: "Downloading",
  verifying: "Checking the download",
  extracting: "Unpacking",
  addingToPath: "Adding to PATH",
};

function progressText(progress: InstallProgress | null): string {
  if (!progress) return "Starting…";
  if (progress.stage !== "downloading") return `${STAGE_LABEL[progress.stage]}…`;
  const mb = (bytes: number) => (bytes / 1_048_576).toFixed(0);
  return progress.total
    ? `Downloading… ${mb(progress.downloaded)} / ${mb(progress.total)} MB`
    : `Downloading… ${mb(progress.downloaded)} MB`;
}

/** A small pill next to the card title: still looking, or which ffmpeg is used. */
export function FfmpegPill({ state }: { state: FfmpegState }) {
  if (state.kind === "checking") return <div className="pill">Looking for ffmpeg…</div>;
  if (state.kind === "found") {
    return (
      <div className="pill pill-ok" title={state.info.path}>
        <span className="pill-dot" />
        ffmpeg {state.info.version}
      </div>
    );
  }
  return (
    <div className="pill pill-missing">
      <span className="pill-dot" />
      no ffmpeg
    </div>
  );
}

/** Shown above the start button while ffmpeg is missing: what it's for, and a button that installs it. */
export function FfmpegBanner({ state, onInstall }: { state: FfmpegState; onInstall: () => void }) {
  if (state.kind !== "missing" && state.kind !== "installing") return null;
  const installing = state.kind === "installing";
  const percent =
    installing && state.progress?.stage === "downloading" && state.progress.total
      ? Math.round((state.progress.downloaded / state.progress.total) * 100)
      : null;

  return (
    <div className="banner">
      <div className="banner-text">
        <strong>Audio needs ffmpeg</strong>
        <span>
          {installing
            ? progressText(state.progress)
            : "PrepKit can download it (about 200 MB) and add it to your PATH, so other tools can use it too."}
        </span>
        {state.kind === "missing" && state.error && <span className="banner-error">{state.error}</span>}
        {installing && (
          <div className="progress progress-small">
            <div className="progress-fill" style={{ width: percent === null ? "100%" : `${percent}%` }} />
          </div>
        )}
      </div>
      <button type="button" className="button button-primary" disabled={installing} onClick={onInstall}>
        <DownloadIcon />
        {installing ? "Installing…" : "Download ffmpeg"}
      </button>
    </div>
  );
}
