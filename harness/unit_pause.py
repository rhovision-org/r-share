#!/usr/bin/env python3
"""Pause suite: global pause/resume, provider pause/resume and unit admin pause.

Admin actions are signed by admin1 and admin2 over the canonical
`rshare:admin:v1|<domain>|<nonce>|<action>|<args>` payload.
"""

from __future__ import annotations

import hashlib
import json

import actors
from assertions import expect, expect_ok, one, root_uri
from market_fixture import terms
from preflight import require_devnet

import deploy as node


def call_uri(uri: str, binding: str, args: str | None = None) -> str:
    head = f'"{binding}"' + (f", {args}" if args else "")
    return f"""
new rl(`rho:registry:lookup`), ch, ret,
    dId(`rho:rchain:deployerId`), deployId(`rho:rchain:deployId`) in {{
  rl!(`{uri}`, *ch) |
  for (C <- ch) {{
    C!({head}, *ret) |
    for (@r <- ret) {{ deployId!(r) }}
  }}
}}
"""


def block_height() -> int:
    term = """
new bd(`rho:block:data`), ch, deployId(`rho:rchain:deployId`) in {
  bd!(*ch) |
  for (@n, @sender, @timestamp <- ch) { deployId!(n) }
}
"""
    height = one(term)
    if not isinstance(height, int):
        raise AssertionError(f"block height: {height!r}")
    return height


def resolve_component(key: str) -> str:
    reply = one(call_uri(root_uri(), "resolve", f'"{key}", 1'))
    if not (isinstance(reply, tuple) and reply[0] is True):
        raise AssertionError(f"resolve {key}: {reply!r}")
    return reply[1]


def admin_sigs(keys: dict, domain: str, nonce: int, action: str,
               args_text: str = "") -> str:
    from coincurve import PrivateKey

    payload = f"rshare:admin:v1|{domain}|{nonce}|{action}|{args_text}"
    digest = hashlib.blake2b(payload.encode(), digest_size=32).digest()
    sigs = []
    for name in ("admin1", "admin2"):
        key = PrivateKey(bytes.fromhex(keys[name]["secret"]))
        sigs.append(f'"{key.sign(digest, hasher=None).hex()}".hexToBytes()')
    return "[" + ", ".join(sigs) + "]"


def admin_literal(keys: dict) -> str:
    return "[" + ", ".join(
        f'"{keys[name]["public"]}".hexToBytes()'
        for name in ("admin1", "admin2", "admin3")
    ) + "]"


def main() -> int:
    print("unit-pause")
    if not require_devnet("unit-pause"):
        return 2

    keys = node.load_keys()
    admin = resolve_component("component:admin")
    provider_book = resolve_component("component:provider-book")
    listing_factory = resolve_component("component:listing-factory")

    # The admin nonce is global and has no read arm; use a monotonic wall-clock value.
    pause_nonce = actors.admin_nonce_read() + 1
    paused = one(call_uri(
        admin, "admin",
        f'{pause_nonce}, "pauseAll", [], '
        f'{admin_sigs(keys, "admin", pause_nonce, "pauseAll")}'))
    expect_ok("global pause", paused)
    try:
        expect("isPaused after pause", one(call_uri(admin, "isPaused")), (True, True))
    finally:
        resume_nonce = actors.admin_nonce_read() + 1
        resumed = one(call_uri(
            admin, "admin",
            f'{resume_nonce}, "resumeAll", [], '
            f'{admin_sigs(keys, "admin", resume_nonce, "resumeAll")}'))
        expect_ok("global resume", resumed)
    expect("isPaused after resume", one(call_uri(admin, "isPaused")), (True, False))

    owner = keys["deployer"]["address"]
    registered = one(call_uri(
        provider_book, "register", '*dId, {"name": "Pause Test"}'))
    expect_ok("provider register", registered)
    again = one(call_uri(
        provider_book, "register", '*dId, {"name": "Pause Test"}'))
    if again != registered:
        raise AssertionError(f"provider register changed: {again!r}")
    provider_uri = registered[1]
    expect_ok("provider pause", one(call_uri(provider_uri, "pause", "*dId")))
    expect("provider inactive", one(call_uri(provider_uri, "isActive")), (True, False))
    expect_ok("provider resume", one(call_uri(provider_uri, "resume", "*dId")))
    expect("provider active", one(call_uri(provider_uri, "isActive")), (True, True))

    now = block_height()
    created_listing = one(call_uri(
        listing_factory, "create",
        f'*dId, "{provider_uri}", "listing:pause:{now}", "robot", '
        f'{json.dumps(terms(pricePerSlot=10))}'))
    expect_ok("listing create", created_listing)
    listing_uri = created_listing[1]
    slots = [{"slotId": "s1", "startHeight": now + 50, "endHeight": now + 150}]
    created = one(call_uri(
        listing_uri, "ensureUnit",
        f'*dId, "unit:pause", {{}}, {json.dumps(slots)}, 10'))
    expect_ok("unit create", created)
    unit_id = created[1]
    expect_ok("listing activate", one(call_uri(listing_uri, "activate", "*dId")))

    expect_ok("unit pause", one(call_uri(
        listing_uri, "pauseUnit", f'*dId, "{unit_id}"')))
    info = one(call_uri(listing_uri, "unit", f'"{unit_id}"'))
    expect_ok("unit info", info)
    expect("unit status", info[1].get("status"), "paused")
    expect_ok("unit resume", one(call_uri(
        listing_uri, "resumeUnit", f'*dId, "{unit_id}"')))
    info = one(call_uri(listing_uri, "unit", f'"{unit_id}"'))
    expect("unit resumed", info[1].get("status"), "active")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
