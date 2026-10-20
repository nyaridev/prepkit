import { useState } from "react";
import type { FileGroup, GroupFile } from "../lib/groups";
import type { FileView } from "../lib/settings";
import { ChevronIcon, FileIcon, FolderIcon, RemoveIcon, WaveIcon } from "./icons";
import { Thumb } from "./Thumb";

interface FileGroupsProps {
  groups: FileGroup[];
  loading: boolean;
  kind: string;
  view: FileView;
  preview: boolean;
  disabled: boolean;
  onRemoveFile: (path: string) => void;
  onRemoveInput: (path: string) => void;
}

/** One collapsible section per added folder (and one for files added directly). */
export function FileGroups({ groups, loading, kind, view, preview, disabled, ...actions }: FileGroupsProps) {
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const toggle = (root: string) =>
    setCollapsed((current) => {
      const next = new Set(current);
      if (next.has(root)) next.delete(root);
      else next.add(root);
      return next;
    });

  return (
    <div className="groups">
      {groups.map((group) => {
        const open = !collapsed.has(group.root);
        return (
          <section key={group.root || "files"} className="group">
            <div className="group-header">
              <button
                type="button"
                className="group-toggle"
                aria-expanded={open}
                title={group.root || undefined}
                onClick={() => toggle(group.root)}
              >
                <span className={open ? "group-chevron group-chevron-open" : "group-chevron"}>
                  <ChevronIcon />
                </span>
                <span className="group-icon">{group.kind === "folder" ? <FolderIcon /> : <FileIcon />}</span>
                <span className="group-name">{group.name}</span>
                <span className="group-count">{loading ? "…" : group.files.length}</span>
              </button>
              {group.kind === "folder" && (
                <button
                  type="button"
                  className="icon-button icon-button-small"
                  aria-label={`Remove ${group.name}`}
                  title="Remove folder"
                  disabled={disabled}
                  onClick={() => actions.onRemoveInput(group.root)}
                >
                  <RemoveIcon />
                </button>
              )}
            </div>
            {open &&
              (group.files.length === 0 ? (
                <p className="group-empty">{loading ? "Reading…" : `No ${kind.toLowerCase()} in this folder.`}</p>
              ) : view === "grid" ? (
                <ul className="file-grid">
                  {group.files.map((file) => (
                    <Tile key={file.path} file={file} disabled={disabled} onRemove={actions.onRemoveFile} />
                  ))}
                </ul>
              ) : (
                <ul className="file-list">
                  {group.files.map((file) => (
                    <Row
                      key={file.path}
                      file={file}
                      preview={preview}
                      disabled={disabled}
                      onRemove={actions.onRemoveFile}
                    />
                  ))}
                </ul>
              ))}
          </section>
        );
      })}
    </div>
  );
}

interface ItemProps {
  file: GroupFile;
  disabled: boolean;
  onRemove: (path: string) => void;
}

function RemoveButton({ file, disabled, onRemove, className }: ItemProps & { className: string }) {
  return (
    <button
      type="button"
      className={className}
      aria-label={`Remove ${file.name}`}
      title="Remove"
      disabled={disabled}
      onClick={() => onRemove(file.path)}
    >
      <RemoveIcon />
    </button>
  );
}

function Row({ file, preview, ...item }: ItemProps & { preview: boolean }) {
  return (
    <li className="file-row" title={file.path}>
      {preview ? (
        <Thumb path={file.path} />
      ) : (
        <span className="file-icon">
          <WaveIcon />
        </span>
      )}
      <span className="file-name">{file.name}</span>
      <span className="file-sub">{file.subfolder}</span>
      <RemoveButton file={file} {...item} className="icon-button icon-button-small" />
    </li>
  );
}

function Tile({ file, ...item }: ItemProps) {
  return (
    <li className="file-tile" title={file.path}>
      <Thumb path={file.path} large />
      <span className="tile-name">{file.name}</span>
      {file.subfolder && <span className="tile-sub">{file.subfolder}</span>}
      <RemoveButton file={file} {...item} className="tile-remove" />
    </li>
  );
}
