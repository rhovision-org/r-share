#!/usr/bin/env python3
"""Refund/transfer cutoff validation (F-013)."""

from __future__ import annotations

import json

import actors
from assertions import expect_ok, expect_refusal
from market_fixture import terms
from preflight import require_devnet


def raw_listing(kind: str, policy: dict):
    actors.ensure_vault("business")
    provider_book = actors.resolve("component:provider-book")
    registered = actors.invoke(
        provider_book, "register", '*dId, {"name": "cutoff"}', key="business")
    expect_ok("provider register", registered)
    factory = actors.resolve("component:listing-factory")
    created = actors.invoke(
        factory, "create",
        f'*dId, "{registered[1]}", "listing:cutoff:{kind}:{id(policy)}", '
        f'"{kind}", {json.dumps(policy)}', key="business")
    expect_ok("listing create", created)
    return created[1]


def main() -> int:
    print("unit-cutoffs")
    if not require_devnet("unit-cutoffs"):
        return 2

    now = actors.block_height()
    slots = [{"slotId": "s1", "startHeight": now + 300,
              "endHeight": now + 400}]

    listing = raw_listing("lodging", terms(fullRefundBeforeBlocks=100000, partialRefundBeforeBlocks=100000))
    refused = actors.invoke(
        listing, "ensureUnit",
        f'*dId, "u1", {{}}, {json.dumps(slots)}, 10', key="business")
    expect_refusal("full-refund cutoff beyond start", refused, "refund window")

    listing = raw_listing("event", terms(eventTransferCutoffBlocks=200))
    refused = actors.invoke(
        listing, "ensureUnit",
        f'*dId, "u2", {{}}, {json.dumps(slots)}, 10', key="business")
    expect_refusal("event cutoff beyond duration", refused, "refund window")

    listing = raw_listing("lodging", terms(fullRefundBeforeBlocks=0,
                                           partialRefundBeforeBlocks=0))
    accepted = actors.invoke(
        listing, "ensureUnit",
        f'*dId, "u3", {{}}, {json.dumps(slots)}, 10', key="business")
    expect_ok("valid cutoff accepted", accepted)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
