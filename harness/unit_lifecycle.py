#!/usr/bin/env python3
"""Lifecycle suite: the dispute trap, the arbitration gate and the default resolution.

The booking fixture uses a stub Bond, so this is a state-machine test. It does not prove
slot settlement or ERTP conservation.
"""

from __future__ import annotations

from assertions import expect_ok, expect_refusal, one
from booking_fixture import booking_term, dummy_bond
from preflight import require_devnet

import deploy as node


def main() -> int:
    print("unit-lifecycle")
    if not require_devnet("unit-lifecycle"):
        return 2

    owner = node.load_keys()["deployer"]["address"]
    bond = dummy_bond()
    print(f"  bond {bond}")

    refused = one(booking_term(
        bond, owner, False,
        'B!("dispute", *dId, "opD", "hash", *out) | '
        'for (@r <- out) { deployId!(r) }',
    ))
    expect_refusal("dispute from requested", refused, "closed")

    arbitrate = one(booking_term(
        bond, owner, True,
        'B!("arbitrate", 1, 0, [], "opA", *out) | '
        'for (@r <- out) { deployId!(r) }',
    ))
    expect_refusal("arbitrate outside dispute", arbitrate, "closed")

    resolved = one(booking_term(
        bond, owner, True,
        'B!("checkIn", *dId, "op1", *c1) | '
        'for (@r1 <- c1) { match r1 { '
        '  (true, Nil) => { '
        '    B!("checkOut", *dId, "op2", *c2) | '
        '    for (@r2 <- c2) { match r2 { '
        '      (true, Nil) => { '
        '        B!("dispute", *dId, "op3", "hash", *c3) | '
        '        for (@r3 <- c3) { match r3 { '
        '          (true, Nil) => { B!("resolveDispute", "op4", *out) | '
        '                           for (@r <- out) { deployId!(r) } } '
        '          _ => { deployId!(("dispute", r3)) } '
        '        } } '
        '      } '
        '      _ => { deployId!(("checkOut", r2)) } '
        '    } } '
        '  } '
        '  _ => { deployId!(("checkIn", r1)) } '
        '} }',
    ))
    expect_ok("dispute -> resolveDispute", resolved)

    evidence = one(booking_term(
        bond, owner, True,
        'B!("checkIn", *dId, "op1", *c1) | '
        'for (@r1 <- c1) { match r1 { '
        '  (true, Nil) => { '
        '    B!("checkOut", *dId, "op2", *c2) | '
        '    for (@r2 <- c2) { match r2 { '
        '      (true, Nil) => { '
        '        B!("dispute", *dId, "op3", "hash", *c3) | '
        '        for (@r3 <- c3) { match r3 { '
        '          (true, Nil) => { B!("info", *out) | '
        '                           for (@r <- out) { deployId!(r) } } '
        '          _ => { deployId!(("dispute", r3)) } '
        '        } } '
        '      } '
        '      _ => { deployId!(("checkOut", r2)) } '
        '    } } '
        '  } '
        '  _ => { deployId!(("checkIn", r1)) } '
        '} }',
    ))
    expect_ok("evidence read", evidence)
    state = evidence[1]
    if state.get("disputeHash") != "hash" or state.get("disputeAtHeight") <= 0:
        raise AssertionError(f"evidence anchors: {state!r}")
    print("  ok evidence anchors")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
