#!/usr/bin/env python3
"""Measure built Rholang terms against the deploy cap and depth budgets."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BUILD = ROOT / "contracts" / "build"
CATALOGUE = ROOT / "contracts" / "components.json"
MANIFEST = ROOT / "contracts" / "measure.json"

sys.path.insert(0, str(ROOT / "tools"))
from minify_rho import minify  # noqa: E402


def strip_strings(text: str) -> str:
    out = []
    in_string = False
    escaped = False
    for ch in text:
        if in_string:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == '"':
                in_string = False
            out.append(" ")
        elif ch == '"':
            in_string = True
            out.append(" ")
        else:
            out.append(ch)
    return "".join(out)


def depth_of(text: str) -> int:
    depth = peak = 0
    for ch in strip_strings(text):
        if ch in "([{":
            depth += 1
            peak = max(peak, depth)
        elif ch in ")]}":
            depth -= 1
    return peak


CONTRACT = re.compile(r"contract\s+([A-Za-z_][A-Za-z0-9_]*)")


def measure(path: Path) -> dict:
    raw = path.read_text()
    small = minify(raw)
    return {
        "bytes": len(small.encode("utf-8")),
        "depth": depth_of(small),
        "contracts": len(set(CONTRACT.findall(small))),
    }


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--write", action="store_true")
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args(argv)

    catalogue = json.loads(CATALOGUE.read_text())
    limits = catalogue.get("limits", {})
    max_bytes = limits.get("bytes", 57344)
    max_depth = limits.get("depth", 96)
    max_contracts = limits.get("contracts", 16)
    per_file = limits.get("per_file", {})

    actual = {p.name: measure(p) for p in sorted(BUILD.glob("*.rho"))}
    failures = []
    for name, m in actual.items():
        if m["bytes"] > max_bytes:
            failures.append(f"{name}: {m['bytes']} bytes > {max_bytes}")
        if m["depth"] > max_depth:
            failures.append(f"{name}: depth {m['depth']} > {max_depth}")
        if m["contracts"] > max_contracts:
            failures.append(f"{name}: {m['contracts']} contracts > {max_contracts}")
        if name in per_file and m["bytes"] > per_file[name]:
            failures.append(f"{name}: {m['bytes']} bytes > per-file {per_file[name]}")

    if args.check:
        if not MANIFEST.is_file():
            print("measure: missing contracts/measure.json; run `make measure`", file=sys.stderr)
            return 1
        expected = json.loads(MANIFEST.read_text())
        if expected.get("terms") != actual:
            print("measure: metrics drifted; run `make measure`", file=sys.stderr)
            return 1

    if args.write:
        MANIFEST.write_text(json.dumps({"limits": limits, "terms": actual}, indent=2) + "\n")
        print(f"measure: wrote {MANIFEST.relative_to(ROOT)}")

    if failures:
        for failure in failures:
            print(f"measure: {failure}", file=sys.stderr)
        return 1
    print(f"measure: {len(actual)} term(s) within budget")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
