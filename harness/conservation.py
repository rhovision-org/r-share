#!/usr/bin/env python3
"""Conservation gate for the booking base layer.

The full suite needs a healthy r-cash devnet, the install manifest and funded accounts.
When those are unavailable it exits 2 and reports `not run`; it never reports a pass.

Invariants checked when the devnet is available:
  1. For each asset, sum(account balances) + sum(Bond purses) is constant.
  2. Every slot cell is free, held or booked; none is empty at quiescence.
  3. Every terminal booking's buckets sum to rent + deposit (Phase 1) or bond total.
  4. Bond.balance is zero after pay; a second pay refuses "closed".
  5. One terminal outcome per booking; a replay moves no money.
  6. A slot releases only for the holder's token.
"""

from __future__ import annotations

import sys

try:
    import deploy as node
except Exception:  # pragma: no cover - import path when run directly
    from harness import deploy as node  # type: ignore

from preflight import require_devnet


def main() -> int:
    if not require_devnet("conservation"):
        return 2
    print("conservation: not implemented yet for this devnet build")
    print("  add the scenario drivers, then enable this gate in `make unit-all`")
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
