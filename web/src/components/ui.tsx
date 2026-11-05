import type { ReactNode } from "react";
import { formatUnits, shortId } from "../lib/format";
import type { Amount, ReadMeta, UnknownStatus } from "../lib/types";

export function Panel({ children, className = "" }: { children: ReactNode; className?: string }) {
  return <section className={`panel ${className}`}>{children}</section>;
}

export function Field({
  label,
  children,
  hint,
}: {
  label: string;
  children: ReactNode;
  hint?: string;
}) {
  return (
    <label className="field">
      <span>{label}</span>
      {children}
      {hint && <small className="muted">{hint}</small>}
    </label>
  );
}

export function Money({ value }: { value: Amount }) {
  return <span className="mono">{formatUnits(value)} REV</span>;
}

export function ReadBadge({ meta }: { meta?: ReadMeta }) {
  if (!meta) return null;
  return (
    <span className={`badge ${meta.source}`}>
      {meta.source === "explore" ? "free read" : "signed read"} · block{" "}
      {meta.atHeight.toString()}
    </span>
  );
}

export function statusText(status: string | UnknownStatus): string {
  return typeof status === "string" ? status : status.raw;
}

export function StatusPill({ status }: { status: string | UnknownStatus }) {
  const text = statusText(status);
  const tone =
    text === "active" || text === "confirmed" || text === "settled"
      ? "good"
      : text === "paused" || text === "requested" || text === "disputed"
        ? "warn"
        : text === "delisted" || text === "cancelled" || text === "expired"
          ? "bad"
          : "";
  return <span className={`pill ${tone}`}>{text}</span>;
}

export function ErrorBox({ error }: { error: unknown }) {
  if (!error) return null;
  return (
    <div className="error">
      {error instanceof Error ? error.message : String(error)}
    </div>
  );
}

export function CopyButton({ text, label = "Copy link" }: { text: string; label?: string }) {
  return (
    <button
      className="ghost"
      onClick={() => {
        void navigator.clipboard.writeText(text);
      }}
    >
      {label}
    </button>
  );
}

export function Uri({ value, label }: { value: string; label?: string }) {
  return (
    <span className="mono small" title={value}>
      {label ? `${label} ` : ""}
      {shortId(value, 10)}
    </span>
  );
}

export function Empty({ children }: { children: ReactNode }) {
  return <p className="muted empty">{children}</p>;
}
