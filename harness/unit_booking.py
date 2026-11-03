#!/usr/bin/env python3
"""End-to-end booking: two units at different prices and windows in one booking.

Uses the real provider/listing/unit/guest-ledger/Bond path. It needs a funded r-cash
account for the deployer; if the account cannot fund the escrow, the book refuses and the
suite fails rather than claiming a pass.
"""

from __future__ import annotations

import json
from pathlib import Path

from assertions import expect_ok, one, root_uri
from preflight import require_devnet
from rcash_paths import configured_manifest

import deploy as node

ROOT = Path(__file__).resolve().parent.parent


def rcash_root_uri() -> str:
    cfg = json.loads((ROOT / "configs" / "devnet.json").read_text())
    manifest = configured_manifest(cfg["rcash_manifest"])
    return json.loads(manifest.read_text())["root_uri"]


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


def fund_account(public_hex: str) -> str:
    """Deposit REV into the deployer's r-cash account through the deposit facet."""
    return f"""
new rl(`rho:registry:lookup`), rootCh, afCh, ac, oCh, fCh, bCh,
    ertpCh, revIssuerCh, purseCh, fundCh, wdCh, depCh,
    deployId(`rho:rchain:deployId`), deployerId(`rho:rchain:deployerId`) in {{
  rl!(`{rcash_root_uri()}`, *rootCh) |
  rl!(`rho:rchain:ertp`, *ertpCh) |
  for (Root <- rootCh; @(_, ERTP) <- ertpCh) {{
    Root!("resolve", "component:account-factory", 1, *afCh) |
    for (@(true, afUri, _) <- afCh) {{
      rl!(afUri, *ac) |
      for (AccountFactory <- ac) {{
        AccountFactory!("getOrCreate", "{public_hex}".hexToBytes(), *oCh) |
        for (@(true, account, ownerId) <- oCh) {{
          @account!("depositFacet", "REV", *fCh) |
          for (@fr <- fCh) {{
            match fr {{
              (true, facet) => {{
                @ERTP!("getRevIssuer", *revIssuerCh) |
                for (@(revBrand, revIssuer) <- revIssuerCh) {{
                  @revIssuer!("makeEmptyPurse", *purseCh) |
                  for (@(true, purse) <- purseCh) {{
                    @purse!("revFund", *deployerId, 100000, *fundCh) |
                    for (@_ <- fundCh) {{
                      @purse!("withdraw", (revBrand, 1000), *wdCh) |
                      for (@(true, payment) <- wdCh) {{
                        @facet!("deposit", payment, *depCh) |
                        for (@d <- depCh) {{
                          match d {{
                            (true, balance) => {{ deployId!([ownerId, balance]) }}
                            _ => {{ deployId!(d) }}
                          }}
                        }}
                      }}
                    }}
                  }}
                }}
              }}
              _ => {{ deployId!(fr) }}
            }}
          }}
        }}
      }}
    }}
  }}
}}
"""


def account_balance(public_hex: str) -> tuple:
    term = f"""
new rl(`rho:registry:lookup`), rootCh, afCh, ac, oCh, bCh,
    deployId(`rho:rchain:deployId`), deployerId(`rho:rchain:deployerId`) in {{
  rl!(`{rcash_root_uri()}`, *rootCh) |
  for (Root <- rootCh) {{
    Root!("resolve", "component:account-factory", 1, *afCh) |
    for (@(true, afUri, _) <- afCh) {{
      rl!(afUri, *ac) |
      for (AccountFactory <- ac) {{
        AccountFactory!("getOrCreate", "{public_hex}".hexToBytes(), *oCh) |
        for (@(true, account, ownerId) <- oCh) {{
          @account!("balance", *deployerId, "REV", *bCh) |
          for (@b <- bCh) {{ deployId!(b) }}
        }}
      }}
    }}
  }}
}}
"""
    reply = one(term)
    if not (isinstance(reply, tuple) and reply[0] is True):
        raise AssertionError(f"account balance: {reply!r}")
    return reply


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


