#!/usr/bin/env python3
"""Validate the remedy table against the coverage manifest."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REMEDY = ROOT / "spec" / "remedy-table.md"
COVERAGE = ROOT / "spec" / "coverage.yaml"
CLASSES = [
    "buyerUseBreach", "providerDeliveryBreach", "noFaultCancellation",
    "noShow", "damage", "partialPerformance",
]


def main() -> int:
    if not REMEDY.is_file():
        print(f"check-remedy: missing {REMEDY}", file=sys.stderr)
        return 1
    text = REMEDY.read_text()
    errors = []
    for name in CLASSES:
        if f"`{name}`" not in text:
            errors.append(f"missing class {name}")
    for section in ("Test vectors", "Conservation and rounding", "Worked examples"):
        if section not in text:
            errors.append(f"missing section {section}")
    for word in ("TODO", "FIXME", "TBD"):
        if word in text:
            errors.append(f"contains {word}")
    coverage_text = COVERAGE.read_text() if COVERAGE.is_file() else ""
    for name in CLASSES:
        if name not in coverage_text:
            errors.append(f"coverage manifest omits {name}")
    for error in errors:
        print(f"check-remedy: {error}", file=sys.stderr)
    if errors:
        return 1
    print(f"check-remedy: {len(CLASSES)} classes, bounds and vectors present")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
