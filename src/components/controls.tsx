// Form controls used by the tool tabs.

import type { ReactNode } from "react";

/** One compact options row: a short label on the left, the control on the right. */
export function Field({ label, title, children }: { label: string; title?: string; children: ReactNode }) {
  return (
    <div className="field" title={title}>
      <span className="field-label">{label}</span>
      <div className="field-control">{children}</div>
    </div>
  );
}

export function Toggle({
  checked,
  onChange,
  label,
}: {
  checked: boolean;
  onChange: (value: boolean) => void;
  label: string;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      className={checked ? "toggle toggle-on" : "toggle"}
      onClick={() => onChange(!checked)}
    >
      <span className="toggle-knob" />
    </button>
  );
}

interface SegmentedProps<T extends string> {
  value: T;
  options: { value: T; label: string }[];
  onChange: (value: T) => void;
  disabled?: boolean;
}

export function Segmented<T extends string>({ value, options, onChange, disabled }: SegmentedProps<T>) {
  return (
    <div className="segmented">
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          aria-pressed={option.value === value}
          disabled={disabled}
          className={option.value === value ? "segment segment-active" : "segment"}
          onClick={() => onChange(option.value)}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

interface NumberFieldProps {
  value: number;
  onChange: (value: number) => void;
  min: number;
  max: number;
  step: number;
  unit: string;
  disabled?: boolean;
}

/** A number with − / + steppers; typed values are clamped. */
export function NumberField({ value, onChange, min, max, step, unit, disabled }: NumberFieldProps) {
  const set = (next: number) => onChange(Math.round(Math.min(max, Math.max(min, next)) * 10) / 10);
  return (
    <div className={disabled ? "number number-disabled" : "number"}>
      <button type="button" className="number-step" disabled={disabled} onClick={() => set(value - step)}>
        −
      </button>
      <input
        type="number"
        className="number-input"
        value={value}
        min={min}
        max={max}
        step={step}
        disabled={disabled}
        onChange={(event) => {
          const parsed = Number.parseFloat(event.target.value);
          if (Number.isFinite(parsed)) set(parsed);
        }}
      />
      <span className="number-unit">{unit}</span>
      <button type="button" className="number-step" disabled={disabled} onClick={() => set(value + step)}>
        +
      </button>
    </div>
  );
}

interface SelectProps<T extends string> {
  value: T;
  options: readonly { value: T; label: string }[];
  onChange: (value: T) => void;
  disabled?: boolean;
}

export function Select<T extends string>({ value, options, onChange, disabled }: SelectProps<T>) {
  return (
    <select
      className="select"
      value={value}
      disabled={disabled}
      onChange={(event) => onChange(event.target.value as T)}
    >
      {options.map((option) => (
        <option key={option.value} value={option.value}>
          {option.label}
        </option>
      ))}
    </select>
  );
}

export function TextInput({ value, onChange }: { value: string; onChange: (value: string) => void }) {
  return (
    <input className="text-input" value={value} spellCheck={false} onChange={(event) => onChange(event.target.value)} />
  );
}
