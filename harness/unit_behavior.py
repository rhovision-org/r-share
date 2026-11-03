#!/usr/bin/env python3
"""Golden-transcript check for the lifecycle state machine.

The transcript is normalized: refusals, booleans and the evidence hash, never heights,
URIs or deploy ids. `--write` records the current transcript after an intentional change.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from preflight import require_devnet
from unit_lifecycle import run

ROOT = Path(__file__).resolve().parent.parent
GOLDEN = ROOT / "harness" / "scenarios" / "golden" / "lifecycle.json"


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--write", action="store_true")
    args = ap.parse_args(argv)

    if not require_devnet("unit-behavior"):
        return 2
    transcript = run()

    if args.write:
        GOLDEN.parent.mkdir(parents=True, exist_ok=True)
        GOLDEN.write_text(json.dumps(transcript, indent=2) + "\n")
        print(f"unit-behavior: wrote {GOLDEN.relative_to(ROOT)}")
        return 0

    if not GOLDEN.is_file():
        print(f"unit-behavior: missing {GOLDEN}; run with --write", file=sys.stderr)
        return 1
    golden = json.loads(GOLDEN.read_text())
    if transcript != golden:
        print(f"unit-behavior: transcript changed\n  golden {golden}\n  now    {transcript}",
              file=sys.stderr)
        return 1
    print("unit-behavior: transcript matches")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
