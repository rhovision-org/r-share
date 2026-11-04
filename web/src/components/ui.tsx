import type { ReactNode } from "react";
import { formatUnits, shortId } from "../lib/format";
import type { ReadMeta } from "../lib/types";

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

export function Money({ value }: { value: number | string }) {
  return <span className="mono">{formatUnits(value)} REV</span>;
}

export function ReadBadge({ meta }: { meta?: ReadMeta }) {
  if (!meta) return null;
  return (
    <span className={`badge ${meta.source}`}>
      {meta.source === "explore" ? "free read" : "signed read"} · block {meta.atHeight}
    </span>
  );
}

export function StatusPill({ status }: { status: string }) {
  const tone =
    status === "active" || status === "confirmed" || status === "settled"
      ? "good"
      : status === "paused" || status === "requested" || status === "disputed"
        ? "warn"
        : status === "delisted" || status === "cancelled" || status === "expired"
          ? "bad"
          : "";
  return <span className={`pill ${tone}`}>{status}</span>;
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
