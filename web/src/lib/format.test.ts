import { describe, expect, it } from "vitest";
import { bps, formatUnits, parseUnits, shortId } from "./format";

describe("exact base-unit formatting", () => {
  it("formats and parses without floating point", () => {
    expect(formatUnits(100_000_000)).toBe("1");
    expect(formatUnits(150_000_000)).toBe("1.5");
    expect(formatUnits(1)).toBe("0.00000001");
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
});