def main() -> int:
    print("unit-booking")
    if not require_devnet("unit-booking"):
        return 2

    now = block_height()
    print(f"  height {now}")
    terms = {
        "assetId": "REV",
        "pricePerSlot": 100,
        "deposit": 50,
        "minSlots": 1,
        "maxSlots": 4,
        "instantBook": True,
        "contiguous": False,
        "holdBlocks": 10,
        "depositReviewBlocks": 0,
        "disputeWindowBlocks": 0,
        "noShowBlocks": 0,
        "fullRefundBeforeBlocks": 0,
        "partialRefundBps": 0,
        "partialRefundBeforeBlocks": 0,
        "feeBps": 0,
        "maxLeadBlocks": 1000000,
        "maxSlotBlocks": 1000,
        "maxUnits": 2,
        "eventTransferCutoffBlocks": 0,
    }

    provider = one(call_uri(
        root_uri(), "resolve", '"component:provider-book", 1'))
    if not (isinstance(provider, tuple) and provider[0] is True):
        raise AssertionError(f"resolve provider book: {provider!r}")
    provider_book_uri = provider[1]
    registered = one(call_uri(
        provider_book_uri, "register", '*dId, {"name": "Ana"}'))
    expect_ok("provider register", registered)
    provider_uri = registered[1]
    print(f"  provider {provider_uri}")

    factory = one(call_uri(
        root_uri(), "resolve", '"component:listing-factory", 1'))
    if not (isinstance(factory, tuple) and factory[0] is True):
        raise AssertionError(f"resolve listing factory: {factory!r}")
    listing_factory = factory[1]
    listing_id = f"listing:{now}"
    created = one(call_uri(
        listing_factory, "create",
        f'*dId, "{provider_uri}", "{listing_id}", "robot", {json.dumps(terms)}'))
    expect_ok("listing create", created)
    listing_uri = created[1]
    print(f"  listing {listing_uri}")

    slots_a = [{"slotId": "s1", "startHeight": now + 100, "endHeight": now + 200}]
    slots_b = [{"slotId": "s1", "startHeight": now + 150, "endHeight": now + 250}]
    unit_a = one(call_uri(
        listing_uri, "ensureUnit",
        f'*dId, "unit-a:{now}", {{"name": "machine"}}, {json.dumps(slots_a)}, 10'))
    expect_ok("unit A create", unit_a)
    unit_b = one(call_uri(
        listing_uri, "ensureUnit",
        f'*dId, "unit-b:{now}", {{"name": "operator"}}, {json.dumps(slots_b)}, 20'))
    expect_ok("unit B create", unit_b)
    unit_a_uri, unit_b_uri = unit_a[1], unit_b[1]

    activated = one(call_uri(listing_uri, "activate", "*dId"))
    expect_ok("listing activate", activated)

    guest = one(call_uri(root_uri(), "resolve", '"component:guest-book", 1'))
    if not (isinstance(guest, tuple) and guest[0] is True):
        raise AssertionError(f"resolve guest book: {guest!r}")
    guest_book_uri = guest[1]
    ledger = one(call_uri(guest_book_uri, "getOrCreate", "*dId"))
    expect_ok("guest ledger", ledger)
    ledger_uri = ledger[1]

    public_hex = node.load_keys()["deployer"]["public"]
    funded = one(fund_account(public_hex))
    if not (isinstance(funded, list) and len(funded) == 2 and funded[1] >= 80):
        raise AssertionError(f"fund account: {funded!r}")
    print(f"  funded {funded[1]}")

    balance_before = account_balance(public_hex)[1]
    print(f"  balance {balance_before}")

    unit_slots = [
        {"unitUri": unit_a_uri, "slots": ["s1"],
         "checkInHeight": now + 110, "checkOutHeight": now + 190},
        {"unitUri": unit_b_uri, "slots": ["s1"],
         "checkInHeight": now + 160, "checkOutHeight": now + 240},
    ]
    booked = one(call_uri(
        ledger_uri, "book",
        f'*dId, "{listing_uri}", {json.dumps(unit_slots)}, "op:{now}", Nil'))
    expect_ok("book", booked)
    booking_uri = booked[2]
    print(f"  booking {booking_uri}")

    info = one(call_uri(booking_uri, "info"))
    expect_ok("booking info", info)
    state = info[1]
    if state.get("rent") != 30:
        raise AssertionError(f"rent: {state.get('rent')} (expected 30)")
    if state.get("checkInHeight") != now + 110:
        raise AssertionError(f"checkInHeight: {state.get('checkInHeight')!r}")
    if state.get("checkOutHeight") != now + 240:
        raise AssertionError(f"checkOutHeight: {state.get('checkOutHeight')!r}")
    if state.get("status") != "confirmed":
        raise AssertionError(f"status: {state.get('status')!r}")

    bond_info = one(call_uri(state["bondUri"], "info"))
    expect_ok("bond info", bond_info)
    if bond_info[1].get("expected") != 80:
        raise AssertionError(f"bond expected: {bond_info[1]!r}")

    balance_after_book = account_balance(public_hex)[1]
    if balance_before - balance_after_book != 80:
        raise AssertionError(
            f"escrow debit: {balance_before} -> {balance_after_book}")
    print("  ok escrow debit 80")

    cancelled = one(call_uri(
        booking_uri, "cancel", f'*dId, "cancel:{now}"'))
    expect_ok("cancel", cancelled)
    balance_after_cancel = account_balance(public_hex)[1]
    if balance_after_cancel != balance_before:
        raise AssertionError(
            f"refund: {balance_before} -> {balance_after_cancel}")
    bond_after = one(call_uri(state["bondUri"], "info"))
    expect_ok("bond after cancel", bond_after)
    if bond_after[1].get("paid") is not True or bond_after[1].get("remaining") != 0:
        raise AssertionError(f"bond after cancel: {bond_after[1]!r}")
    print("  ok refund and bond closed")

    print("  ok rent, windows and bond total")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
