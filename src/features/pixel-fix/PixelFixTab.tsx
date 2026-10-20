import type { SetInputs } from "../../app/App";
import { useFileList } from "../../app/useFileList";
import { useJob } from "../../app/useJob";
import { Field, Segmented, TextInput, Toggle } from "../../components/controls";
import { FilePanel } from "../../components/FilePanel";
import { ResultsPanel } from "../../components/ResultsPanel";
import { RunButton } from "../../components/RunButton";
import { api } from "../../lib/api";
import { addUnique } from "../../lib/paths";
import { cleanName, IMAGE_EXTENSIONS, pixelFixOptions, type Settings } from "../../lib/settings";
import type { OutputMode } from "../../lib/types";

type PixelSettings = Settings["pixelFix"];

interface PixelFixTabProps {
  hidden: boolean;
  inputs: string[];
  setInputs: SetInputs;
  dragging: boolean;
  settings: PixelSettings;
  onChange: (settings: PixelSettings) => void;
}

const OUTPUTS: { value: OutputMode; label: string }[] = [
  { value: "replace", label: "Replace" },
  { value: "folder", label: "Folder" },
  { value: "prefix", label: "Prefix" },
];

function example(settings: PixelSettings): string {
  if (settings.output === "folder") return `coin.png → ${settings.folder || "fixed"}/coin.png`;
  if (settings.output === "prefix") return `coin.png → ${settings.prefix || "fixed_"}coin.png`;
  return "coin.png is overwritten";
}

export function PixelFixTab({ hidden, inputs, setInputs, dragging, settings, onChange }: PixelFixTabProps) {
  const job = useJob("pixel-fix");
  const options = pixelFixOptions(settings);
  const list = useFileList(inputs, (roots) => api.listImages(roots, options), JSON.stringify(options));
  const set = (patch: Partial<PixelSettings>) => onChange({ ...settings, ...patch });
  const count = list.files.length;

  return (
    <div className="tab-page" hidden={hidden}>
      <FilePanel
        title="Images"
        inputs={inputs}
        files={list.files}
        loading={list.loading}
        onAdd={(paths) => setInputs((current) => addUnique(current, paths))}
        onRemoveFile={list.remove}
        onRemoveInput={(path) => setInputs((current) => current.filter((input) => input !== path))}
        onClear={() => setInputs(() => [])}
        dragging={dragging}
        disabled={job.state.running}
        kind="Images"
        extensions={IMAGE_EXTENSIONS}
        preview={{ view: settings.view, onViewChange: (view) => set({ view }) }}
      />
      <div className="side">
        <section className="options">
          <h2 className="section-label options-head">Pixel Fix</h2>
          <Field label="Save to">
            <Segmented value={settings.output} options={OUTPUTS} onChange={(output) => set({ output })} />
          </Field>
          {settings.output === "folder" && (
            <Field label="Folder name">
              <TextInput value={settings.folder} onChange={(folder) => set({ folder: cleanName(folder) })} />
            </Field>
          )}
          {settings.output === "prefix" && (
            <Field label="Prefix">
              <TextInput value={settings.prefix} onChange={(prefix) => set({ prefix: cleanName(prefix) })} />
            </Field>
          )}
          <Field label="Subfolders" title="Also list and fix images inside sub-folders">
            <Toggle
              label="Include subfolders"
              checked={settings.recursive}
              onChange={(recursive) => set({ recursive })}
            />
          </Field>
          <p className="options-hint">{example(settings)}</p>
        </section>
        <ResultsPanel
          state={job.state}
          action={
            <RunButton
              running={job.state.running}
              canStart={count > 0}
              label={count > 0 ? `Fix ${count} image${count === 1 ? "" : "s"}` : "Fix images"}
              onStart={() => job.start(() => api.runPixelFix(list.files, options))}
              onCancel={job.cancel}
            />
          }
        />
      </div>
    </div>
  );
}
