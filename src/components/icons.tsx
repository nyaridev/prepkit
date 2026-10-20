// PrepKit's own icon set: drawn on a 16×16 grid, 1.5 px strokes with round caps and joins, colored with
// currentColor. Small filled details (dots) use fill instead of stroke.

import type { ReactNode } from "react";

function Icon({ children, size = 16 }: { children: ReactNode; size?: number }) {
  return (
    <svg
      className="icon"
      width={size}
      height={size}
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {children}
    </svg>
  );
}

const Dot = ({ cx, cy, r = 0.9 }: { cx: number; cy: number; r?: number }) => (
  <circle cx={cx} cy={cy} r={r} fill="currentColor" stroke="none" />
);

export function LogoMark() {
  return (
    <svg className="icon" width="20" height="20" viewBox="0 0 32 32" aria-hidden="true">
      <defs>
        <linearGradient id="logo-fill" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor="var(--accent)" />
          <stop offset="1" stopColor="var(--accent-deep)" />
        </linearGradient>
      </defs>
      <rect width="32" height="32" rx="7" fill="url(#logo-fill)" />
      <path
        d="M11 23V9h6.2a4.6 4.6 0 0 1 0 9.2H11"
        fill="none"
        stroke="var(--accent-text)"
        strokeWidth="3"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <circle cx="23" cy="23" r="2.2" fill="var(--gold)" />
    </svg>
  );
}

/* Window controls: thin and quiet, like the system's. */

export const MinimizeIcon = () => (
  <Icon size={14}>
    <path d="M3.5 8h9" strokeWidth="1.25" />
  </Icon>
);

export const MaximizeIcon = () => (
  <Icon size={14}>
    <rect x="3.75" y="3.75" width="8.5" height="8.5" rx="1.75" strokeWidth="1.25" />
  </Icon>
);

export const CloseIcon = () => (
  <Icon size={14}>
    <path d="M4 4l8 8M12 4l-8 8" strokeWidth="1.25" />
  </Icon>
);

/* Tools. */

export const ImageIcon = () => (
  <Icon>
    <rect x="2" y="2.75" width="12" height="10.5" rx="2.25" />
    <Dot cx={5.75} cy={6.25} r={1.1} />
    <path d="M2.5 11.75l3.1-3.1a1 1 0 0 1 1.4 0l2.1 2.1M8.25 9.9l1.15-1.15a1 1 0 0 1 1.4 0l2.7 2.7" />
  </Icon>
);

export const WaveIcon = () => (
  <Icon>
    <path d="M2.75 6.75v2.5M5.375 4.75v6.5M8 2.75v10.5M10.625 5.25v5.5M13.25 7v2" />
  </Icon>
);

/* Files. */

export const FolderIcon = () => (
  <Icon>
    <path d="M2 5c0-.83.67-1.5 1.5-1.5h2.38c.4 0 .78.16 1.06.44l.62.62c.28.28.66.44 1.06.44h3.88c.83 0 1.5.67 1.5 1.5v5c0 .83-.67 1.5-1.5 1.5h-9A1.5 1.5 0 0 1 2 11.5z" />
  </Icon>
);

export const FileIcon = () => (
  <Icon>
    <path d="M4.5 2h4.1c.4 0 .78.16 1.06.44l2.4 2.4c.28.28.44.66.44 1.06v6.6c0 .83-.67 1.5-1.5 1.5H4.5A1.5 1.5 0 0 1 3 12.5v-9C3 2.67 3.67 2 4.5 2z" />
    <path d="M8.75 2.25V5c0 .41.34.75.75.75h2.75" />
  </Icon>
);

export const DropIcon = () => (
  <Icon size={30}>
    <path d="M8 9.75V2.75M5.25 5.25L8 2.5l2.75 2.75" />
    <path d="M2.75 9.5v1.75c0 1.1.9 2 2 2h6.5c1.1 0 2-.9 2-2V9.5" />
  </Icon>
);

export const ChevronIcon = () => (
  <Icon size={14}>
    <path d="M6 4l4 4-4 4" />
  </Icon>
);

export const ListIcon = () => (
  <Icon size={15}>
    <path d="M6 4.25h7.25M6 8h7.25M6 11.75h7.25" />
    <Dot cx={3} cy={4.25} />
    <Dot cx={3} cy={8} />
    <Dot cx={3} cy={11.75} />
  </Icon>
);

export const GridIcon = () => (
  <Icon size={15}>
    <rect x="2.5" y="2.5" width="4.5" height="4.5" rx="1.25" />
    <rect x="9" y="2.5" width="4.5" height="4.5" rx="1.25" />
    <rect x="2.5" y="9" width="4.5" height="4.5" rx="1.25" />
    <rect x="9" y="9" width="4.5" height="4.5" rx="1.25" />
  </Icon>
);

export const TrashIcon = () => (
  <Icon size={15}>
    <path d="M2.75 4.5h10.5M6.25 4.5v-1c0-.55.45-1 1-1h1.5c.55 0 1 .45 1 1v1" />
    <path d="M4.25 4.5l.6 7.8c.06.8.73 1.45 1.55 1.45h3.2c.82 0 1.49-.64 1.55-1.45l.6-7.8" />
  </Icon>
);

export const RemoveIcon = () => (
  <Icon size={12}>
    <path d="M4.5 4.5l7 7M11.5 4.5l-7 7" strokeWidth="1.75" />
  </Icon>
);

/* Results. */

export const CheckIcon = () => (
  <Icon size={14}>
    <path d="M3.5 8.5l2.75 2.75L12.5 5" strokeWidth="1.75" />
  </Icon>
);

export const SkipIcon = () => (
  <Icon size={14}>
    <circle cx="8" cy="8" r="5.25" />
    <path d="M5.75 8h4.5" />
  </Icon>
);

export const AlertIcon = () => (
  <Icon size={14}>
    <circle cx="8" cy="8" r="5.25" />
    <path d="M8 5.25v3.25" />
    <Dot cx={8} cy={10.75} r={0.85} />
  </Icon>
);

export const DownloadIcon = () => (
  <Icon size={14}>
    <path d="M8 2.75v7M5.25 7.25L8 10l2.75-2.75M3 13.25h10" />
  </Icon>
);
