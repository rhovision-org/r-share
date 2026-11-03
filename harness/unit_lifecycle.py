#!/usr/bin/env python3
"""Lifecycle suite: the dispute trap, the arbitration gate and the default resolution.

The booking factory is the installed `component:booking-factory`. A stub Bond accepts the
single payout, so the test can reach `resolveDispute` without a full listing/unit setup.
The deployer is both guest and provider; this is a unit test of the state machine, not a
marketplace scenario.
"""

from __future__ import annotations

import json
import sys

from assertions import expect_ok, expect_refusal, one, root_uri
from preflight import require_devnet

import deploy as node


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


def booking_term(bond_uri: str, owner: str, instant_book: bool, call: str) -> str:
    cancellation = {
        "eventTransferCutoffBlocks": 0,
        "depositReviewBlocks": 0,
        "disputeWindowBlocks": 0,
        "noShowBlocks": 0,
        "fullRefundBeforeBlocks": 0,
        "partialRefundBps": 0,
        "partialRefundBeforeBlocks": 0,
        "feeBps": 0,
    }
    data = {
        "bookingId": "booking:" + ("instant" if instant_book else "requested"),
        "guestId": owner,
        "providerId": owner,
        "listingId": "listing:test",
        "listingUri": "",
        "kind": "lodging",
        "status": "confirmed" if instant_book else "requested",
        "unitSlots": [],
        "assetId": "REV",
        "rent": 100,
        "deposit": 50,
        "feeBps": 0,
        "holdUntilHeight": 1000000000,
        "checkInHeight": 0,
        "checkOutHeight": 1000000000,
        "cancellation": cancellation,
        "admins": [],
        "threshold": 1,
        "instantBook": instant_book,
    }
    # Rholang map literal with JSON strings; JSON and Rholang string escaping agree here.
    data_literal = json.dumps(data)
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
        BookingFactory!("create", *dId, {data_literal}, *releaseToken,
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


def main() -> int:
    print("unit-lifecycle")
    if not require_devnet("unit-lifecycle"):
        return 2

    owner = node.load_keys()["deployer"]["address"]
    bond = dummy_bond()
    print(f"  bond {bond}")

    refused = one(booking_term(
        bond, owner, False,
        'B!("dispute", *dId, "opD", "hash", *out) | '
        'for (@r <- out) { deployId!(r) }',
    ))
    expect_refusal("dispute from requested", refused, "closed")

    arbitrate = one(booking_term(
        bond, owner, True,
        'B!("arbitrate", 1, 0, [], "opA", *out) | '
        'for (@r <- out) { deployId!(r) }',
    ))
    expect_refusal("arbitrate outside dispute", arbitrate, "closed")

    resolved = one(booking_term(
        bond, owner, True,
        'B!("checkIn", *dId, "op1", *c1) | '
        'for (@r1 <- c1) { match r1 { '
        '  (true, Nil) => { '
        '    B!("checkOut", *dId, "op2", *c2) | '
        '    for (@r2 <- c2) { match r2 { '
        '      (true, Nil) => { '
        '        B!("dispute", *dId, "op3", "hash", *c3) | '
        '        for (@r3 <- c3) { match r3 { '
        '          (true, Nil) => { B!("resolveDispute", "op4", *out) | '
        '                           for (@r <- out) { deployId!(r) } } '
        '          _ => { deployId!(("dispute", r3)) } '
        '        } } '
        '      } '
        '      _ => { deployId!(("checkOut", r2)) } '
        '    } } '
        '  } '
        '  _ => { deployId!(("checkIn", r1)) } '
        '} }',
    ))
    expect_ok("dispute -> resolveDispute", resolved)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
