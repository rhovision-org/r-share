#!/usr/bin/env python3
"""Cell liveness and failure compensation regression (F-002/F-003)."""

from __future__ import annotations

import json

import actors
from assertions import expect, expect_refusal
from faults import (fake_bond_factory, fake_guest_ledger_factory,
                    restore_component, rotate_component)
from market_fixture import started
from preflight import require_devnet


def main() -> int:
    print("unit-cell-liveness")
    if not require_devnet("unit-cell-liveness"):
        return 2

    guest_key = "key21"
    actors.ensure_vault(guest_key)
    guest_book = actors.resolve("component:guest-book")
    original = rotate_component("component:guest-ledger-factory",
                                fake_guest_ledger_factory())
    try:
        expect_refusal("GuestBook factory refusal",
                       actors.invoke(guest_book, "getOrCreate", "*dId",
                                     key=guest_key),
                       "unavailable")
        expect_refusal("GuestBook remains live",
                       actors.invoke(guest_book, "getOrCreate", "*dId",
                                     key=guest_key),
                       "unavailable")
    finally:
        restore_component("component:guest-ledger-factory", *original)

    market = started("fault-bond", instant_book=True)
    original = rotate_component("component:bond-factory", fake_bond_factory())
    try:
        refused = actors.invoke(
            market.ledger_uri, "book",
            f'*dId, "{market.listing_uri}", {json.dumps(market.unit_slots())}, '
            f'"{market.op("book")}", Nil',
            key=market.guest)
        expect_refusal("Bond factory unavailable", refused,
                       "unavailable")
        pending = [op for op, entry in market.ops().items()
                   if isinstance(entry, dict) and entry.get("pending") is True]
        expect("no pending op after Bond failure", pending, [])
        expect("slot released after Bond failure",
               market.slot_state()[0], "free")
    finally:
        restore_component("component:bond-factory", *original)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
