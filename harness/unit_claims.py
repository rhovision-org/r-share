#!/usr/bin/env python3
"""Claim-class suite for the four implemented classes, across all five kinds.

Covered: no-fault cancellation, no-show, and the deposit claim path used by
`buyerUseBreach` and `damage`. `providerDeliveryBreach` and `partialPerformance` are
design-only and are not exercised.
"""

from __future__ import annotations

from assertions import expect_ok, one
from booking_fixture import booking_term, dummy_bond
from preflight import require_devnet

import deploy as node

KINDS = ("lodging", "event", "bike", "car", "robot")


def cancel_call() -> str:
    return (
        'B!("cancel", *dId, "opC", *out) | '
        'for (@r <- out) { deployId!(r) }'
    )


def no_show_call() -> str:
    return (
        'B!("claimNoShow", *dId, "opN", *out) | '
        'for (@r <- out) { deployId!(r) }'
    )


def damage_call() -> str:
    return (
        'B!("checkIn", *dId, "op1", *c1) | '
        'for (@r1 <- c1) { match r1 { '
        '  (true, Nil) => { '
        '    B!("checkOut", *dId, "op2", *c2) | '
        '    for (@r2 <- c2) { match r2 { '
        '      (true, Nil) => { '
        '        B!("claimDeposit", *dId, "op3", 20, "hash", *c3) | '
        '        for (@r3 <- c3) { match r3 { '
        '          (true, Nil) => { B!("resolveDeposit", "op4", *out) | '
        '                           for (@r <- out) { deployId!(r) } } '
        '          _ => { deployId!(("claimDeposit", r3)) } '
        '        } } '
        '      } '
        '      _ => { deployId!(("checkOut", r2)) } '
        '    } } '
        '  } '
        '  _ => { deployId!(("checkIn", r1)) } '
        '} }'
    )


def main() -> int:
    print("unit-claims")
    if not require_devnet("unit-claims"):
        return 2

    owner = node.load_keys()["deployer"]["address"]
    bond = dummy_bond()
    print(f"  bond {bond}")

    for kind in KINDS:
        cancelled = one(booking_term(
            bond, owner, False, cancel_call(), kind=kind,
            cancellation={"fullRefundBeforeBlocks": 0},
            check_in=1_000_000_000, check_out=1_000_000_001,
        ))
        expect_ok(f"{kind} noFaultCancellation", cancelled)

        no_show = one(booking_term(
            bond, owner, True, no_show_call(), kind=kind,
            cancellation={"noShowBlocks": 0},
            check_in=0,
        ))
        expect_ok(f"{kind} noShow", no_show)

        damage = one(booking_term(
            bond, owner, True, damage_call(), kind=kind,
            cancellation={"depositReviewBlocks": 1000, "disputeWindowBlocks": 0},
            check_in=0,
        ))
        expect_ok(f"{kind} buyerUseBreach/damage", damage)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
