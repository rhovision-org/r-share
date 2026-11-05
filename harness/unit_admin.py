#!/usr/bin/env python3
"""Admin suite: installed admin set, ShareAdmin refusals and booking `setAdmins`.

Payloads are produced by the on-chain `Codec("adminPayload", ...)` arm, never reconstructed
in Python, then signed off-chain over the blake2b-256 digest.
"""

from __future__ import annotations

import hashlib

import actors
from assertions import expect, expect_ok, expect_refusal, one
from market_fixture import started
from preflight import require_devnet
from unit_pause import block_height, call_uri, resolve_component

import deploy as node


def admin_literal(keys: dict, names: list[str]) -> str:
    return "[" + ", ".join(
        f'"{keys[name]["public"]}".hexToBytes()' for name in names) + "]"


def sign(keys: dict, names: list[str], payload: str) -> str:
    from coincurve import PrivateKey

    digest = hashlib.blake2b(payload.encode(), digest_size=32).digest()
    sigs = []
    for name in names:
        key = PrivateKey(bytes.fromhex(keys[name]["secret"]))
        sigs.append(f'"{key.sign(digest, hasher=None).hex()}".hexToBytes()')
    return "[" + ", ".join(sigs) + "]"


def payload(codec: str, domain: str, nonce: int, action: str,
            args_literal: str) -> str:
    reply = one(call_uri(
        codec, "adminPayload",
        f'"{domain}", {nonce}, "{action}", {args_literal}'))
    if not isinstance(reply, str):
        raise AssertionError(f"admin payload: {reply!r}")
    return reply


