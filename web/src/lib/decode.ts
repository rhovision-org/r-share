// Composable runtime decoders for the JSON boundary between the WASM facade and the UI.
// Every decoder either returns a typed value or throws a path-aware DecodeError. There is
// no unchecked assertion anywhere in this module.

import type { UnknownStatus } from "./types";

/** A path-aware decoding failure. `path` names the offending field, `$` is the root. */
export class DecodeError extends Error {
  readonly path: string;
  readonly expected: string;
  readonly actual: string;

  constructor(path: string, expected: string, actual: string) {
    super(`${path}: expected ${expected}, got ${actual}`);
    this.name = "DecodeError";
    this.path = path;
    this.expected = expected;
    this.actual = actual;
  }
}

export type Decoder<T> = (value: unknown, path?: string) => T;

function actualOf(value: unknown): string {
  if (value === null) return "null";
  if (Array.isArray(value)) return "array";
  if (typeof value === "undefined") return "undefined";
  if (typeof value === "string") {
    const shown = value.length > 60 ? `${value.slice(0, 57)}...` : value;
    return `string ${JSON.stringify(shown)}`;
  }
  if (typeof value === "number") return `number ${String(value)}`;
  if (typeof value === "bigint") return `bigint ${value.toString()}`;
  if (typeof value === "boolean") return `boolean ${String(value)}`;
  return typeof value;
}

/** Decode a JSON object into a plain record without asserting its fields. */
export const object: Decoder<Record<string, unknown>> = (value, path = "$") => {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new DecodeError(path, "object", actualOf(value));
  }
  const out: Record<string, unknown> = {};
  for (const [key, item] of Object.entries(value)) out[key] = item;
  return out;
};

/** Read a required field from an already-decoded object. */
export function required<T>(
  source: Record<string, unknown>,
  key: string,
  decoder: Decoder<T>,
  path = "$",
): T {
  const fieldPath = `${path}.${key}`;
  if (!Object.prototype.hasOwnProperty.call(source, key) || source[key] === undefined) {
    throw new DecodeError(fieldPath, "a value", "missing");
  }
  return decoder(source[key], fieldPath);
}

/** Read an optional field. `null` and `undefined` both mean absent. */
export function optional<T>(
  source: Record<string, unknown>,
  key: string,
  decoder: Decoder<T>,
  path = "$",
): T | undefined {
  const value = source[key];
  if (value === undefined || value === null) return undefined;
  return decoder(value, `${path}.${key}`);
}

export const string: Decoder<string> = (value, path = "$") => {
  if (typeof value !== "string") throw new DecodeError(path, "string", actualOf(value));
  return value;
};

export const boolean: Decoder<boolean> = (value, path = "$") => {
  if (typeof value !== "boolean") throw new DecodeError(path, "boolean", actualOf(value));
  return value;
};

/** A free-form JSON value (already parsed, so any JSON shape is acceptable). */
export const unknownValue: Decoder<unknown> = (value) => value;

export function array<T>(decoder: Decoder<T>): Decoder<T[]> {
  return (value, path = "$") => {
    if (!Array.isArray(value)) throw new DecodeError(path, "array", actualOf(value));
    return value.map((item, index) => decoder(item, `${path}[${index}]`));
  };
}

export function record<T>(decoder: Decoder<T>): Decoder<Record<string, T>> {
  return (value, path = "$") => {
    const raw = object(value, path);
    const out: Record<string, T> = {};
    for (const [key, item] of Object.entries(raw)) {
      out[key] = decoder(item, `${path}.${key}`);
    }
    return out;
  };
}

export interface IntOptions {
  min?: number;
  max?: number;
}

/** A bounded JavaScript integer. Counts and basis points stay `number`. */
export function int(options: IntOptions = {}): Decoder<number> {
  const { min, max } = options;
  const expected =
    min !== undefined && max !== undefined
      ? `integer between ${String(min)} and ${String(max)}`
      : min !== undefined
        ? `integer >= ${String(min)}`
        : max !== undefined
          ? `integer <= ${String(max)}`
          : "safe integer";
  return (value, path = "$") => {
    if (typeof value !== "number" || !Number.isInteger(value) || !Number.isSafeInteger(value)) {
      throw new DecodeError(path, expected, actualOf(value));
    }
    if (min !== undefined && value < min) throw new DecodeError(path, expected, actualOf(value));
    if (max !== undefined && value > max) throw new DecodeError(path, expected, actualOf(value));
    return value;
  };
}

const DECIMAL = /^-?\d+$/;

/**
 * A chain integer. Accepts a decimal string or a safe JSON integer and returns `bigint`.
 * Unsafe numbers and malformed strings are refused rather than silently rounded.
 */
export const chainInt: Decoder<bigint> = (value, path = "$") => {
  if (typeof value === "bigint") return value;
  if (typeof value === "number") {
    if (!Number.isSafeInteger(value)) {
      throw new DecodeError(path, "safe integer or decimal string", actualOf(value));
    }
    return BigInt(value);
  }
  if (typeof value === "string") {
    if (!DECIMAL.test(value)) {
      throw new DecodeError(path, "decimal string", actualOf(value));
    }
    return BigInt(value);
  }
  throw new DecodeError(path, "decimal string or safe integer", actualOf(value));
};

/** Decode a string that must be one of `values`. */
export function enumOf<T extends string>(...values: T[]): Decoder<T> {
  return (value, path = "$") => {
    const text = string(value, path);
    for (const candidate of values) {
      if (candidate === text) return candidate;
    }
    throw new DecodeError(path, `one of ${values.join(", ")}`, actualOf(value));
  };
}

/**
 * Decode a string that may be one of `values`. Unknown values are preserved as a typed
 * `UnknownStatus` so a newer chain state is displayed, not mistaken for a known one.
 */
export function unknownEnum<T extends string>(
  ...values: T[]
): Decoder<T | UnknownStatus> {
  return (value, path = "$") => {
    const text = string(value, path);
    for (const candidate of values) {
      if (candidate === text) return candidate;
    }
    return { kind: "unknown", raw: text };
  };
}

/** Parse JSON text and decode it. Invalid JSON is reported as a DecodeError. */
export function parseJson<T>(text: string, decoder: Decoder<T>, path = "$"): T {
  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch {
    throw new DecodeError(path, "JSON", "invalid JSON");
  }
  return decoder(value, path);
}
