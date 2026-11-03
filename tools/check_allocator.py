#!/usr/bin/env python3
"""Guard the allocator cut-over: no per-slot contracts and no raw state reply."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEPLOY = ROOT / "contracts" / "src" / "deploy"
FORBIDDEN = ("SlotContract", "slotState", "holdAll", "confirmAll", "releaseAll",
             "UnitContract")


def main() -> int:
    problems = []
    for path in sorted(DEPLOY.glob("*.rho")):
        for number, line in enumerate(path.read_text().splitlines(), start=1):
            code = line.split("//", 1)[0]
            for word in FORBIDDEN:
                if word in code:
                    problems.append(f"{path.relative_to(ROOT)}:{number}: {word}")
    listing = (DEPLOY / "14_listing_contract.rho").read_text()
    if 'ListingContract(@"info", ret2)' in listing:
        start = listing.index('ListingContract(@"info", ret2)')
        body = listing[start:start + 400]
        if 'delete("slots")' not in body:
            problems.append("14_listing_contract.rho: info must redact slots")
    if 'ret2!((true, st))' in listing:
        problems.append("14_listing_contract.rho: raw listingState reply")
    if problems:
        for problem in problems:
            print(f"check-allocator: {problem}", file=sys.stderr)
        return 1
    print("check-allocator: no slot contracts or raw state replies")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
