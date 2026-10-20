import { CloseIcon, LogoMark, MaximizeIcon, MinimizeIcon } from "../components/icons";
import { api } from "../lib/api";

/** The custom title bar: drag anywhere on it, double-click to maximize. */
export function TitleBar({ onClose }: { onClose: () => void }) {
  return (
    <header className="titlebar" data-tauri-drag-region>
      <div className="titlebar-brand">
        <LogoMark />
        <span className="titlebar-name">PrepKit</span>
        <span className="titlebar-version">v{__APP_VERSION__}</span>
      </div>
      <div className="window-controls">
        <button type="button" className="window-button" aria-label="Minimize" onClick={() => api.window.minimize()}>
          <MinimizeIcon />
        </button>
        <button
          type="button"
          className="window-button"
          aria-label="Maximize"
          onClick={() => api.window.toggleMaximize()}
        >
          <MaximizeIcon />
        </button>
        <button type="button" className="window-button window-close" aria-label="Close" onClick={onClose}>
          <CloseIcon />
        </button>
      </div>
    </header>
  );
}
