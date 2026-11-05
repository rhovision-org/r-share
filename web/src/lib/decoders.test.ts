import { describe, expect, it } from "vitest";
import { parseRead } from "./api";
import { DecodeError, chainInt } from "./decode";
import {
  guestOp,
  listingStatus,
  nonceView,
  probeStates,
  providerView,
} from "./decoders";

const provider = {
  providerId: "provider-1",
  profile: { name: "p" },
  status: "active",
  listings: {},
  admins: ["admin"],
  threshold: 1,
  nonce: 3,
};

describe("decoders", () => {
  it("decodes a valid view and preserves chain integers as bigint", () => {
    const decoded = providerView(provider, "$.provider");
    expect(decoded.providerId).toBe("provider-1");
    expect(decoded.nonce).toBe(3n);
    expect(decoded.threshold).toBe(1);
  });

  it("reports a missing required field with its path", () => {
    const { providerId: _omitted, ...missing } = provider;
    let caught: unknown;
    try {
      providerView(missing, "$.provider");
    } catch (error) {
      caught = error;
    }
    expect(caught).toBeInstanceOf(DecodeError);
    if (!(caught instanceof DecodeError)) throw new Error("expected DecodeError");
    expect(caught.path).toBe("$.provider.providerId");
    expect(caught.actual).toBe("missing");
  });

  it("rejects a wrong field type", () => {
    expect(() => providerView({ ...provider, nonce: "3.5" }, "$.provider")).toThrow(
      DecodeError,
    );
    expect(() => providerView({ ...provider, nonce: {} }, "$.provider")).toThrow(
      /expected decimal string or safe integer/,
    );
  });

  it("keeps an unknown status typed instead of a bare string", () => {
    expect(listingStatus("active")).toBe("active");
    expect(listingStatus("futureState")).toEqual({
      kind: "unknown",
      raw: "futureState",
    });
    const decoded = providerView({ ...provider, status: "futureStatus" });
    expect(decoded.status).toEqual({ kind: "unknown", raw: "futureStatus" });
  });

  it("accepts decimal strings and safe integers and refuses unsafe numbers", () => {
    expect(chainInt("42")).toBe(42n);
    expect(chainInt(-7)).toBe(-7n);
    expect(chainInt(Number.MAX_SAFE_INTEGER)).toBe(BigInt(Number.MAX_SAFE_INTEGER));
    expect(() => chainInt(Number.MAX_SAFE_INTEGER + 1)).toThrow(DecodeError);
    expect(() => chainInt("12a")).toThrow(DecodeError);
    expect(() => chainInt(1.5)).toThrow(DecodeError);
  });

  it("decodes guest op state, including absent ids", () => {
    expect(
      guestOp({ pending: true, ok: false, bookingId: null, uri: "rho:id:b" }),
    ).toEqual({ pending: true, ok: false, bookingId: null, uri: "rho:id:b" });
    expect(guestOp({ pending: false, ok: true })).toEqual({
      pending: false,
      ok: true,
      bookingId: null,
      uri: null,
    });
    expect(() => guestOp({ pending: "yes", ok: true })).toThrow(DecodeError);
  });

  it("decodes slot states by tag", () => {
    expect(
      probeStates({ unit: { s1: ["held", "booking", 12] } }, "$.probe"),
    ).toEqual({ unit: { s1: ["held", "booking", 12n] } });
    expect(probeStates({ unit: { s1: ["invalid"] } })).toEqual({
      unit: { s1: ["invalid"] },
    });
  });

  it("decodes the admin nonce read envelope", () => {
    const raw = JSON.stringify({
      ok: true,
      data: { current: "5", next: 6 },
      read: { atHeight: 9, blockHash: "block", source: "explore", deployId: null },
    });
    const envelope = parseRead(raw, nonceView);
    expect(envelope.data.current).toBe(5n);
    expect(envelope.data.next).toBe(6n);
    expect(envelope.read.atHeight).toBe(9n);
  });

  it("refuses a false read envelope", () => {
    const raw = JSON.stringify({
      ok: false,
      data: {},
      read: { atHeight: 9, blockHash: "block", source: "explore", deployId: null },
    });
    expect(() => parseRead(raw, nonceView)).toThrow(/expected true/);
  });
});
