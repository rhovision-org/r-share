#!/usr/bin/env python3
"""Placeholder for suites scheduled in the milestone plan but not yet implemented."""

from __future__ import annotations

import sys

PENDING = {
    "provider": "M2 provider registration and ownership refusals",
    "listing": "M2 listing creation, terms validation and unit creation",
    "booking": "M3 instant/request booking and escrow",
    "concurrency": "M3 same-block races and multi-slot atomicity",
    "lifecycle": "M4 accept/reject/cancel/checkout/settle/dispute",
    "events": "M5 ticket redeem, transfer and close",
    "admin": "M6 2-of-3 pause/delist/arbitration/rotation",
    "generality": "M7 bike/car/robot fixtures on the same path",
}


def main(argv: list[str]) -> int:
    if len(argv) != 2 or argv[1] not in PENDING:
        print(f"usage: {argv[0]} <{ '|'.join(PENDING) }>", file=sys.stderr)
        return 2
    print(f"unit-{argv[1]}: not implemented yet — {PENDING[argv[1]]}")
    print("See docs/TESTING.md for the milestone acceptance map.")
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
