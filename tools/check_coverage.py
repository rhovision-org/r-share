#!/usr/bin/env python3
"""Validate the class x claim coverage manifest. Only `passing` counts as coverage."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
COVERAGE = ROOT / "spec" / "coverage.yaml"
STATUSES = {"passing", "mapping", "design", "not_implemented"}
CLAIMS = {
    "buyerUseBreach", "providerDeliveryBreach", "noFaultCancellation",
    "noShow", "damage", "partialPerformance",
}


def parse(path: Path) -> list[dict]:
    classes = []
    current = None
    for line in path.read_text().splitlines():
        s = line.strip()
        if s.startswith("- name:"):
            current = {"name": s.split(":", 1)[1].strip(), "kind": "", "claims": {}}
            classes.append(current)
        elif current is not None and s.startswith("kind:"):
            current["kind"] = s.split(":", 1)[1].strip()
        elif current is not None and ":" in s and not s.startswith("#"):
            key, value = s.split(":", 1)
            key, value = key.strip(), value.strip()
            if value in STATUSES:
                current["claims"][key] = value
    return classes


def parse_evidence(path: Path) -> dict[str, str]:
    evidence: dict[str, str] = {}
    in_block = False
    for line in path.read_text().splitlines():
        s = line.strip()
        if s == "evidence:":
            in_block = True
            continue
        if not in_block:
            continue
        if not s or s.startswith("#"):
            continue
        if ":" not in s:
            in_block = False
            continue
        key, value = s.split(":", 1)
        evidence[key.strip()] = value.strip()
    return evidence


def main() -> int:
    if not COVERAGE.is_file():
        print(f"check-coverage: missing {COVERAGE}", file=sys.stderr)
        return 1
    classes = parse(COVERAGE)
    evidence = parse_evidence(COVERAGE)
    errors = []
    counts = {"passing": 0, "mapping": 0, "design": 0, "not_implemented": 0}
    if len(classes) != 5:
        errors.append(f"expected 5 classes, found {len(classes)}")
    for entry in classes:
        missing = CLAIMS - set(entry["claims"])
        extra = set(entry["claims"]) - CLAIMS
        if missing:
            errors.append(f"{entry['name']}: missing claims {sorted(missing)}")
        if extra:
            errors.append(f"{entry['name']}: unknown claims {sorted(extra)}")
        for claim, status in entry["claims"].items():
            counts[status] += 1
            if status == "passing" and not (
                evidence.get(f"{entry['name']}.{claim}") or evidence.get(claim)
            ):
                errors.append(f"{entry['name']}.{claim}: passing without evidence")
    for error in errors:
        print(f"check-coverage: {error}", file=sys.stderr)
    if errors:
        return 1
    total = sum(counts.values())
    print(f"check-coverage: {total} cells — passing {counts['passing']}, "
          f"mapping {counts['mapping']}, design {counts['design']}, "
          f"not implemented {counts['not_implemented']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
