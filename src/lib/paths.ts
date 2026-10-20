/** The last part of a Windows or POSIX path. */
export function fileName(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

/** Everything before the last part. */
export function parentPath(path: string): string {
  const index = Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/"));
  return index > 0 ? path.slice(0, index) : "";
}

/** Adds paths that aren't in the list yet, keeping the order they came in. */
export function addUnique(list: string[], added: string[]): string[] {
  const seen = new Set(list.map((path) => path.toLowerCase()));
  const result = [...list];
  for (const path of added) {
    const key = path.toLowerCase();
    if (!seen.has(key)) {
      seen.add(key);
      result.push(path);
    }
  }
  return result;
}
