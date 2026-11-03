#!/usr/bin/env python3
"""One-booking conservation check.

This runs the end-to-end booking suite, which asserts that the escrow debit is exactly
rent + deposit, a full-refund cancel restores the balance, and the Bond ends paid with zero
remaining. The full per-asset harness in `conservation.py` is still a stub.
"""

from __future__ import annotations

from unit_booking import main as booking_main


def main() -> int:
    print("unit-conservation (one booking)")
    return booking_main()


if __name__ == "__main__":
    raise SystemExit(main())
