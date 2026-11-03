#!/usr/bin/env python3
"""Same-block race: two guests (the same deployer key, two opIds) contend for one slot.

Exactly one book may succeed. The loser must refuse with a slot reason; it must not create
a second booking against the same slot.
"""

from __future__ import annotations

import json

from assertions import decode, expect_ok, one, root_uri
from booking_fixture import dummy_bond  # noqa: F401  (kept for fixture parity)
from preflight import require_devnet
from unit_booking import call_uri, fund_account, block_height

import deploy as node


def resolve_component(key: str) -> str:
    reply = one(call_uri(root_uri(), "resolve", f'"{key}", 1'))
    if not (isinstance(reply, tuple) and reply[0] is True):
        raise AssertionError(f"resolve {key}: {reply!r}")
    return reply[1]


def book_term(ledger_uri: str, listing_uri: str, unit_uri: str, op_id: str) -> str:
    unit_slots = [{"unitUri": unit_uri, "slots": ["s1"]}]
    return call_uri(
        ledger_uri, "book",
        f'*dId, "{listing_uri}", {json.dumps(unit_slots)}, "{op_id}", Nil')


def unwrap(result):
    if isinstance(result, list) and len(result) == 1:
        return result[0]
    return result


def main() -> int:
    print("unit-concurrency")
    if not require_devnet("unit-concurrency"):
        return 2

    now = block_height()
    print(f"  height {now}")
    terms = {
        "assetId": "REV",
        "pricePerSlot": 100,
        "deposit": 50,
        "minSlots": 1,
        "maxSlots": 4,
        "instantBook": True,
        "contiguous": False,
        "holdBlocks": 10,
        "depositReviewBlocks": 0,
        "disputeWindowBlocks": 0,
        "noShowBlocks": 0,
        "fullRefundBeforeBlocks": 0,
        "partialRefundBps": 0,
        "partialRefundBeforeBlocks": 0,
        "feeBps": 0,
        "maxLeadBlocks": 1000000,
        "maxSlotBlocks": 1000,
        "maxUnits": 1,
        "eventTransferCutoffBlocks": 0,
    }
    keys = node.load_keys()
    owner = keys["deployer"]["address"]

    provider_book = resolve_component("component:provider-book")
    registered = one(call_uri(provider_book, "register", '*dId, {"name": "Race"}'))
    expect_ok("provider register", registered)
    provider_uri = registered[1]

    factory = resolve_component("component:listing-factory")
    listing_id = f"race:{now}"
    created = one(call_uri(
        factory, "create",
        f'*dId, "{provider_uri}", "{listing_id}", "robot", {json.dumps(terms)}'))
    expect_ok("listing create", created)
    listing_uri = created[1]

    slots = [{"slotId": "s1", "startHeight": now + 100, "endHeight": now + 200}]
    unit = one(call_uri(
        listing_uri, "ensureUnit",
        f'*dId, "race-unit:{now}", {{"name": "machine"}}, {json.dumps(slots)}, 10'))
    expect_ok("unit create", unit)
    unit_uri = unit[1]
    expect_ok("activate", one(call_uri(listing_uri, "activate", "*dId")))

    guest_book = resolve_component("component:guest-book")
    ledger = one(call_uri(guest_book, "getOrCreate", "*dId"))
    expect_ok("guest ledger", ledger)
    ledger_uri = ledger[1]
    funded = one(fund_account(keys["deployer"]["public"]))
    if not (isinstance(funded, list) and len(funded) == 2):
        raise AssertionError(f"fund: {funded!r}")

    terms_out = [
        book_term(ledger_uri, listing_uri, unit_uri, f"race-a:{now}"),
        book_term(ledger_uri, listing_uri, unit_uri, f"race-b:{now}"),
    ]
    ids = node.deploy_same_block(terms_out, keys["deployer"]["secret"])
    results = [unwrap(decode(node.deploy_result(deploy_id))) for deploy_id in ids]
    print(f"  results {results!r}")

    wins = [r for r in results if isinstance(r, tuple) and r and r[0] is True]
    losses = [r for r in results if isinstance(r, tuple) and r and r[0] is False]
    if len(wins) != 1 or len(losses) != 1:
        raise AssertionError(f"expected one win and one loss: {results!r}")
    reason = losses[0][1]
    if not (isinstance(reason, str) and ("slot" in reason or "held" in reason)):
        raise AssertionError(f"loser reason: {losses[0]!r}")
    print(f"  ok one booking won; loser refused {reason!r}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
