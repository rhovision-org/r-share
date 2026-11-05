#!/usr/bin/env python3
"""Claim-class suite for the four implemented classes, across all five kinds.

Covered: no-fault cancellation, no-show, and the deposit claim path used by
`buyerUseBreach` and `damage`. `providerDeliveryBreach` and `partialPerformance` are
design-only and are not exercised.
"""

from __future__ import annotations

import json
from pathlib import Path

import actors
from assertions import expect, expect_ok, one
from booking_fixture import booking_term, dummy_bond
from preflight import require_devnet

import deploy as node

ROOT = Path(__file__).resolve().parent.parent


def vector_checks() -> None:
    vectors = json.loads((ROOT / "spec" / "vectors" / "core.json").read_text())["vectors"]
    remedy = actors.resolve("component:remedy")
    fee_input = [int(x) for x in vectors["remedy.fee"]["input"].split(",")]
    fee = actors.invoke(remedy, "fee", f"{fee_input[0]}, {fee_input[1]}")
    expect("vector remedy.fee", fee[1], int(vectors["remedy.fee"]["output"]))
    split_input = [int(x) for x in vectors["remedy.depositSplit.total"]["input"].split(",")]
    split = actors.invoke(remedy, "depositSplit",
                          f"{split_input[0]}, {split_input[1]}")
    total = split[1]["providerDamage"] + split[1]["guestRefund"]
    expect("vector remedy.depositSplit", total,
           int(vectors["remedy.depositSplit.total"]["output"]))
    full_input = [int(x) for x in vectors["remedy.full.total"]["input"].split(",")]
    policy = {"fullRefundBeforeBlocks": 0, "partialRefundBeforeBlocks": 0,
              "partialRefundBps": 0, "eventTransferCutoffBlocks": 0}
    cancelled = actors.invoke(
        remedy, "cancel",
        f'"lodging", {json.dumps(policy)}, 100, 100, {full_input[0]}, '
        f'{full_input[1]}, "guest"')
    total = sum(cancelled[1][key] for key in
                ("providerRent", "platformFee", "guestRefund", "providerDamage"))
    expect("vector remedy.full", total, int(vectors["remedy.full.total"]["output"]))

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

    vector_checks()
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
            cancellation={"noShowBlocks": 0, "disputeWindowBlocks": 10},
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
