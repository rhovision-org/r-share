#!/usr/bin/env python3
"""Reset safety regression: only free or expired-held slots may be reset."""

from __future__ import annotations

import actors
from assertions import expect, expect_ok, expect_refusal
from market_fixture import started
from preflight import require_devnet


def reset(market, key=None):
    return actors.invoke(
        market.listing_uri, "reset",
        f'*dId, "{market.unit_id}", "{market.slot_ids[0]}"',
        key=key or market.provider,
    )


def main() -> int:
    print("unit-reset-safety")
    if not require_devnet("unit-reset-safety"):
        return 2

    booked = started("reset-booked", instant_book=True)
    first = booked.book(booked.op("book"))
    expect_ok("booked booking", first)
    expect("slot booked", booked.slot_state()[0], "booked")
    expect_refusal("reset booked", reset(booked), "slot booked")
    expect("booked slot unchanged", booked.slot_state()[0], "booked")
    expect_ok("cancel booked", booked.cancel(first[2], booked.op("cancel")))
    expect("slot free", booked.slot_state()[0], "free")
    expect_ok("reset free", reset(booked))

    held = started("reset-held", instant_book=False)
    pending = held.book(held.op("book"))
    expect_ok("held booking", pending)
    expect("slot held", held.slot_state()[0], "held")
    expect_refusal("reset live held", reset(held), "slot held")
    expect("held slot unchanged", held.slot_state()[0], "held")
    expect_ok("cancel held", held.cancel(pending[2], held.op("cancel")))

    expired = started("reset-expired", instant_book=False, holdBlocks=1)
    old = expired.book(expired.op("book"))
    expect_ok("expired-hold booking", old)
    info = expired.booking_info(old[2])
    actors.wait_height(info["holdUntilHeight"] + 1)
    expect_ok("reset expired held", reset(expired))
    expect("expired slot free", expired.slot_state()[0], "free")

    wrong = started("reset-wrong", instant_book=True)
    expect_ok("wrong-owner booking", wrong.book(wrong.op("book")))
    expect_refusal("wrong owner reset", reset(wrong, wrong.guest), "not owner")
    expect("wrong owner left slot booked", wrong.slot_state()[0], "booked")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
