#!/usr/bin/env python3
"""CoverTerms and CoverBook: payer rule, pricing floor, offer lifecycle."""

from __future__ import annotations

import json

import actors
from assertions import expect, expect_ok, expect_refusal
from preflight import require_devnet


def offer(**overrides) -> dict:
    base = {
        "offerId": "offer:cover:1",
        "subjectKind": "booking",
        "useCase": "lodging",
        "product": "asset_protection",
        "trigger": "damage",
        "insuredRole": "provider",
        "beneficiaryRole": "provider",
        "payerRole": "provider",
        "payer": "insured",
        "assetClass": "property",
        "assetId": "REV",
        "coverageCap": 200,
        "deductible": 10,
        "claimWindowBlocks": 100,
        "evidenceSchema": "inspection/v1",
        "depositEffect": "reduce_to_deductible",
        "kassetUri": "",
        "engineeringBps": 4000,
        "insurerVarianceBps": 5000,
        "loadBps": 2000,
        "expectedLoss": 6,
        "premiumFloor": 6,
        "premiumTotal": 8,
        "gapBps": 1000,
        "pricingMode": "insurer-only",
        "publisherId": actors.address("deployer"),
    }
    base.update(overrides)
    return base


def main() -> int:
    print("unit-cover")
    if not require_devnet("unit-cover"):
        return 2

    terms = actors.resolve("component:cover-terms")
    book = actors.resolve("component:cover-book")

    valid = offer()
    expect_ok("valid provider offer", actors.invoke(
        terms, "validate", json.dumps(valid)))
    expect_ok("valid platform offer", actors.invoke(
        terms, "validate", json.dumps(offer(
            payer="platform", payerRole="platform"))))

    expect_refusal("payer cannot be the counterparty", actors.invoke(
        terms, "validate", json.dumps(offer(
            payer="guest", payerRole="guest"))), "payer")
    expect_refusal("insured payer must be the insured role", actors.invoke(
        terms, "validate", json.dumps(offer(
            payer="insured", payerRole="guest"))), "payer")
    expect_refusal("premium below floor", actors.invoke(
        terms, "validate", json.dumps(offer(premiumTotal=5))), "premium")
    expect_refusal("deductible at the cap", actors.invoke(
        terms, "validate", json.dumps(offer(deductible=200))), "deductible")
    expect_refusal("negative premium", actors.invoke(
        terms, "validate", json.dumps(offer(premiumFloor=-1))), "premium")

    published = actors.invoke(book, "publish", f'*dId, {json.dumps(valid)}')
    expect_ok("publish provider offer", published)
    offer_id, digest = published[1], published[2]
    again = actors.invoke(book, "publish", f'*dId, {json.dumps(valid)}')
    expect("publish is idempotent for the same hash", again[1:], published[1:])
    different = actors.invoke(book, "publish", f'*dId, {json.dumps(offer(
        coverageCap=300))}')
    expect_refusal("same offer id with a new hash", different, "offer exists")

    stored = actors.invoke(book, "offer", f'"{offer_id}"')
    expect_ok("read offer", stored)
    expect("offer status", stored[1].get("status"), "active")
    expect("offer hash", stored[1].get("offerHash"), digest)
    expect_ok("pause offer", actors.invoke(
        book, "pause", f'*dId, "{offer_id}"'))
    expect("paused status", actors.invoke(
        book, "offer", f'"{offer_id}"')[1].get("status"), "paused")
    expect_ok("retire offer", actors.invoke(
        book, "retire", f'*dId, "{offer_id}"'))
    expect_refusal("publish a retired offer", actors.invoke(
        book, "publish", f'*dId, {json.dumps(valid)}'), "retired")
    expect_refusal("pause a retired offer", actors.invoke(
        book, "pause", f'*dId, "{offer_id}"'), "retired")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
