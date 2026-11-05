#!/usr/bin/env python3
"""Booking-factory origin regression (F-009): a direct caller has no reservation."""

from __future__ import annotations

import json

import actors
from assertions import expect_ok, expect_refusal, one
from booking_fixture import dummy_bond
from preflight import require_devnet

import deploy as node


def false_listing(owner: str) -> str:
    term = f"""
new FakeListing, rs(`rho:registry:insertArbitrary`), ch,
    deployId(`rho:rchain:deployId`) in {{
  contract FakeListing(@"verifyHold", @bookingId, @unitSlots, releaseToken, ret) = {{
    ret!((false, "slot lost"))
  }} |
  contract FakeListing(@"info", ret) = {{
    ret!((true, {{"providerId": "{owner}", "listingId": "listing:test",
                   "kind": "lodging", "status": "active"}}))
  }} |
  rs!(bundle+{{*FakeListing}}, *ch) |
  for (@uri <- ch) {{ deployId!(uri) }}
}}
"""
    uri = one(term)
    if not isinstance(uri, str) or not uri.startswith("rho:id:"):
        raise AssertionError(f"false listing URI: {uri!r}")
    return uri


def main() -> int:
    print("unit-factory-auth")
    if not require_devnet("unit-factory-auth"):
        return 2

    owner = node.load_keys()["deployer"]["address"]
    factory = actors.resolve("component:booking-factory")
    bond = dummy_bond()
    listing_uri = false_listing(owner)
    data = {
        "bookingId": "booking:fake",
        "guestId": owner,
        "providerId": owner,
        "listingId": "listing:test",
        "listingUri": listing_uri,
        "kind": "lodging",
        "status": "confirmed",
        "unitSlots": [],
        "assetId": "REV",
        "rent": 100,
        "deposit": 50,
        "feeBps": 0,
        "holdUntilHeight": 1_000_000_000,
        "checkInHeight": 0,
        "checkOutHeight": 1_000_000_000,
        "cancellation": {"eventTransferCutoffBlocks": 0,
                         "depositReviewBlocks": 0,
                         "disputeWindowBlocks": 10,
                         "noShowBlocks": 0,
                         "fullRefundBeforeBlocks": 0,
                         "partialRefundBps": 0,
                         "partialRefundBeforeBlocks": 0,
                         "feeBps": 0},
        "admins": [],
        "threshold": 2,
        "instantBook": True,
    }
    term = f"""
new rl(`rho:registry:lookup`), fCh, ret, releaseToken, settleToken,
    dId(`rho:rchain:deployerId`), deployId(`rho:rchain:deployId`) in {{
  rl!(`{factory}`, *fCh) |
  for (F <- fCh) {{
    F!("create", *dId, {json.dumps(data)}, *releaseToken, "{bond}",
       *settleToken, *ret) |
    for (@r <- ret) {{ deployId!(r) }}
  }}
}}
"""
    expect_refusal("direct factory without reservation", one(term), "reservation")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
