import { describe, expect, it } from "vitest";
import { defaultTerms, KINDS, TERM_FIELDS } from "./termsDefaults";

describe("terms defaults", () => {
  it("cover every kind with a valid starter policy", () => {
    for (const kind of KINDS) {
      const terms = defaultTerms(kind);
      expect(terms.kind).toBe(kind);
      expect(terms.pricePerSlot).toBeGreaterThan(0);
      expect(terms.minSlots).toBeLessThanOrEqual(terms.maxSlots);
    }
  });

  it("lists every non-identity field exactly once", () => {
    const keys = TERM_FIELDS.map((field) => field.key);
    expect(new Set(keys).size).toBe(keys.length);
  });
});
