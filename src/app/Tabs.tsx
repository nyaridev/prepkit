import type { ReactNode } from "react";
import { ImageIcon, WaveIcon } from "../components/icons";
import type { JobId } from "../lib/types";

const TABS: { id: JobId; label: string; icon: ReactNode }[] = [
  { id: "pixel-fix", label: "Pixel Fix", icon: <ImageIcon /> },
  { id: "audio-convert", label: "Audio Convert", icon: <WaveIcon /> },
];

export function Tabs({ active, onSelect }: { active: JobId; onSelect: (tab: JobId) => void }) {
  return (
    <div className="tabs" role="tablist">
      {TABS.map((tab) => (
        <button
          key={tab.id}
          type="button"
          role="tab"
          aria-selected={tab.id === active}
          className={tab.id === active ? "tab tab-active" : "tab"}
          onClick={() => onSelect(tab.id)}
        >
          {tab.icon}
          {tab.label}
        </button>
      ))}
    </div>
  );
}
