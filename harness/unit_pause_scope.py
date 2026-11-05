#!/usr/bin/env python3
"""Pause scope regression (F-007): direct acquire and provider creation are gated."""

from __future__ import annotations

import json

import actors
from assertions import expect_ok, expect_refusal, one
from market_fixture import started
from preflight import require_devnet
from unit_allocator import raw_book
from unit_pause import admin_sigs


def direct_acquire(market, key: str, until: int):
    term = f"""
new rl(`rho:registry:lookup`), lCh, ret, releaseToken,
    deployId(`rho:rchain:deployId`) in {{
  rl!(`{market.listing_uri}`, *lCh) |
  for (L <- lCh) {{
    L!("acquire", "audit-pause", {json.dumps(market.unit_slots())}, {until},
       *releaseToken, *ret) |
    for (@r <- ret) {{ deployId!(r) }}
  }}
}}
"""
    return one(term, key=key)


def main() -> int:
    print("unit-pause-scope")
    if not require_devnet("unit-pause-scope"):
        return 2

    keys = actors.keys()
    admin = actors.resolve("component:admin")
    market = started("pause-scope", instant_book=True)
    pause_nonce = actors.admin_nonce_read() + 1
    expect_ok("global pause", actors.invoke(
        admin, "admin",
        f'{pause_nonce}, "pauseAll", [], '
        f'{admin_sigs(keys, "admin", pause_nonce, "pauseAll")}'))
    try:
        expect_refusal("GuestLedger book while paused",
                       raw_book(market, market.op("guest"), market.slot_ids),
                       "paused")
        expect_refusal("direct acquire while paused",
                       direct_acquire(market, market.provider,
                                      actors.block_height() + 50),
                       "paused")
    finally:
        resume_nonce = actors.admin_nonce_read() + 1
        expect_ok("global resume", actors.invoke(
            admin, "admin",
            f'{resume_nonce}, "resumeAll", [], '
            f'{admin_sigs(keys, "admin", resume_nonce, "resumeAll")}'))

    expect_ok("provider pause", actors.invoke(
        market.provider_uri, "pause", "*dId", key=market.provider))
    try:
        expect_refusal("direct acquire while provider paused",
                       direct_acquire(market, market.provider,
                                      actors.block_height() + 50),
                       "provider paused")
    finally:
        expect_ok("provider resume", actors.invoke(
            market.provider_uri, "resume", "*dId", key=market.provider))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
