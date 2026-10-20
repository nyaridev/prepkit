import { type ReactNode, useEffect, useRef } from "react";
import type { JobState } from "../app/useJob";
import { fileName } from "../lib/paths";
import type { FileResult } from "../lib/types";
import { AlertIcon, CheckIcon, SkipIcon } from "./icons";

const STATUS_ICON = { written: <CheckIcon />, skipped: <SkipIcon />, failed: <AlertIcon /> };

/** Progress, the counts and the per-file log of the latest run, with the tool's run button at the bottom. */
export function ResultsPanel({ state, action }: { state: JobState; action: ReactNode }) {
  const percent = state.total > 0 ? Math.round((state.done / state.total) * 100) : 0;
  const active = state.running || state.summary !== null;

  return (
    <section className="results">
      <div className="results-head">
        <h2 className="section-label">Results</h2>
        <Counts state={state} />
      </div>
      {active && <progress className="progress" max={100} value={percent} />}
      {state.error && <p className="results-error">{state.error}</p>}
      <LogList entries={state.log} />
      {action}
    </section>
  );
}

function Counts({ state }: { state: JobState }) {
  if (state.running) {
    return <span className="results-counts">{state.total > 0 ? `${state.done} / ${state.total}` : "Starting…"}</span>;
  }
  const summary = state.summary;
  if (!summary) return null;
  if (summary.total === 0) return <span className="results-counts">Nothing to do</span>;
  return (
    <span className="results-counts">
      <span className="count count-written">{summary.written}</span>
      <span className="count count-skipped">{summary.skipped}</span>
      <span className="count count-failed">{summary.failed}</span>
      {summary.cancelled && <span>cancelled</span>}
    </span>
  );
}

function LogList({ entries }: { entries: FileResult[] }) {
  const end = useRef<HTMLLIElement>(null);
  useEffect(() => {
    if (entries.length > 0) end.current?.scrollIntoView({ block: "end" });
  }, [entries.length]);

  if (entries.length === 0) return <div className="log log-empty">Results show up here</div>;
  return (
    <ul className="log">
      {entries.map((entry) => (
        <li key={entry.path} className={`log-item log-${entry.status}`} title={entry.output ?? entry.path}>
          <span className="log-icon">{STATUS_ICON[entry.status]}</span>
          <span className="log-name">{fileName(entry.output ?? entry.path)}</span>
          <span className="log-message">{entry.message}</span>
        </li>
      ))}
      <li ref={end} className="log-end" />
    </ul>
  );
}
