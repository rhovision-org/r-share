#!/usr/bin/env python3
"""Safe profile: plain escrow only, cover explicitly disabled."""

from __future__ import annotations

import json
import time

import actors
import market_fixture
from assertions import expect, expect_ok, expect_refusal, root_uri
from preflight import require_devnet


def main() -> int:
    print("unit-cover-disabled")
    if not require_devnet("unit-cover-disabled"):
        return 2
    if not actors.require_profile("safe"):
        print("unit-cover-disabled: not the safe profile")
        return 2

    for key in ("component:cover-terms", "component:cover-book",
                "component:cover-pool", "component:cover-policy-factory",
                "component:guest-ledger-cover"):
        expect_refusal(f"absent {key}", actors.invoke(
            root_uri(), "resolve", f'"{key}", 1'), "unknown component")

    market = market_fixture.Market("safedefault").setup()
    booked = market.book(market.op("safe-book"))
    info = market.booking_info(booked[2])
    expect("plain deposit", info.get("deposit"), market.deposit)
    expect("plain cover mode", info.get("cover").get("mode"), "none")
    expect("plain cover status", info.get("cover").get("status"), "none")
    bond = actors.bond_info(info.get("bondUri"))
    expect("plain bond expected", bond.get("expected"),
           info.get("rent") + market.deposit)
    market.cancel(booked[2], market.op("safe-cancel"))

    ledger = market.ledger_uri
    op = market.op("disabled-cover")
    expect_refusal("bookWithCover is disabled", actors.invoke(
        ledger, "bookWithCover",
        f'*dId, "{market.listing_uri}", {json.dumps(market.unit_slots())}, '
        f'"{op}", Nil, "missing", "policy:missing"', key=market.guest),
        "cover disabled")

    # The disabled attempt drops its ledger op; the same operation id can book normally.
    booked2 = market.book(op)
    expect_ok("book after disabled cover", booked2)
    info2 = market.booking_info(booked2[2])
    expect("fallback plain deposit", info2.get("deposit"), market.deposit)
    expect("fallback cover none", info2.get("cover").get("status"), "none")
    print("unit-cover-disabled: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
