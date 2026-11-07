#!/usr/bin/env python3
"""Allocator conformance: atomic all-or-none acquisition, token gates, pause and privacy.

The interleaving properties are proved in `spec/lean/RShare/Model/Allocator.lean`;
this suite checks that the deployed listing arm implements the same transition table.
"""

from __future__ import annotations

import json

import actors
from assertions import expect, expect_ok, expect_refusal, one
from market_fixture import Market, started
from preflight import require_devnet


def raw_book(market: Market, op_id: str, slot_ids: list[str]):
    return actors.invoke(
        market.ledger_uri, "book",
        f'*dId, "{market.listing_uri}", '
        f"{json.dumps(market.unit_slots(slot_ids))}, \"{op_id}\", Nil",
        key=market.guest)


def fresh_token_call(listing_uri: str, arm: str, args: str):
    term = f"""
new rl(`rho:registry:lookup`), lCh, wrong, ret,
    deployId(`rho:rchain:deployId`) in {{
  rl!(`{listing_uri}`, *lCh) |
  for (L <- lCh) {{
    L!("{arm}", {args}, *wrong, *ret) |
    for (@r <- ret) {{ deployId!(r) }}
  }}
}}
"""
    return one(term)


def multi_slot() -> None:
    print("  atomic multi-slot acquire")
    market = started("alloc-multi", instant_book=False, slot_ids=["s1", "s2"])
    booked = market.book(market.op("book"))
    expect_ok("multi book", booked)
    states = market.slot_states(["s1", "s2"])
    for slot in ("s1", "s2"):
        if states[slot][0] != "held":
            raise AssertionError(f"multi partial hold: {states!r}")
    market.accept(booked[2], market.op("accept"))
    states = market.slot_states(["s1", "s2"])
    for slot in ("s1", "s2"):
        if states[slot][0] != "booked":
            raise AssertionError(f"multi confirm: {states!r}")
    market.cancel(booked[2], market.op("cancel"))
    states = market.slot_states(["s1", "s2"])
    for slot in ("s1", "s2"):
        if states[slot][0] != "free":
            raise AssertionError(f"multi release: {states!r}")
    print("  ok all-or-none hold, confirm and release")


def held_refuses_all() -> None:
    print("  one held slot refuses the whole set")
    market = started("alloc-partial", instant_book=False, slot_ids=["s1", "s2"])
    expect_ok("first hold", market.book(market.op("first"), slot_ids=["s1"]))
    refused = raw_book(market, market.op("second"), ["s1", "s2"])
    expect_refusal("partial request", refused, "slot held")
    states = market.slot_states(["s1", "s2"])
    if states["s2"][0] != "free":
        raise AssertionError(f"partial acquisition left s2 held: {states!r}")
    print("  ok refused request left the free slot untouched")


def token_gates() -> None:
    print("  token-only confirm and release")
    market = started("alloc-token", instant_book=False)
    booked = market.book(market.op("book"))
    uri = booked[2]
    unit_slots = market.unit_slots()
    confirm = fresh_token_call(
        market.listing_uri, "confirm",
        f'{json.dumps(booked[1])}, {json.dumps(unit_slots)}')
    expect_refusal("wrong token confirm", confirm, "not holder")
    release = fresh_token_call(
        market.listing_uri, "release", json.dumps(unit_slots))
    expect_refusal("wrong token release", release, "not holder")
    expect_ok("holder confirm", market.accept(uri, market.op("accept")))
    expect_ok("holder release", market.cancel(uri, market.op("cancel")))
    print("  ok wrong token moved nothing")


def pause_gates() -> None:
    print("  pause gates")
    market = started("alloc-pause", instant_book=True)
    expect_ok("listing pause", actors.invoke(
        market.listing_uri, "pause", "*dId", key=market.provider))
    refused = raw_book(market, market.op("paused"), market.slot_ids)
    expect_refusal("book paused listing", refused, "listing inactive")
    expect_ok("listing resume", actors.invoke(
        market.listing_uri, "resume", "*dId", key=market.provider))
    expect_ok("unit pause", actors.invoke(
        market.listing_uri, "pauseUnit",
        f'*dId, "{market.unit_id}"', key=market.provider))
    refused = raw_book(market, market.op("unit-paused"), market.slot_ids)
    expect_refusal("book paused unit", refused, "unit paused")
    expect_ok("unit resume", actors.invoke(
        market.listing_uri, "resumeUnit",
        f'*dId, "{market.unit_id}"', key=market.provider))
    print("  ok listing and unit pause refuse acquisition")


def expiry_reclaim() -> None:
    print("  expiry takeover")
    market = started("alloc-expiry", instant_book=False, holdBlocks=1)
    old = market.book(market.op("old"))
    actors.wait_height(market.booking_info(old[2])["holdUntilHeight"] + 1)
    new = market.book(market.op("new"))
    expect_ok("takeover book", new)
    expect("old expired", market.expire(old[2], market.op("expire"))[0], True)
    expect("old terminal", market.booking_info(old[2])["status"], "expired")
    state = market.slot_state()
    if state[0] != "held":
        raise AssertionError(f"takeover slot: {state!r}")
    print("  ok expired hold reclaimed without a double booking")


def privacy() -> None:
    print("  no slot capability leak")
    market = started("alloc-privacy", instant_book=False)
    info = actors.invoke(market.listing_uri, "info")[1]
    expect("info has no slots", "slots" in info, False)
    unit = actors.invoke(market.listing_uri, "unit", f'"{market.unit_id}"')[1]
    expect("unit has no slots", "slots" in unit, False)
    held = market.book(market.op("book"))
    state = market.slot_state()
    if len(state) != 3 or state[0] != "held":
        raise AssertionError(f"probe leaked a token: {state!r}")
    market.cancel(held[2], market.op("cancel"))
    state = market.slot_state()
    if len(state) != 2 or state[0] != "free":
        raise AssertionError(f"free probe shape: {state!r}")
    print("  ok projections and probe redact the token")


def contention() -> None:
    print("  same-slot contention")
    market = started("alloc-contention", instant_book=True)
    first = market.book(market.op("first"))
    expect_ok("first booking", first)
    second = raw_book(market, market.op("second"), market.slot_ids)
    expect_refusal("second booking", second, "slot held")
    state = market.slot_state()
    if state[0] != "booked":
        raise AssertionError(f"contention slot: {state!r}")
    print("  ok one owner and one refusal")


def main() -> int:
    print("unit-allocator")
    if not require_devnet("unit-allocator"):
        return 2
    multi_slot()
    held_refuses_all()
    token_gates()
    pause_gates()
    expiry_reclaim()
    privacy()
    contention()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
