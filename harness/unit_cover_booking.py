#!/usr/bin/env python3
"""Atomic covered booking: reduced deposit, attach, claim token and settlement."""

from __future__ import annotations

import json
import time

import actors
import market_fixture
from assertions import expect, expect_ok, expect_refusal, one
from preflight import require_devnet
from unit_cover_policy import cover_offer, reserve_term, single


def authorize_attach_term(policy_uri: str, booking_uri: str, claim_id: str,
                          loss_id: str, amount: int, assessed: int,
                          non_cover: int) -> str:
    return f"""
new rl(`rho:registry:lookup`), pCh, aCh, bCh, ret,
    deployerId(`rho:rchain:deployerId`), deployId(`rho:rchain:deployId`) in {{
  rl!(`{policy_uri}`, *pCh) |
  for (P <- pCh) {{
    P!("authorize", *deployerId, "{claim_id}", "{loss_id}", {amount},
      {assessed}, {non_cover}, "evidence", *aCh) |
    for (@(true, Claim) <- aCh) {{
      rl!(`{booking_uri}`, *bCh) |
      for (B <- bCh) {{
        B!("attachClaim", *deployerId, "{claim_id}", Claim, *ret) |
        for (@r <- ret) {{ deployId!([r]) }}
      }}
    }}
  }}
}}
"""


def main() -> int:
    print("unit-cover-booking")
    if not require_devnet("unit-cover-booking"):
        return 2

    market = market_fixture.Market(
        "coverbooking", deposit_review_blocks=100, dispute_window_blocks=2).setup()
    provider = market.provider
    guest = market.guest
    provider_id = actors.address(provider)
    guest_id = actors.address(guest)
    actors.ensure_vault(provider)
    actors.ensure_account(provider)
    actors.fund(provider, 2000)
    actors.ensure_vault(guest)
    actors.ensure_account(guest)

    offer = cover_offer(provider_id)
    pool = actors.resolve("component:cover-pool")
    book = actors.resolve("component:cover-book")
    expect_ok("publish covered offer", actors.invoke(
        book, "publish", f'*dId, {json.dumps(offer)}', key=provider))
    expect_ok("reserve provider premium", one(
        reserve_term(pool, provider, provider_id, 1000), key=provider,
        phlo_limit=4_000_000))

    ledger = actors.resolve("component:guest-book")
    ledger_reply = actors.invoke(ledger, "getOrCreate", "*dId", key=guest)
    ledger_uri = ledger_reply[1]
    ledger_guest = ledger_reply[2]
    expect("ledger owner", ledger_guest, guest_id)

    pool_before = actors.invoke(pool, "info")[1]
    op_id = f"coverbook:{time.time_ns()}"
    policy_id = f"policy:{time.time_ns()}"
    booked = actors.invoke(
        ledger_uri, "bookWithCover",
        f'*dId, "{market.listing_uri}", {json.dumps(market.unit_slots())}, '
        f'"{op_id}", Nil, "{offer["offerId"]}", "{policy_id}"',
        key=guest, phlo_limit=7_500_000)
    expect_ok("atomic covered booking", booked)
    booking_uri = booked[2]
    booking = actors.invoke(booking_uri, "info")[1]
    expect("effective deposit", booking.get("deposit"), offer["deductible"])
    expect("cover attached", booking.get("cover").get("status"), "attached")
    expect("cover policy", booking.get("cover").get("policyId"), policy_id)
    bond = actors.bond_info(booking.get("bondUri"))
    expect("bond expected rent plus deductible", bond.get("expected"),
           booking.get("rent") + offer["deductible"])

    market.wait_in()
    in_op = market.op("cover-in")
    out_op = market.op("cover-out")
    claim_op = market.op("cover-claim")
    resolve_op = market.op("cover-resolve")
    market.check_in(booking_uri, in_op)
    market.check_out(booking_uri, out_op)
    expect_ok("provider claim", actors.invoke(
        booking_uri, "claimDeposit",
        f'*dId, "{claim_op}", 60, "damage"', key=provider))

    policy_uri = booking.get("cover").get("policyUri")
    claim_id = booking.get("bookingId")
    expect_ok("operator pays cover", actors.invoke(
        policy_uri, "pay",
        f'*dId, "{claim_id}", "{claim_id}", 50, 60, 10, "damage"'))

    actors.wait_height(actors.block_height() + 4)
    provider_before = actors.balance(provider)
    expect_ok("resolve deposit", actors.invoke(
        booking_uri, "resolveDeposit", f'"{resolve_op}"'))
    settle = actors.invoke(booking_uri, "info")[1]
    expect("booking settled", settle.get("status"), "settled")
    expect("cover closed", settle.get("cover").get("status"), "closed")
    expect("provider receives rent plus deposit", actors.balance(provider),
           provider_before + settle.get("rent") + offer["deductible"])
    expect("guest deposit consumed", actors.bond_balance(booking.get("bondUri")), 0)
    policy = actors.invoke(policy_uri, "info")[1]
    expect("policy closed", policy.get("status"), "settled")
    pool_info = actors.invoke(pool, "info")[1]
    expect("pool outstanding cleared", pool_info.get("outstanding"),
           pool_before.get("outstanding"))
    expect("pool paid cover", pool_info.get("claimsPaid"),
           pool_before.get("claimsPaid") + 50)

    # Unknown offer refuses before any Bond or slot hold.
    bad_op = f"coverbook:bad:{time.time_ns()}"
    expect_refusal("unknown covered offer", actors.invoke(
        ledger_uri, "bookWithCover",
        f'*dId, "{market.listing_uri}", {json.dumps(market.unit_slots())}, '
        f'"{bad_op}", Nil, "missing", "policy:missing"', key=guest),
        "unknown offer")

    print("unit-cover-booking: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
