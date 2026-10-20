import { api } from "../lib/api";
import { groupFiles } from "../lib/groups";
import type { FileView } from "../lib/settings";
import { FileGroups } from "./FileGroups";
import { DropIcon, FileIcon, FolderIcon, GridIcon, ListIcon, TrashIcon } from "./icons";

export interface FilePanelProps {
  title: string;
  /** What the user added (files and folders). */
  inputs: string[];
  /** What a run would process, from the backend. */
  files: string[];
  loading: boolean;
  onAdd: (paths: string[]) => void;
  onRemoveFile: (path: string) => void;
  onRemoveInput: (path: string) => void;
  onClear: () => void;
  dragging: boolean;
  disabled: boolean;
  /** For the file picker filter, e.g. "Images". */
  kind: string;
  extensions: readonly string[];
  /** Image previews, with a list/grid switch. */
  preview?: { view: FileView; onViewChange: (view: FileView) => void };
}

/** What a tool will process, grouped by added folder. Drop files or folders on the window, or pick them. */
export function FilePanel(props: FilePanelProps) {
  const { title, inputs, files, loading, onAdd, onClear, dragging, disabled, kind, extensions, preview } = props;
  const pickFiles = async () => onAdd(await api.pickFiles(kind, extensions));
  const pickFolders = async () => onAdd(await api.pickFolders());

  return (
    <section className={dragging ? "files files-drop" : "files"}>
      <header className="files-header">
        <span className="files-title">{title}</span>
        {files.length > 0 && <span className="files-count">{files.length}</span>}
        <div className="files-tools">
          {preview && (
            <div className="view-toggle">
              <button
                type="button"
                className="icon-button"
                aria-label="List view"
                title="List"
                aria-pressed={preview.view === "list"}
                onClick={() => preview.onViewChange("list")}
              >
                <ListIcon />
              </button>
              <button
                type="button"
                className="icon-button"
                aria-label="Grid view"
                title="Grid"
                aria-pressed={preview.view === "grid"}
                onClick={() => preview.onViewChange("grid")}
              >
                <GridIcon />
              </button>
            </div>
          )}
          <button type="button" className="button button-small" disabled={disabled} onClick={pickFolders}>
            <FolderIcon />
            Folder
          </button>
          <button type="button" className="button button-small" disabled={disabled} onClick={pickFiles}>
            <FileIcon />
            Files
          </button>
          {inputs.length > 0 && (
            <button
              type="button"
              className="icon-button"
              aria-label="Clear all"
              title="Clear all"
              disabled={disabled}
              onClick={onClear}
            >
              <TrashIcon />
            </button>
          )}
        </div>
      </header>
      <div className="files-body">
        {inputs.length === 0 ? (
          <button type="button" className="files-empty" disabled={disabled} onClick={pickFiles}>
            <DropIcon />
            <span className="files-empty-title">Drop files or folders here</span>
            <span>or click to choose files</span>
            <span className="files-empty-types">{extensions.join("  ·  ")}</span>
          </button>
        ) : (
          <FileGroups
            groups={groupFiles(inputs, files)}
            loading={loading}
            kind={kind}
            view={preview?.view ?? "list"}
            preview={Boolean(preview)}
            disabled={disabled}
            onRemoveFile={props.onRemoveFile}
            onRemoveInput={props.onRemoveInput}
          />
        )}
      </div>
    </section>
  );
}
