#!/usr/bin/env python3
"""Event settlement timing regression (F-005)."""

from __future__ import annotations

import actors
from assertions import expect_ok, expect_refusal, one
from booking_fixture import booking_term, dummy_bond
from preflight import require_devnet

import deploy as node


def redeem_then_settle(bond: str, owner: str, check_out: int) -> tuple:
    return one(booking_term(
        bond, owner, True,
        'B!("redeem", *dId, "opR", *c1) | '
        'for (@r1 <- c1) { match r1 { '
        '  (true, Nil) => { B!("settle", "opS", *out) | '
        '                   for (@r <- out) { deployId!(r) } } '
        '  _ => { deployId!(("redeem", r1)) } '
        '} }',
        kind="event", deposit=0, check_out=check_out,
        cancellation={"eventTransferCutoffBlocks": 0},
    ))


def main() -> int:
    print("unit-event-timing")
    if not require_devnet("unit-event-timing"):
        return 2

    owner = node.load_keys()["deployer"]["address"]
    bond = dummy_bond()
    now = actors.block_height()
    expect_refusal("settle before event end",
                   redeem_then_settle(bond, owner, now + 1000), "too early")

    later = actors.block_height()
    check_out = later + 1
    actors.wait_height(check_out + 1)
    expect_ok("settle after event end",
              redeem_then_settle(bond, owner, check_out))

    early_close = one(booking_term(
        bond, owner, True,
        'B!("closeEvent", "opC", *out) | '
        'for (@r <- out) { deployId!(r) }',
        kind="event", deposit=0, check_out=actors.block_height() + 1000,
        cancellation={"eventTransferCutoffBlocks": 0},
    ))
    expect_refusal("closeEvent before event end", early_close, "too early")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
