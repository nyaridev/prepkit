import { useCallback, useEffect, useState } from "react";
import { api } from "../../lib/api";
import type { FfmpegInfo, InstallProgress } from "../../lib/types";

export type FfmpegState =
  | { kind: "checking" }
  | { kind: "found"; info: FfmpegInfo }
  | { kind: "missing"; error?: string }
  | { kind: "installing"; progress: InstallProgress | null };

/** Whether ffmpeg is available, and a way to install it (download + add to the user's PATH). */
export function useFfmpeg() {
  const [state, setState] = useState<FfmpegState>({ kind: "checking" });

  useEffect(() => {
    let active = true;
    api.ffmpegStatus().then((info) => {
      if (active) setState(info ? { kind: "found", info } : { kind: "missing" });
    });
    return () => {
      active = false;
    };
  }, []);

  const install = useCallback(async () => {
    setState({ kind: "installing", progress: null });
    const unlisten = await api.onInstallProgress((progress) => setState({ kind: "installing", progress }));
    try {
      const info = await api.installFfmpeg();
      setState({ kind: "found", info });
    } catch (error) {
      setState({ kind: "missing", error: String(error) });
    } finally {
      unlisten();
    }
  }, []);

  return { state, install };
}
