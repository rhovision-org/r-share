#!/usr/bin/env python3
"""Generality suite: every advertised kind validates; an unknown kind refuses."""

from __future__ import annotations

import json

from assertions import expect_ok, expect_refusal, one, root_uri
from preflight import require_devnet
from unit_booking import call_uri

KINDS = ("lodging", "event", "bike", "car", "robot")


def main() -> int:
    print("unit-generality")
    if not require_devnet("unit-generality"):
        return 2

    resolved = one(call_uri(root_uri(), "resolve", '"component:terms", 1'))
    if not (isinstance(resolved, tuple) and resolved[0] is True):
        raise AssertionError(f"resolve terms: {resolved!r}")
    terms_uri = resolved[1]
    terms = {
        "assetId": "REV",
        "pricePerSlot": 100,
        "deposit": 50,
        "minSlots": 1,
        "maxSlots": 4,
        "instantBook": True,
        "contiguous": False,
        "holdBlocks": 10,
        "depositReviewBlocks": 0,
        "disputeWindowBlocks": 0,
        "noShowBlocks": 0,
        "fullRefundBeforeBlocks": 0,
        "partialRefundBps": 0,
        "partialRefundBeforeBlocks": 0,
        "feeBps": 0,
        "maxLeadBlocks": 1000000,
        "maxSlotBlocks": 1000,
        "maxUnits": 2,
        "eventTransferCutoffBlocks": 0,
    }
    for kind in KINDS:
        reply = one(call_uri(
            terms_uri, "validate", f'"{kind}", {json.dumps(terms)}'))
        expect_ok(f"{kind} terms", reply)

    unknown = one(call_uri(
        terms_uri, "validate", f'"drone", {json.dumps(terms)}'))
    expect_refusal("unknown kind", unknown, "kind")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
