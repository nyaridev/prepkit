import { useEffect, useState } from "react";
import { AudioTab } from "../features/audio-convert/AudioTab";
import { PixelFixTab } from "../features/pixel-fix/PixelFixTab";
import { api } from "../lib/api";
import { addUnique } from "../lib/paths";
import type { JobId } from "../lib/types";
import { Tabs } from "./Tabs";
import { TitleBar } from "./TitleBar";
import { useSettings } from "./useSettings";

export type SetInputs = (change: (paths: string[]) => string[]) => void;

export function App() {
  const { settings, update, flush } = useSettings();
  const [inputs, setInputs] = useState<Record<JobId, string[]>>({ "pixel-fix": [], "audio-convert": [] });
  const [dragging, setDragging] = useState(false);
  const tab = settings?.lastTab ?? "pixel-fix";

  // Files dropped anywhere on the window go to the open tab.
  useEffect(() => {
    const unlisten = api.onDrop((event) => {
      if (event.type === "drop") {
        setDragging(false);
        setInputs((current) => ({ ...current, [tab]: addUnique(current[tab], event.paths) }));
      } else {
        setDragging(event.type !== "leave");
      }
    });
    return () => {
      unlisten.then((stop) => stop());
    };
  }, [tab]);

  const setInputsFor =
    (job: JobId): SetInputs =>
    (change) =>
      setInputs((current) => ({ ...current, [job]: change(current[job]) }));

  const close = async () => {
    await flush();
    await api.window.close();
  };

  return (
    <div className="app">
      <TitleBar onClose={close} />
      <Tabs active={tab} onSelect={(next) => update((s) => ({ ...s, lastTab: next }))} />
      {settings && (
        <main className="content">
          <PixelFixTab
            hidden={tab !== "pixel-fix"}
            inputs={inputs["pixel-fix"]}
            setInputs={setInputsFor("pixel-fix")}
            dragging={dragging && tab === "pixel-fix"}
            settings={settings.pixelFix}
            onChange={(pixelFix) => update((s) => ({ ...s, pixelFix }))}
          />
          <AudioTab
            hidden={tab !== "audio-convert"}
            inputs={inputs["audio-convert"]}
            setInputs={setInputsFor("audio-convert")}
            dragging={dragging && tab === "audio-convert"}
            settings={settings.audio}
            onChange={(audio) => update((s) => ({ ...s, audio }))}
          />
        </main>
      )}
    </div>
  );
}
