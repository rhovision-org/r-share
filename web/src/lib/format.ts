// Exact formatting for protocol base units. No JS floating-point arithmetic on money.

import type { Amount } from "./types";

const BASE = 100_000_000n;

/** Format a base-unit count as a decimal amount. */
export function formatUnits(value: Amount, trim = true): string {
  const negative = value < 0n;
  const magnitude = negative ? -value : value;
  const whole = magnitude / BASE;
  const fraction = (magnitude % BASE).toString().padStart(8, "0");
  const text = trim ? fraction.replace(/0+$/, "") : fraction;
  const sign = negative ? "-" : "";
  return text ? `${sign}${whole}.${text}` : `${sign}${whole}`;
}

/** Parse a decimal string to base units, refusing more than eight fractional digits. */
export function parseUnits(value: string): bigint {
  const text = value.trim();
  if (!/^-?\d*(\.\d{0,8})?$/.test(text) || text === "" || text === ".") {
    throw new Error("amount must be a decimal with at most 8 fractional digits");
  }
  const negative = text.startsWith("-");
  const digits = negative ? text.slice(1) : text;
  const [whole = "0", fraction = ""] = digits.split(".");
  const scaled = BigInt(whole || "0") * BASE + BigInt((fraction + "00000000").slice(0, 8));
  return negative ? -scaled : scaled;
}

/**
 * Serialize a value for the WASM boundary. A bigint outside the safe range is refused,
 * not rounded; safe bigints become JSON integers for the Rust `i64` parameters.
 */
export function jsonValue(value: unknown): string {
  return JSON.stringify(value, (_key, item: unknown) => {
    if (typeof item === "bigint") {
      if (
        item > BigInt(Number.MAX_SAFE_INTEGER) ||
        item < BigInt(Number.MIN_SAFE_INTEGER)
      ) {
        throw new Error("integer argument is outside Number.MAX_SAFE_INTEGER");
      }
      return Number(item);
    }
    return item;
  });
}

/** Serialize WASM argument lists. */
export function jsonArgs(values: unknown[]): string {
  return jsonValue(values);
}

/** A short middle-truncated identifier. */
export function shortId(value: string, keep = 8): string {
  if (value.length <= keep * 2 + 1) return value;
  return `${value.slice(0, keep)}…${value.slice(-keep)}`;
}

/** Basis points as a percentage string. */
export function bps(value: number): string {
  return `${(value / 100).toFixed(2)}%`;
}

/** Format a block height as a decimal string. */
export function height(value: bigint): string {
  return value.toString();
}

export function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
