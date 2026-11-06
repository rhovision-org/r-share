#!/usr/bin/env python3
"""Shared booking fixture for the lifecycle and claim suites.

The fixture creates a booking through the installed `component:booking-factory` and points
its Bond at a stub that accepts the single payout. It is a state-machine fixture: it does
not exercise slots or ERTP conservation.
"""

from __future__ import annotations

import json

from assertions import one, root_uri

DEFAULT_CANCELLATION = {
    "eventTransferCutoffBlocks": 0,
    "depositReviewBlocks": 0,
    "disputeWindowBlocks": 0,
    "noShowBlocks": 0,
    "fullRefundBeforeBlocks": 0,
    "partialRefundBps": 0,
    "partialRefundBeforeBlocks": 0,
    "feeBps": 0,
}


def dummy_bond() -> str:
    term = """
new Bond, rs(`rho:registry:insertArbitrary`), ch,
    deployId(`rho:rchain:deployId`) in {
  contract Bond(@"pay", @buckets, @expected, settleToken, ret) = { ret!((true, Nil)) } |
  contract Bond(@"balance", ret) = { ret!((true, 0)) } |
  rs!(bundle+{*Bond}, *ch) |
  for (@uri <- ch) { deployId!(uri) }
}
"""
    uri = one(term)
    if not isinstance(uri, str) or not uri.startswith("rho:id:"):
        raise AssertionError(f"dummy bond URI: {uri!r}")
    return uri


def fake_listing(owner: str, kind: str, listing_id: str = "listing:test") -> str:
    """A test-only listing that satisfies the production factory's origin check.

    It is not registered as a component. The lifecycle/claims/events fixtures use it so
    the production BookingContractFactory can keep its real verifyHold/info gate.
    """
    term = f"""
new FakeListing, rs(`rho:registry:insertArbitrary`), ch,
    deployId(`rho:rchain:deployId`) in {{
  contract FakeListing(@"verifyHold", @bookingId, @unitSlots, releaseToken, ret) = {{
    ret!((true, Nil))
  }} |
  contract FakeListing(@"info", ret) = {{
    ret!((true, {{"providerId": "{owner}", "listingId": "{listing_id}",
                   "kind": "{kind}", "status": "active"}}))
  }} |
  rs!(bundle+{{*FakeListing}}, *ch) |
  for (@uri <- ch) {{ deployId!(uri) }}
}}
"""
    uri = one(term)
    if not isinstance(uri, str) or not uri.startswith("rho:id:"):
        raise AssertionError(f"fake listing URI: {uri!r}")
    return uri


def booking_term(
    bond_uri: str,
    owner: str,
    instant_book: bool,
    call: str,
    *,
    kind: str = "lodging",
    cancellation: dict | None = None,
    check_in: int = 0,
    check_out: int = 1_000_000_000,
    rent: int = 100,
    deposit: int = 50,
) -> str:
    policy = dict(DEFAULT_CANCELLATION)
    if cancellation:
        policy.update(cancellation)
    data = {
        "bookingId": "booking:" + ("instant" if instant_book else "requested"),
        "guestId": owner,
        "providerId": owner,
        "listingId": "listing:test",
        "listingUri": fake_listing(owner, kind),
        "kind": kind,
        "status": "confirmed" if instant_book else "requested",
        "unitSlots": [],
        "assetId": "REV",
        "rent": rent,
        "deposit": deposit,
        "feeBps": policy["feeBps"],
        "holdUntilHeight": 1_000_000_000,
        "checkInHeight": check_in,
        "checkOutHeight": check_out,
        "cancellation": policy,
        "admins": [],
        "threshold": 1,
        "instantBook": instant_book,
        "cover": {
            "policyUri": "", "policyId": "", "offerId": "",
            "cap": 0, "deductible": 0, "effectiveDeposit": 0,
            "termsDeposit": 0, "mode": "none", "status": "none",
            "claimId": "", "lossId": "", "depositPart": 0,
            "coverPart": 0, "coverExpiresAtHeight": 0,
        },
    }
    return f"""
new rl(`rho:registry:lookup`), rootCh, rCh, bfCh, bCh,
    dId(`rho:rchain:deployerId`), releaseToken, settleToken,
    c1, c2, c3, createCh, out, deployId(`rho:rchain:deployId`) in {{
  rl!(`{root_uri()}`, *rootCh) |
  for (Root <- rootCh) {{
    Root!("resolve", "component:booking-factory", 1, *rCh) |
    for (@(true, uri, _) <- rCh) {{
      rl!(uri, *bfCh) |
      for (BookingFactory <- bfCh) {{
        BookingFactory!("create", *dId, {json.dumps(data)}, *releaseToken,
                        "{bond_uri}", *settleToken, *createCh) |
        for (@created <- createCh) {{
          match created {{
            (true, bookingUri) => {{
              rl!(bookingUri, *bCh) |
              for (B <- bCh) {{
                {call}
              }}
            }}
            _ => {{ deployId!(created) }}
          }}
        }}
      }}
    }}
  }}
}}
"""
