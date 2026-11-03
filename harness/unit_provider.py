#!/usr/bin/env python3
"""Provider suite: register, profile update, pause/resume and owner check."""

from __future__ import annotations

from assertions import expect, expect_ok, one, root_uri
from preflight import require_devnet
from unit_booking import call_uri

import deploy as node


def resolve(key: str) -> str:
    reply = one(call_uri(root_uri(), "resolve", f'"{key}", 1'))
    if not (isinstance(reply, tuple) and reply[0] is True):
        raise AssertionError(f"resolve {key}: {reply!r}")
    return reply[1]


def main() -> int:
    print("unit-provider")
    if not require_devnet("unit-provider"):
        return 2

    keys = node.load_keys()
    owner = keys["deployer"]["address"]
    book = resolve("component:provider-book")

    first = one(call_uri(book, "register", '*dId, {"name": "Provider One"}'))
    expect_ok("register", first)
    provider_uri, owner_id = first[1], first[2]
    expect("owner id", owner_id, owner)

    again = one(call_uri(book, "register", '*dId, {"name": "Provider One"}'))
    expect("register is stable", again, first)

    info = one(call_uri(provider_uri, "info"))
    expect_ok("info", info)
    expect("status", info[1].get("status"), "active")

    expect_ok("update profile", one(call_uri(
        provider_uri, "updateProfile",
        '*dId, {"name": "Provider Two"}, "profile:1"')))
    profile = one(call_uri(provider_uri, "profile"))
    expect("profile read", profile, (True, {"name": "Provider Two"}))

    expect_ok("pause", one(call_uri(provider_uri, "pause", "*dId")))
    expect("inactive", one(call_uri(provider_uri, "isActive")), (True, False))
    expect_ok("resume", one(call_uri(provider_uri, "resume", "*dId")))
    expect("active again", one(call_uri(provider_uri, "isActive")), (True, True))

    check = one(call_uri(provider_uri, "checkOwner", "*dId"))
    expect_ok("check owner", check)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
