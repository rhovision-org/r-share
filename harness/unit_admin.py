#!/usr/bin/env python3
"""Admin suite: threshold and replay refusals, and the installed admin set."""

from __future__ import annotations

import hashlib

from assertions import expect, expect_refusal, one
from preflight import require_devnet
from unit_pause import block_height, call_uri, resolve_component

import deploy as node


def one_sig(keys: dict, domain: str, nonce: int, action: str) -> str:
    from coincurve import PrivateKey

    payload = f"rshare:admin:v1|{domain}|{nonce}|{action}|"
    digest = hashlib.blake2b(payload.encode(), digest_size=32).digest()
    key = PrivateKey(bytes.fromhex(keys["admin1"]["secret"]))
    return f'["{key.sign(digest, hasher=None).hex()}".hexToBytes()]'


def main() -> int:
    print("unit-admin")
    if not require_devnet("unit-admin"):
        return 2

    keys = node.load_keys()
    admin = resolve_component("component:admin")

    keys_reply = one(call_uri(admin, "keys"))
    expect("admin keys", keys_reply[0], True)
    if len(keys_reply[1]) != 3 or keys_reply[2] != 2:
        raise AssertionError(f"admin set: {keys_reply!r}")

    nonce = block_height() + 1000
    below = one(call_uri(
        admin, "admin",
        f'{nonce}, "pauseAll", [], {one_sig(keys, "admin", nonce, "pauseAll")}'))
    expect_refusal("below threshold", below, "below threshold")

    replay = one(call_uri(admin, "admin", '0, "pauseAll", [], []'))
    expect_refusal("replay", replay, "replay")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
