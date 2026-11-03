#!/usr/bin/env python3
"""Root directory suite: resolve, keys, unknown key."""

from __future__ import annotations

import sys

from assertions import expect, expect_ok, expect_refusal, one, root_call
from preflight import require_devnet


def main() -> int:
    print("unit-root")
    if not require_devnet("unit-root"):
        return 2
    keys = one(root_call("keys"))
    expect_ok("root keys", keys)
    if not isinstance(keys[1], list) or "component:root" not in keys[1]:
        raise AssertionError(f"root keys missing component:root: {keys!r}")

    clock = one(root_call("resolve", '"component:clock", 1'))
    expect_ok("resolve clock", clock)
    if clock[1].startswith("rho:id:") is False or clock[2] != 1:
        raise AssertionError(f"bad clock entry: {clock!r}")

    missing = one(root_call("resolve", '"component:nope", 1'))
    expect_refusal("unknown component", missing, "unknown component")

    stale = one(root_call("resolve", '"component:clock", 99'))
    expect_refusal("stale version", stale, "stale")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
