#!/usr/bin/env python3
"""Listing suite: create, unit, activate, pause/resume, delist and read projections."""

from __future__ import annotations

import json

from assertions import expect, expect_ok, one, root_uri
from preflight import require_devnet
from unit_booking import block_height, call_uri

import deploy as node


def resolve(key: str) -> str:
    reply = one(call_uri(root_uri(), "resolve", f'"{key}", 1'))
    if not (isinstance(reply, tuple) and reply[0] is True):
        raise AssertionError(f"resolve {key}: {reply!r}")
    return reply[1]


def main() -> int:
    print("unit-listing")
    if not require_devnet("unit-listing"):
        return 2

    now = block_height()
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

    book = resolve("component:provider-book")
    registered = one(call_uri(book, "register", '*dId, {"name": "Listing"}'))
    expect_ok("provider register", registered)
    provider_uri = registered[1]

    factory = resolve("component:listing-factory")
    listing_id = f"listing:suite:{now}"
    created = one(call_uri(
        factory, "create",
        f'*dId, "{provider_uri}", "{listing_id}", "robot", {json.dumps(terms)}'))
    expect_ok("listing create", created)
    listing_uri = created[1]

    info = one(call_uri(listing_uri, "info"))
    expect_ok("listing info", info)
    expect("draft status", info[1].get("status"), "draft")

    slots = [{"slotId": "s1", "startHeight": now + 100, "endHeight": now + 200}]
    unit = one(call_uri(
        listing_uri, "ensureUnit",
        f'*dId, "listing-unit:{now}", {{"name": "machine"}}, {json.dumps(slots)}, 10'))
    expect_ok("ensure unit", unit)
    units = one(call_uri(listing_uri, "units"))
    expect("unit registered", units[0], True)

    expect_ok("activate", one(call_uri(listing_uri, "activate", "*dId")))
    expect("active status", one(call_uri(listing_uri, "info"))[1].get("status"), "active")

    expect_ok("pause listing", one(call_uri(listing_uri, "pause", "*dId")))
    expect("paused status", one(call_uri(listing_uri, "info"))[1].get("status"), "paused")
    expect_ok("resume listing", one(call_uri(listing_uri, "resume", "*dId")))
    expect("resumed status", one(call_uri(listing_uri, "info"))[1].get("status"), "active")

    read_terms = one(call_uri(listing_uri, "terms"))
    expect_ok("read terms", read_terms)
    expect("price", read_terms[1].get("pricePerSlot"), 100)

    expect_ok("delist", one(call_uri(listing_uri, "delist", "*dId")))
    expect("delisted status", one(call_uri(listing_uri, "info"))[1].get("status"), "delisted")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
