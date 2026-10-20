import { useCallback, useEffect, useState } from "react";
import { api } from "../lib/api";
import type { FileResult, JobId, Summary } from "../lib/types";

export interface JobState {
  running: boolean;
  done: number;
  total: number;
  log: FileResult[];
  summary: Summary | null;
  error: string | null;
}

const IDLE: JobState = { running: false, done: 0, total: 0, log: [], summary: null, error: null };

/** Runs one tool at a time and collects its live progress. */
export function useJob(job: JobId) {
  const [state, setState] = useState<JobState>(IDLE);

  useEffect(() => {
    const unlisten = api.onProgress((progress) => {
      if (progress.job !== job) return;
      setState((current) => ({
        ...current,
        done: progress.done,
        total: progress.total,
        log: [...current.log, progress.result],
      }));
    });
    return () => {
      unlisten.then((stop) => stop());
    };
  }, [job]);

  const start = useCallback(async (run: () => Promise<Summary>) => {
    setState({ ...IDLE, running: true });
    try {
      const summary = await run();
      setState((current) => ({ ...current, running: false, summary, total: summary.total }));
    } catch (error) {
      setState((current) => ({ ...current, running: false, error: String(error) }));
    }
  }, []);

  const cancel = useCallback(() => api.cancel(job), [job]);

  return { state, start, cancel };
}
