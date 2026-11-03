#!/usr/bin/env python3
"""GuestBook suite: getOrCreate is stable and the existing-guest path does not rewrite."""

from __future__ import annotations

from assertions import component_uri, expect, one
from preflight import require_devnet


def get_or_create(uri: str) -> str:
    return f"""
new rl(`rho:registry:lookup`), ch, ret,
    dId(`rho:rchain:deployerId`), deployId(`rho:rchain:deployId`) in {{
  rl!(`{uri}`, *ch) |
  for (C <- ch) {{
    C!("getOrCreate", *dId, *ret) |
    for (@r <- ret) {{ deployId!(r) }}
  }}
}}
"""


def ledger_of(uri: str, guest_id: str) -> str:
    return f"""
new rl(`rho:registry:lookup`), ch, ret,
    deployId(`rho:rchain:deployId`) in {{
  rl!(`{uri}`, *ch) |
  for (C <- ch) {{
    C!("ledger", "{guest_id}", *ret) |
    for (@r <- ret) {{ deployId!(r) }}
  }}
}}
"""


def main() -> int:
    print("unit-guestbook")
    if not require_devnet("unit-guestbook"):
        return 2

    book = component_uri("20_booking.rho")
    first = one(get_or_create(book))
    if not (isinstance(first, tuple) and first[0] is True and len(first) == 3):
        raise AssertionError(f"first getOrCreate: {first!r}")
    _, ledger_uri, guest_id = first
    print(f"  ledger {ledger_uri}")

    second = one(get_or_create(book))
    if second != first:
        raise AssertionError(f"second getOrCreate changed the ledger: {second!r}")

    read_back = one(ledger_of(book, guest_id))
    expect("ledger read-back", read_back, (True, ledger_uri))
    print("  ok stable guest ledger")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
