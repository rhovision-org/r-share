#!/usr/bin/env python3
"""No-show contest window regression (F-004)."""

from __future__ import annotations

import actors
from assertions import expect, expect_ok, expect_refusal
from market_fixture import started
from preflight import require_devnet


def main() -> int:
    print("unit-no-show-contest")
    if not require_devnet("unit-no-show-contest"):
        return 2

    market = started("noshow-contest", instant_book=True,
                     noShowBlocks=0, disputeWindowBlocks=10)
    before = actors.balance(market.guest)
    booked = market.book(market.op("book"))
    expect_ok("book", booked)
    uri = booked[2]
    market.wait_in()
    expect_ok("claim no-show", market.claim_no_show(uri, market.op("claim")))
    info = market.booking_info(uri)
    expect("claimed status", info["status"], "noShowClaimed")
    expect("no payout on claim", actors.balance(market.guest), before - 150)
    expect_ok("guest dispute", actors.invoke(
        uri, "disputeNoShow", f'*dId, "{market.op("dispute")}", "hash"',
        key=market.guest))
    info = market.booking_info(uri)
    expect("disputed status", info["status"], "noShowDisputed")
    expect_refusal("early resolve", actors.invoke(
        uri, "resolveNoShow", f'"{market.op("early")}"'), "too early")
    actors.wait_height(info["disputeAtHeight"] + 11)
    expect_ok("resolve disputed", actors.invoke(
        uri, "resolveNoShow", f'"{market.op("resolve")}"'))
    info = market.booking_info(uri)
    expect("refunded status", info["status"], "noShowRefunded")
    expect("full refund", actors.balance(market.guest), before)
    expect("slot free after refund", market.slot_state()[0], "free")

    uncontested = started("noshow-provider", instant_book=True,
                          noShowBlocks=0, disputeWindowBlocks=10)
    booked = uncontested.book(uncontested.op("book"))
    expect_ok("provider-path book", booked)
    uri = booked[2]
    uncontested.wait_in()
    expect_ok("provider-path claim", uncontested.claim_no_show(
        uri, uncontested.op("claim")))
    info = uncontested.booking_info(uri)
    actors.wait_height(info["noShowClaimAtHeight"] + 11)
    expect_ok("provider-path resolve", actors.invoke(
        uri, "resolveNoShow", f'"{uncontested.op("resolve")}"'))
    expect("provider-path status",
           uncontested.booking_info(uri)["status"], "noShow")
    expect("provider-path slot free", uncontested.slot_state()[0], "free")

    zero = started("noshow-zero", instant_book=True,
                   noShowBlocks=0, disputeWindowBlocks=0)
    booked = zero.book(zero.op("book"))
    expect_ok("zero-window book", booked)
    zero.wait_in()
    expect_refusal("zero-window claim", actors.invoke(
        booked[2], "claimNoShow", f'*dId, "{zero.op("claim")}"',
        key=zero.provider), "window")
    expect("zero-window stays confirmed",
           zero.booking_info(booked[2])["status"], "confirmed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
