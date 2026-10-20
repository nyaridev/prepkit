import { useEffect, useRef, useState } from "react";

/**
 * The files a tool will process: the added files and folders expanded by the backend, minus the ones the user
 * removed. `listKey` changes whenever the listing options change (e.g. "include subfolders").
 */
export function useFileList(roots: string[], list: (roots: string[]) => Promise<string[]>, listKey: string) {
  const [files, setFiles] = useState<string[]>([]);
  const [removed, setRemoved] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState(false);
  const listRef = useRef(list);
  listRef.current = list;

  // biome-ignore lint/correctness/useExhaustiveDependencies: listKey stands in for the listing options.
  useEffect(() => {
    if (roots.length === 0) {
      setFiles([]);
      setRemoved(new Set());
      return;
    }
    let active = true;
    setLoading(true);
    listRef.current(roots).then((found) => {
      if (!active) return;
      setFiles(found);
      setLoading(false);
    });
    return () => {
      active = false;
    };
  }, [roots, listKey]);

  return {
    files: files.filter((file) => !removed.has(file)),
    loading,
    remove: (file: string) => setRemoved((current) => new Set(current).add(file)),
  };
}
