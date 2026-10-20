interface RunButtonProps {
  running: boolean;
  canStart: boolean;
  label: string;
  /** Why it can't start yet, shown as a tooltip. */
  blockedReason?: string;
  onStart: () => void;
  onCancel: () => void;
}

/** The tool's main action, at the bottom of its options. It turns into Cancel while the tool runs. */
export function RunButton({ running, canStart, label, blockedReason, onStart, onCancel }: RunButtonProps) {
  if (running) {
    return (
      <button type="button" className="button button-danger button-wide run-button" onClick={onCancel}>
        Cancel
      </button>
    );
  }
  return (
    <button
      type="button"
      className="button button-primary button-wide run-button"
      disabled={!canStart}
      title={blockedReason}
      onClick={onStart}
    >
      {label}
    </button>
  );
}
