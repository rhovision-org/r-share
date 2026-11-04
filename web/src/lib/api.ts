// Small helpers around the wasm JSON boundary.

import type { ReadEnvelope, WriteEnvelope } from "./types";

export function parseRead<T>(raw: string): ReadEnvelope<T> {
  const value = JSON.parse(raw) as ReadEnvelope<T>;
  if (!value.ok) throw new Error("read failed");
  return value;
}

export function parseWrite<T>(raw: string): WriteEnvelope<T> {
  const value = JSON.parse(raw) as WriteEnvelope<T>;
  if (!value.ok) throw new Error("write failed");
  return value;
}

export function parseError(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return String(error);
}

export function jsonArgs(values: unknown[]): string {
  return JSON.stringify(values);
}

export function copyToClipboard(text: string): Promise<void> {
  return navigator.clipboard.writeText(text);
}

export function shareUrl(kind: "listing" | "booking" | "provider", uri: string): string {
  return `${window.location.origin}${window.location.pathname}#/${kind}/${uri}`;
}
