#!/usr/bin/env python3
"""HoldDeposit: refundable per-acquire deposit and the confirm gate."""

from __future__ import annotations

import json
import time

import actors
import market_fixture
from assertions import expect, expect_ok, expect_refusal, one
from preflight import require_devnet


def _components() -> tuple[str, str]:
    return actors.resolve("component:payments"), actors.resolve("component:hold-deposit")


def acquire_term(market: market_fixture.Market, booking_id: str, amount: int,
                 action: str = "refund") -> str:
    payments, hold = _components()
    payer = actors.address(market.guest)
    until = actors.block_height() + 40
    slots = json.dumps(market.unit_slots())
    after = {
        "refund": f"""
                  rl!(`{hold}`, *hdCh) |
                  for (H <- hdCh) {{
                    H!("settle", "{booking_id}", "release", *releaseToken, *s1) |
                    for (s1v <- s1) {{
                      rl!(`{market.listing_uri}`, *l2) |
                      for (L2 <- l2) {{
                        L2!("release", {slots}, *releaseToken, *relCh) |
                        for (relv <- relCh) {{
                          H!("settle", "{booking_id}", "release",
                            *releaseToken, *s2) |
                          for (s2v <- s2) {{
                            deployId!([r, s1v, relv, s2v])
                          }}
                        }}
                      }}
                    }}
                  }}
""",
        "confirm": f"""
                  rl!(`{market.listing_uri}`, *l2) |
                  for (L2 <- l2) {{
                    L2!("confirm", "{booking_id}", "{market.listing_uri}",
                       {slots}, *releaseToken, *cfCh) |
                    for (cfv <- cfCh) {{ deployId!([r, cfv]) }}
                  }}
""",
        "none": "deployId!([r])",
    }[action]
    return f"""
new rl(`rho:registry:lookup`), payCh, wCh, lCh, l2, ret, cfCh, relCh,
    s1, s2, hdCh, releaseToken, dId(`rho:rchain:deployerId`),
    deployId(`rho:rchain:deployId`) in {{
  rl!(`{payments}`, *payCh) |
  for (P <- payCh) {{
    P!("withdrawPayment", *dId, "REV", {amount}, *wCh) |
    for (w <- wCh) {{
      match w {{
        (true, payment) => {{
          rl!(`{market.listing_uri}`, *lCh) |
          for (L <- lCh) {{
            L!("acquire", "{booking_id}", "{payer}",
               {slots}, {until}, *releaseToken, *payment, *ret) |
            for (r <- ret) {{
              match r {{
                (true, _) => {{{after}}}
                _ => {{ deployId!([r]) }}
              }}
            }}
          }}
        }}
        _ => {{ deployId!([w]) }}
      }}
    }}
  }}
}}
"""


def refund_flow() -> None:
    print("  deposit is held, refused while held, refunded after release")
    market = market_fixture.Market("hold-refund", instant_book=False,
                                   slot_lead=0).setup()
    booking_id = f"{actors.address(market.guest)}:hold:{time.time_ns()}"
    result = one(acquire_term(market, booking_id, market.price), key=market.guest)
    expect("acquire", result[0][0], True)
    expect_refusal("settle while held", result[1], "held")
    expect_ok("release", result[2])
    expect_ok("settle after release", result[3])
    print("  ok refunded exactly once")


def confirm_gate() -> None:
    print("  confirm without a genuine booking is refused")
    market = market_fixture.Market("hold-confirm", instant_book=True,
                                   slot_lead=0).setup()
    booking_id = f"{actors.address(market.guest)}:hold:{time.time_ns()}"
    result = one(acquire_term(market, booking_id, market.price, "confirm"),
                 key=market.guest)
    expect("acquire", result[0][0], True)
    expect_refusal("confirm without booking", result[1], "booking")
    states = market.slot_states()
    for slot in states.values():
        if slot[0] != "held":
            raise AssertionError(f"confirm gate changed the slot: {states!r}")
    print("  ok confirm gate left the hold in place")


def wrong_amount() -> None:
    print("  a short payment is refused and moves no slot")
    market = market_fixture.Market("hold-amount", instant_book=False,
                                   slot_lead=0).setup()
    booking_id = f"{actors.address(market.guest)}:hold:{time.time_ns()}"
    result = one(acquire_term(market, booking_id, market.price - 1, "none"),
                 key=market.guest)
    expect_refusal("short deposit", result, "amount")
    states = market.slot_states()
    for slot in states.values():
        if slot[0] != "free":
            raise AssertionError(f"short deposit changed the slot: {states!r}")
    print("  ok short deposit refused, slot free")


def plain_book_refund() -> None:
    print("  a normal instant book still confirms")
    market = market_fixture.Market("hold-book", instant_book=True,
                                   slot_lead=0).setup()
    booked = market.book(market.op("book"))
    expect_ok("book", booked)
    info = market.booking_info(booked[2])
    expect("booking status", info.get("status"), "confirmed")
    print("  ok plain book confirmed with the deposit path")


def main() -> int:
    print("unit-hold-deposit")
    if not require_devnet("unit-hold-deposit"):
        return 2
    refund_flow()
    confirm_gate()
    wrong_amount()
    plain_book_refund()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
