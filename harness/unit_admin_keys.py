#!/usr/bin/env python3
"""Admin-set soundness regression (F-006): distinct keys, threshold >= 2."""

from __future__ import annotations

import hashlib

import actors
from assertions import expect_ok, expect_refusal
from preflight import require_devnet
from unit_pause import admin_sigs

import deploy as node


def signature(keys: dict, name: str, payload: str) -> str:
    from coincurve import PrivateKey

    digest = hashlib.blake2b(payload.encode(), digest_size=32).digest()
    key = PrivateKey(bytes.fromhex(keys[name]["secret"]))
    return f'"{key.sign(digest, hasher=None).hex()}".hexToBytes()'


def key_list(keys: dict, names: list[str]) -> str:
    return "[" + ", ".join(
        f'"{keys[name]["public"]}".hexToBytes()' for name in names) + "]"


def main() -> int:
    print("unit-admin-keys")
    if not require_devnet("unit-admin-keys"):
        return 2

    keys = node.load_keys()
    guard = actors.resolve("component:admin-guard")
    payload = "rshare:admin:v1|audit|1|test|arg"

    duplicate = actors.invoke(
        guard, "verify",
        f'"audit", 1, "test", ["arg"], [{signature(keys, "admin1", payload)}], '
        f'{key_list(keys, ["admin1", "admin1", "admin2"])}, 2')
    expect_refusal("duplicate keys cannot satisfy threshold", duplicate,
                   "below threshold")

    distinct = actors.invoke(
        guard, "verify",
        f'"audit", 1, "test", ["arg"], '
        f'[{signature(keys, "admin1", payload)}, '
        f'{signature(keys, "admin2", payload)}], '
        f'{key_list(keys, ["admin1", "admin2", "admin3"])}, 2')
    expect_ok("distinct 2-of-3", distinct)

    low = actors.invoke(
        guard, "verify",
        f'"audit", 1, "test", ["arg"], [{signature(keys, "admin1", payload)}], '
        f'{key_list(keys, ["admin1", "admin2", "admin3"])}, 1')
    expect_refusal("threshold 1 refused", low, "below threshold")

    admin = actors.resolve("component:admin")
    nonce = actors.admin_nonce_read() + 1
    sigs = admin_sigs(keys, "admin", nonce, "setThreshold", "1")
    refused = actors.invoke(
        admin, "admin", f'{nonce}, "setThreshold", [1], {sigs}')
    expect_refusal("ShareAdmin setThreshold 1", refused, "threshold")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
