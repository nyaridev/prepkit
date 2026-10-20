// Groups the listed files by what the user added: one group per added folder, plus one for files added directly.

export interface FileGroup {
  /** The added folder, or "" for the files added directly. */
  root: string;
  kind: "folder" | "files";
  name: string;
  files: GroupFile[];
}

export interface GroupFile {
  path: string;
  name: string;
  /** Sub-folder inside the group's folder ("" when the file sits directly in it). */
  subfolder: string;
}

const key = (path: string) => path.replace(/\//g, "\\").replace(/\\+$/, "").toLowerCase();

function lastPart(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

/**
 * `inputs` are the files and folders the user added (in order); `files` what the backend listed for them.
 * An input that appears in `files` itself is a file; anything else is a folder. A file under two added folders
 * goes to the first one.
 */
export function groupFiles(inputs: string[], files: string[]): FileGroup[] {
  const listed = new Map(files.map((file) => [key(file), file]));
  const placed = new Set<string>();
  const groups: FileGroup[] = [];
  const loose: GroupFile[] = [];

  for (const input of inputs) {
    const inputKey = key(input);
    const asFile = listed.get(inputKey);
    if (asFile) {
      if (!placed.has(inputKey)) loose.push({ path: asFile, name: lastPart(asFile), subfolder: "" });
      placed.add(inputKey);
      continue;
    }
    const prefix = `${inputKey}\\`;
    const members: GroupFile[] = [];
    for (const file of files) {
      const fileKey = key(file);
      if (placed.has(fileKey) || !fileKey.startsWith(prefix)) continue;
      placed.add(fileKey);
      const relative = file.replace(/\//g, "\\").slice(prefix.length);
      const cut = relative.lastIndexOf("\\");
      members.push({ path: file, name: lastPart(file), subfolder: cut > 0 ? relative.slice(0, cut) : "" });
    }
    groups.push({ root: input, kind: "folder", name: lastPart(input), files: members });
  }

  if (loose.length > 0) groups.push({ root: "", kind: "files", name: "Files", files: loose });
  return groups;
}
