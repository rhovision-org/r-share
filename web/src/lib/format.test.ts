import { describe, expect, it } from "vitest";
import { bps, formatUnits, jsonArgs, jsonValue, parseUnits, shortId } from "./format";

describe("exact base-unit formatting", () => {
  it("formats and parses without floating point", () => {
    expect(formatUnits(100_000_000n)).toBe("1");
    expect(formatUnits(150_000_000n)).toBe("1.5");
    expect(formatUnits(1n)).toBe("0.00000001");
    expect(parseUnits("1.5")).toBe(150_000_000n);
    expect(parseUnits("0.00000001")).toBe(1n);
  });

  it("refuses more than eight fractional digits", () => {
    expect(() => parseUnits("0.000000001")).toThrow();
    expect(() => parseUnits("abc")).toThrow();
  });

  it("formats bps and short ids", () => {
    expect(bps(250)).toBe("2.50%");
    expect(shortId("abcdefghijklmnop", 4)).toBe("abcd…mnop");
  });

  it("serializes safe bigint arguments and refuses unsafe ones", () => {
    expect(jsonArgs([1n, "x"])).toBe('[1,"x"]');
    expect(jsonValue({ price: 100n })).toBe('{"price":100}');
    expect(() => jsonArgs([BigInt(Number.MAX_SAFE_INTEGER) + 1n])).toThrow();
    expect(() => jsonArgs([BigInt(Number.MIN_SAFE_INTEGER) - 1n])).toThrow();
  });
});
