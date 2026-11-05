// Small helpers around the wasm JSON boundary. Reads and writes are decoded with an
// explicit decoder; there is no generic assertion over the parsed JSON.

import { parseJson, type Decoder } from "./decode";
import { readEnvelope, writeEnvelope } from "./decoders";
import type { ReadEnvelope, WriteEnvelope } from "./types";

export function parseRead<T>(raw: string, decoder: Decoder<T>): ReadEnvelope<T> {
  return parseJson(raw, readEnvelope(decoder), "$read");
}

export function parseWrite<T>(raw: string, decoder: Decoder<T>): WriteEnvelope<T> {
  return parseJson(raw, writeEnvelope(decoder), "$write");
}

export function parseError(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return String(error);
}

export function copyToClipboard(text: string): Promise<void> {
  return navigator.clipboard.writeText(text);
}

export function shareUrl(kind: "listing" | "booking" | "provider", uri: string): string {
  return `${window.location.origin}${window.location.pathname}#/${kind}/${uri}`;
}
