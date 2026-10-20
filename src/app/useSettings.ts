import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "../lib/api";
import { mergeSettings, type Settings } from "../lib/settings";

const SAVE_DELAY_MS = 300;

/** Loads the saved settings once, and saves every change shortly after it happens. `flush` saves right away. */
export function useSettings() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const latest = useRef<Settings | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  useEffect(() => {
    let active = true;
    api
      .loadSettings()
      .catch(() => null)
      .then((raw) => {
        if (!active) return;
        const loaded = mergeSettings(raw);
        latest.current = loaded;
        setSettings(loaded);
      });
    return () => {
      active = false;
    };
  }, []);

  const flush = useCallback(async () => {
    clearTimeout(timer.current);
    if (latest.current) await api.saveSettings(latest.current).catch(() => {});
  }, []);

  // A reload or an unexpected close still saves the last change.
  useEffect(() => {
    const save = () => void flush();
    window.addEventListener("pagehide", save);
    return () => window.removeEventListener("pagehide", save);
  }, [flush]);

  const update = useCallback(
    (change: (current: Settings) => Settings) => {
      if (!latest.current) return;
      const next = change(latest.current);
      latest.current = next;
      setSettings(next);
      clearTimeout(timer.current);
      timer.current = setTimeout(flush, SAVE_DELAY_MS);
    },
    [flush],
  );

  return { settings, update, flush };
}