def main() -> int:
    print("unit-admin")
    if not require_devnet("unit-admin"):
        return 2

    keys = node.load_keys()
    admin = resolve_component("component:admin")
    codec = resolve_component("component:codec")

    keys_reply = one(call_uri(admin, "keys"))
    expect("admin keys", keys_reply[0], True)
    if len(keys_reply[1]) != 3 or keys_reply[2] != 2:
        raise AssertionError(f"admin set: {keys_reply!r}")

    nonce = actors.admin_nonce_read() + 1
    below = one(call_uri(
        admin, "admin", f'{nonce}, "pauseAll", [], '
        + sign(keys, ["admin1"],
               payload(codec, "admin", nonce, "pauseAll", "[]"))))
    expect_refusal("ShareAdmin below threshold", below, "below threshold")

    replay = one(call_uri(admin, "admin", '0, "pauseAll", [], []'))
    expect_refusal("ShareAdmin replay", replay, "replay")

    # ---- booking setAdmins -----------------------------------------------------------

    market = started("admin-payload", provider="recipient", instant_book=False)
    booked = market.book(market.op("book"))
    booking_uri = booked[2]
    state = market.booking_info(booking_uri)
    domain = "booking:" + state["bookingId"]
    print(f"  booking domain {domain}")

    current = ["admin2", "admin3", "key12"]
    current_lit = admin_literal(keys, current)
    nonce = actors.admin_nonce_read() + 1
    admins_text = ",".join(keys[name]["public"] for name in current)
    args = f'["{admins_text}", 2]'
    digest_payload = payload(codec, domain, nonce, "setAdmins", args)

    rotated = one(call_uri(
        booking_uri, "setAdmins",
        f'{nonce}, {current_lit}, 2, '
        + sign(keys, ["admin1", "admin2"], digest_payload)))
    expect_ok("rotation", rotated)
    state = market.booking_info(booking_uri)
    expect("rotation admins", state["admins"],
           [bytes.fromhex(keys[name]["public"]) for name in current])
    expect("rotation threshold", state["threshold"], 2)
    expect("rotation nonce", state["nonce"], nonce)

    replay = one(call_uri(
        booking_uri, "setAdmins",
        f'{nonce}, {current_lit}, 2, '
        + sign(keys, ["admin1", "admin2"], digest_payload)))
    expect_refusal("booking replay", replay, "replay")

    next_nonce = nonce + 1
    one_sig = sign(keys, ["admin2"],
                   payload(codec, domain, next_nonce, "setAdmins", args))
    expect_refusal("booking below threshold",
                   one(call_uri(booking_uri, "setAdmins",
                                f'{next_nonce}, {current_lit}, 2, {one_sig}')),
                   "below threshold")

    bad_args = f'["{admins_text}", 4]'
    bad_payload = payload(codec, domain, next_nonce, "setAdmins", bad_args)
    expect_refusal("invalid threshold",
                   one(call_uri(booking_uri, "setAdmins",
                                f'{next_nonce}, {current_lit}, 4, '
                                + sign(keys, ["admin2", "admin3"], bad_payload))),
                   "threshold")

    wrong_payload = payload(codec, domain, next_nonce, "pauseAll", "[]")
    expect_refusal("wrong payload",
                   one(call_uri(booking_uri, "setAdmins",
                                f'{next_nonce}, {current_lit}, 2, '
                                + sign(keys, ["admin2", "admin3"], wrong_payload))),
                   "below threshold")

    stale_payload = payload(codec, domain, next_nonce, "setAdmins", args)
    expect_refusal("old-key signatures",
                   one(call_uri(booking_uri, "setAdmins",
                                f'{next_nonce}, {current_lit}, 2, '
                                + sign(keys, ["admin1", "admin2"], stale_payload))),
                   "below threshold")

    restore = ["admin1", "admin2", "admin3"]
    restore_lit = admin_literal(keys, restore)
    restore_text = ",".join(keys[name]["public"] for name in restore)
    restore_args = f'["{restore_text}", 2]'
    restore_payload = payload(codec, domain, next_nonce, "setAdmins", restore_args)
    restored = one(call_uri(
        booking_uri, "setAdmins",
        f'{next_nonce}, {restore_lit}, 2, '
        + sign(keys, ["admin2", "admin3"], restore_payload)))
    expect_ok("new-key rotation", restored)
    state = market.booking_info(booking_uri)
    expect("restored admins", state["admins"],
           [bytes.fromhex(keys[name]["public"]) for name in restore])

    # Listing and provider setAdmins use the same payload encoder; exercise both so the
    # shared AdminText fix cannot regress behind the booking test. Rotation is idempotent:
    # read the current set, rotate to the other set, then restore the original.
    listing_id = state["listingId"]
    provider_owner = keys[market.provider]["address"]
    by_public = {keys[name]["public"]: name for name in keys}
    sets = [["admin1", "admin2", "admin3"], ["admin2", "admin3", "key12"]]
    for label, uri, surface_domain, surface_nonce in (
        ("listing", market.listing_uri, "listing:" + listing_id, next_nonce + 10),
        ("provider", market.provider_uri, "provider:" + provider_owner,
         next_nonce + 20),
    ):
        info = one(call_uri(uri, "info"))
        current_names = [by_public[admin.hex()] for admin in info[1]["admins"]]
        new_names = sets[0] if current_names != sets[0] else sets[1]
        new_lit = admin_literal(keys, new_names)
        new_text = ",".join(keys[name]["public"] for name in new_names)
        surface_args = f'["{new_text}", 2]'
        surface_payload = payload(codec, surface_domain, surface_nonce,
                                  "setAdmins", surface_args)
        reply = one(call_uri(
            uri, "setAdmins",
            f'{surface_nonce}, {new_lit}, 2, '
            + sign(keys, current_names[:2], surface_payload)))
        expect_ok(label + " rotation", reply)
        info = one(call_uri(uri, "info"))
        expect(label + " admins", info[1]["admins"],
               [bytes.fromhex(keys[name]["public"]) for name in new_names])
        expect(label + " threshold", info[1]["threshold"], 2)

        restore_lit = admin_literal(keys, current_names)
        restore_text = ",".join(keys[name]["public"] for name in current_names)
        restore_args = f'["{restore_text}", 2]'
        restore_nonce = surface_nonce + 1
        restore_payload = payload(codec, surface_domain, restore_nonce,
                                  "setAdmins", restore_args)
        restored_surface = one(call_uri(
            uri, "setAdmins",
            f'{restore_nonce}, {restore_lit}, 2, '
            + sign(keys, new_names[:2], restore_payload)))
        expect_ok(label + " restore", restored_surface)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
