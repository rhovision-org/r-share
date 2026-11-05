#!/usr/bin/env python3
"""Terminal settlement releases the booked slots (F-021 companion)."""

from __future__ import annotations

import actors
from assertions import expect, expect_ok
from market_fixture import started
from preflight import require_devnet


def main() -> int:
    print("unit-settle-release")
    if not require_devnet("unit-settle-release"):
        return 2

    settled = started("release-settle", instant_book=True)
    booked = settled.book(settled.op("book"))
    expect_ok("settle booking", booked)
    uri = booked[2]
    expect("slot booked before settle", settled.slot_state()[0], "booked")
    settled.wait_in()
    settled.check_in(uri, settled.op("in"))
    settled.check_out(uri, settled.op("out"))
    settled.settle(uri, settled.op("settle"))
    expect("settled", settled.booking_info(uri)["status"], "settled")
    expect("slot free after settle", settled.slot_state()[0], "free")

    deposit = started("release-deposit", instant_book=True,
                      depositReviewBlocks=100, disputeWindowBlocks=0)
    booked = deposit.book(deposit.op("book"))
    expect_ok("deposit booking", booked)
    uri = booked[2]
    deposit.wait_in()
    deposit.check_in(uri, deposit.op("in"))
    deposit.check_out(uri, deposit.op("out"))
    deposit.claim_deposit(uri, deposit.op("claim"), 20)
    deposit.resolve_deposit(uri, deposit.op("resolve"))
    expect("slot free after deposit resolution",
           deposit.slot_state()[0], "free")

    event = started("release-event", kind="event", instant_book=True, deposit=0)
    booked = event.book(event.op("book"))
    expect_ok("event booking", booked)
    uri = booked[2]
    expect_ok("event redeem", actors.invoke(
        uri, "redeem", '*dId, "redeem"', key=event.provider))
    check_out = event.booking_info(uri)["checkOutHeight"]
    actors.wait_height(check_out + 1)
    expect_ok("event close", actors.invoke(uri, "closeEvent", '"close"'))
    expect("slot free after closeEvent", event.slot_state()[0], "free")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
