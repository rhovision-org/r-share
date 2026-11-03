#!/usr/bin/env python3
"""Payments adapter suite: r-cash health, platform owner, REV issuer."""

from __future__ import annotations

from assertions import (call, component_uri, expect, expect_ok,
                        expect_refusal, one)
from preflight import require_devnet


def main() -> int:
    print("unit-payments")
    if not require_devnet("unit-payments"):
        return 2
    payments = component_uri("40_payments.rho")

    health = one(call(payments, "health"))
    expect_ok("payments health", health)
    versions = health[1]
    for key in ("accountFactory", "assetResolver", "clock"):
        if versions.get(key) != 1:
            raise AssertionError(f"{key} version: {versions!r}")

    owner = one(call(payments, "platformOwner"))
    expect_ok("platform owner", owner)
    if not isinstance(owner[1], str) or not owner[1].startswith("1111"):
        raise AssertionError(f"platform owner is not a REV address: {owner!r}")

    issuer = one(call(payments, "issuer", '"REV"'))
    expect_ok("REV issuer", issuer)
    if len(issuer) < 3:
        raise AssertionError(f"issuer reply shape: {issuer!r}")

    unknown = one(call(payments, "issuer", '"NOPE"'))
    expect_refusal("unknown asset", unknown, "unknown asset")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
