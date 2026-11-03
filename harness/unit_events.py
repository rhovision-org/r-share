#!/usr/bin/env python3
"""Event suite: redeem, closeEvent and transferTicket on event bookings."""

from __future__ import annotations

from assertions import expect_ok, one
from booking_fixture import booking_term, dummy_bond
from preflight import require_devnet
from unit_booking import block_height

import deploy as node


def redeem_and_close(bond: str, owner: str, now: int) -> tuple:
    return one(booking_term(
        bond, owner, True,
        'B!("redeem", *dId, "opR", *c1) | '
        'for (@r1 <- c1) { match r1 { '
        '  (true, Nil) => { B!("closeEvent", "opC", *out) | '
        '                   for (@r <- out) { deployId!(r) } } '
        '  _ => { deployId!(("redeem", r1)) } '
        '} }',
        kind="event", deposit=0, check_out=0,
        cancellation={"eventTransferCutoffBlocks": 0},
    ))


def transfer(bond: str, owner: str, new_guest: str, now: int) -> tuple:
    return one(booking_term(
        bond, owner, True,
        f'B!("transferTicket", *dId, "{new_guest}", "opT", *c1) | '
        'for (@r1 <- c1) { match r1 { '
        '  (true, Nil) => { B!("ticket", *out) | '
        '                   for (@r <- out) { deployId!(r) } } '
        '  _ => { deployId!(("transfer", r1)) } '
        '} }',
        kind="event", deposit=0, check_out=now + 1000,
        cancellation={"eventTransferCutoffBlocks": 0},
    ))


def main() -> int:
    print("unit-events")
    if not require_devnet("unit-events"):
        return 2

    keys = node.load_keys()
    owner = keys["deployer"]["address"]
    new_guest = keys["admin1"]["address"]
    bond = dummy_bond()
    now = block_height()

    closed = redeem_and_close(bond, owner, now)
    expect_ok("redeem -> closeEvent", closed)

    moved = transfer(bond, owner, new_guest, now)
    expect_ok("transfer ticket", moved)
    if moved[1].get("holderId") != new_guest:
        raise AssertionError(f"ticket holder: {moved!r}")
    print("  ok ticket moved to the new holder")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
