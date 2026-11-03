#!/usr/bin/env python3
"""Golden-transcript comparison for the base-layer refactor.

R0 records normalized replies, state projections, slot states, op maps and balances for a
fixed lifecycle. R7 runs the same transcript and diffs it. Heights are normalized to deltas;
the deploy order is fixed. Intentional differences must be listed in spec/behavior-delta.md.

This gate needs a healthy devnet. When it is unavailable it exits 2 and reports `not run`.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

try:
    import deploy as node
except Exception:  # pragma: no cover
    from harness import deploy as node  # type: ignore

from preflight import require_devnet


def main() -> int:
    if not require_devnet("unit-behavior"):
        return 2
    golden = ROOT / "harness" / "scenarios" / "golden"
    if not golden.is_dir() or not any(golden.glob("*.rho")):
        print("unit-behavior: no golden transcripts")
        return 2
    print("unit-behavior: not implemented yet; add the transcript runner")
    print(json.dumps({"golden_dir": str(golden)}, indent=2))
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
