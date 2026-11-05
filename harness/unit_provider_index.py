#!/usr/bin/env python3
"""Provider removeListing must delist the authoritative listing (F-012/F-014)."""

from __future__ import annotations

import actors
from assertions import expect, expect_ok, expect_refusal
from market_fixture import started
from preflight import require_devnet


def main() -> int:
    print("unit-provider-index")
    if not require_devnet("unit-provider-index"):
        return 2

    market = started("provider-index", instant_book=True)
    listing = actors.invoke(market.listing_uri, "info")[1]
    listing_id = listing["listingId"]

    before = actors.invoke(market.provider_uri, "listing", f'"{listing_id}"',
                           key=market.provider)
    expect_ok("provider index before remove", before)
    expect("listing active before remove",
           actors.invoke(market.listing_uri, "isActive")[1], True)

    removed = actors.invoke(
        market.provider_uri, "removeListing",
        f'*dId, "{listing_id}"', key=market.provider)
    expect_ok("removeListing", removed)
    after = actors.invoke(market.provider_uri, "listing", f'"{listing_id}"',
                          key=market.provider)
    expect_refusal("provider index after remove", after, "removed")
    expect("listing delisted", actors.invoke(
        market.listing_uri, "isActive")[1], False)
    expect_refusal("resume delisted", actors.invoke(
        market.listing_uri, "resume", "*dId", key=market.provider), "delisted")
    expect_refusal("pause delisted", actors.invoke(
        market.listing_uri, "pause", "*dId", key=market.provider), "delisted")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
