#!/usr/bin/env python3
"""Conservation suite entry point used by `run_suites.py`.

The scenarios, closed-set snapshots and negative control live in `conservation.py`.
"""

from __future__ import annotations

from conservation import main as conservation_main


def main() -> int:
    print("unit-conservation (per-scenario closed set)")
    return conservation_main()


if __name__ == "__main__":
    raise SystemExit(main())
