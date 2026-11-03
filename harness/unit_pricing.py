#!/usr/bin/env python3
"""Per-unit pricing suite: the allocator stores each unit's price and refuses zero."""

from __future__ import annotations

import json

import actors
from assertions import expect, expect_ok, expect_refusal
from market_fixture import Market, started, terms
from preflight import require_devnet


def listing_for(name: str, price: int) -> tuple[str, str]:
    now = actors.block_height()
    provider_book = actors.resolve("component:provider-book")
    registered = actors.invoke(
        provider_book, "register", '*dId, {"name": "pricing"}', key="business")
    expect_ok("provider", registered)
    factory = actors.resolve("component:listing-factory")
    created = actors.invoke(
        factory, "create",
        f'*dId, "{registered[1]}", "listing:{name}:{now}", "robot", '
        f"{json.dumps(terms(pricePerSlot=max(price, 1)))}",
        key="business")
    expect_ok("listing", created)
    return created[1], registered[1]


def unit_info(listing_uri: str, unit_id: str) -> dict:
    reply = actors.invoke(listing_uri, "unit", f'"{unit_id}"')
    expect_ok("unit info", reply)
    return reply[1]


def main() -> int:
    print("unit-pricing")
    if not require_devnet("unit-pricing"):
        return 2

    ten = started("pricing-ten", price=10)
    expect("unit A price", unit_info(ten.listing_uri, ten.unit_id)["pricePerSlot"], 10)

    twenty = started("pricing-twenty", price=20)
    expect("unit B price",
           unit_info(twenty.listing_uri, twenty.unit_id)["pricePerSlot"], 20)

    listing_uri, _ = listing_for("pricing-zero", 0)
    slots = [{"slotId": "s1", "startHeight": actors.block_height() + 50,
              "endHeight": actors.block_height() + 150}]
    bad = actors.invoke(
        listing_uri, "ensureUnit",
        f'*dId, "unit-zero", {{"name": "bad"}}, {json.dumps(slots)}, 0',
        key="business")
    expect_refusal("unit price zero", bad, "price")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
