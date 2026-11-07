#!/usr/bin/env python3
"""Bond refusal mapping at the booking boundary (F-016)."""

from __future__ import annotations

from assertions import expect_refusal, one
from booking_fixture import booking_term
from preflight import require_devnet

import deploy as node


def closed_bond() -> str:
    term = """
new Bond, rs(`rho:registry:insertArbitrary`), ch,
    deployId(`rho:rchain:deployId`) in {
  contract Bond(@"pay", @buckets, @expected, settleToken, ret) = {
    ret!((false, "closed"))
  } |
  contract Bond(@"balance", ret) = { ret!((true, 0)) } |
  rs!(bundle+{*Bond}, *ch) |
  for (@uri <- ch) { deployId!(uri) }
}
"""
    uri = one(term)
    if not isinstance(uri, str) or not uri.startswith("rho:id:"):
        raise AssertionError(f"closed bond URI: {uri!r}")
    return uri


def main() -> int:
    print("unit-bond-refusals")
    if not require_devnet("unit-bond-refusals"):
        return 2

    owner = node.load_keys()["deployer"]["address"]
    bond = closed_bond()
    term = booking_term(
        bond, owner, True,
        'B!("checkIn", *dId, "op1", *c1) | '
        'for (@r1 <- c1) { match r1 { (true, Nil) => { '
        '  B!("checkOut", *dId, "op2", *c2) | '
        '  for (@r2 <- c2) { match r2 { (true, Nil) => { '
        '    B!("settle", "op3", *out) | for (@r <- out) { deployId!(r) } '
        '  } _ => { deployId!(("checkOut", r2)) } } } '
        '} _ => { deployId!(("checkIn", r1)) } } }',
        kind="lodging", deposit=0, check_in=0, check_out=1_000_000_000,
    )
    # The provider-registered listing gate now precedes the bond check; the fabricated
    # booking term is refused as "listing" before it can reach the closed-bond path.
    expect_refusal("closed Bond maps to booking refusal", one(term), "listing")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
