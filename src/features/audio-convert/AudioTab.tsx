import type { SetInputs } from "../../app/App";
import { useFileList } from "../../app/useFileList";
import { useJob } from "../../app/useJob";
import { Field, NumberField, Segmented, Select, TextInput, Toggle } from "../../components/controls";
import { FilePanel } from "../../components/FilePanel";
import { ResultsPanel } from "../../components/ResultsPanel";
import { RunButton } from "../../components/RunButton";
import { api } from "../../lib/api";
import { addUnique } from "../../lib/paths";
import {
  AUDIO_EXTENSIONS,
  AUDIO_FORMATS,
  audioOptions,
  cleanName,
  type Settings,
  TARGET_RANGE,
} from "../../lib/settings";
import { FfmpegBanner, FfmpegPill } from "./FfmpegStatus";
import { useFfmpeg } from "./useFfmpeg";

type AudioSettings = Settings["audio"];

interface AudioTabProps {
  hidden: boolean;
  inputs: string[];
  setInputs: SetInputs;
  dragging: boolean;
  settings: AudioSettings;
  onChange: (settings: AudioSettings) => void;
}

const FORMAT_OPTIONS = [
  { value: "keep" as const, label: "Keep original" },
  ...AUDIO_FORMATS.map((format) => ({ value: format, label: `.${format}` })),
];

export function AudioTab({ hidden, inputs, setInputs, dragging, settings, onChange }: AudioTabProps) {
  const job = useJob("audio-convert");
  const ffmpeg = useFfmpeg();
  const list = useFileList(inputs, (roots) => api.listAudio(roots), "audio");
  const set = (patch: Partial<AudioSettings>) => onChange({ ...settings, ...patch });
  const peak = settings.mode === "peak";
  const ready = ffmpeg.state.kind === "found";
  const nothingToDo = !settings.normalize && settings.format === "keep";
  const extension = settings.format === "keep" ? "wav" : settings.format;
  const count = list.files.length;

  return (
    <div className="tab-page" hidden={hidden}>
      <FilePanel
        title="Audio"
        inputs={inputs}
        files={list.files}
        loading={list.loading}
        onAdd={(paths) => setInputs((current) => addUnique(current, paths))}
        onRemoveFile={list.remove}
        onRemoveInput={(path) => setInputs((current) => current.filter((input) => input !== path))}
        onClear={() => setInputs(() => [])}
        dragging={dragging}
        disabled={job.state.running}
        kind="Audio"
        extensions={AUDIO_EXTENSIONS}
      />
      <div className="side">
        <section className="options">
          <div className="options-head">
            <h2 className="section-label">Audio Convert</h2>
            <FfmpegPill state={ffmpeg.state} />
          </div>
          <Field label="Normalize">
            <Toggle
              label="Normalize volume"
              checked={settings.normalize}
              onChange={(normalize) => set({ normalize })}
            />
          </Field>
          <Field
            label="Mode"
            title={
              peak
                ? "The loudest sample reaches the target"
                : "Average loudness reaches the target; peaks stay under -1 dBFS"
            }
          >
            <Segmented
              value={settings.mode}
              disabled={!settings.normalize}
              options={[
                { value: "rms", label: "RMS" },
                { value: "peak", label: "Peak" },
              ]}
              onChange={(mode) => set({ mode })}
            />
          </Field>
          <Field label="Target">
            <NumberField
              value={peak ? settings.targetPeak : settings.targetRms}
              onChange={(value) => set(peak ? { targetPeak: value } : { targetRms: value })}
              min={TARGET_RANGE.min}
              max={TARGET_RANGE.max}
              step={0.5}
              unit="dBFS"
              disabled={!settings.normalize}
            />
          </Field>
          <Field label="Format">
            <Select value={settings.format} options={FORMAT_OPTIONS} onChange={(format) => set({ format })} />
          </Field>
          <Field label="Suffix">
            <TextInput value={settings.suffix} onChange={(suffix) => set({ suffix: cleanName(suffix) })} />
          </Field>
          <p className="options-hint">
            jump.wav → normalized/jump{settings.suffix}.{extension} · existing files are skipped
          </p>
          <FfmpegBanner state={ffmpeg.state} onInstall={ffmpeg.install} />
        </section>
        <ResultsPanel
          state={job.state}
          action={
            <RunButton
              running={job.state.running}
              canStart={count > 0 && ready && !nothingToDo}
              label={count > 0 ? `Convert ${count} file${count === 1 ? "" : "s"}` : "Convert audio"}
              blockedReason={
                !ready ? "Audio needs ffmpeg first" : nothingToDo ? "Turn on Normalize or pick a format" : undefined
              }
              onStart={() => job.start(() => api.runAudioConvert(list.files, audioOptions(settings)))}
              onCancel={job.cancel}
            />
          }
        />
      </div>
    </div>
  );
}
