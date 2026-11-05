#!/usr/bin/env python3
"""Measure the structural metrics the refactor plan tracks.

Bytes, depth and contract counts come from contracts/measure.json (produced by
measure_contracts.py). Component, cell, arm and resolve-site counts are read from the
catalogue and the deploy source.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MEASURE = ROOT / "contracts" / "measure.json"
CATALOGUE = ROOT / "contracts" / "components.json"
DEPLOY = ROOT / "contracts" / "src" / "deploy"

ARM = re.compile(r"contract\s+\w+\s*\(\s*@\"")
ROOT_RESOLVE = re.compile(r'Root!\("resolve"')
RL = re.compile(r"\brl!\(")
RS = re.compile(r"\brs!\(")


def collect() -> dict:
    measure = json.loads(MEASURE.read_text())
    terms = measure.get("terms", {})
    catalogue = json.loads(CATALOGUE.read_text())
    components = catalogue.get("components", {})

    cells = set()
    for entry in components.values():
        cells.update(entry.get("state_cells", []))

    arms = root_resolves = rl_sites = rs_sites = 0
    for path in sorted(DEPLOY.glob("*.rho")):
        text = path.read_text()
        arms += len(ARM.findall(text))
        root_resolves += len(ROOT_RESOLVE.findall(text))
        rl_sites += len(RL.findall(text))
        rs_sites += len(RS.findall(text))

    return {
        "deploy_terms": len(terms),
        "bytes_total": sum(t["bytes"] for t in terms.values()),
        "bytes_max": max((t["bytes"] for t in terms.values()), default=0),
        "depth_max": max((t["depth"] for t in terms.values()), default=0),
        "contracts_max": max((t["contracts"] for t in terms.values()), default=0),
        "components": len(components),
        "state_cell_types": len(cells),
        "arm_declarations": arms,
        "root_resolve_sites": root_resolves,
        "registry_lookup_sites": rl_sites,
        "registry_insert_sites": rs_sites,
        "terms_detail": {
            name: {"bytes": t["bytes"], "depth": t["depth"], "contracts": t["contracts"]}
            for name, t in sorted(terms.items())
        },
    }


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--write", type=Path)
    ap.add_argument("--write-baseline", action="store_true",
                    help="write contracts/metrics-baseline.json from current metrics")
    ap.add_argument("--compare", type=Path)
    ap.add_argument("--allow-drift", action="store_true",
                    help="report drift but return 0 (use only for an intentional refactor)")
    args = ap.parse_args(argv)

    current = collect()
    if args.write:
        args.write.write_text(json.dumps(current, indent=2) + "\n")
        print(f"metrics: wrote {args.write}")
        return 0

    if args.write_baseline:
        path = ROOT / "contracts" / "metrics-baseline.json"
        path.write_text(json.dumps(current, indent=2) + "\n")
        print(f"metrics: wrote {path.relative_to(ROOT)}")
        return 0

    if args.compare:
        if not args.compare.is_file():
            print(f"metrics: missing baseline {args.compare}", file=sys.stderr)
            return 1
        baseline = json.loads(args.compare.read_text())
        keys = [
            "deploy_terms", "bytes_total", "bytes_max", "depth_max", "contracts_max",
            "components", "state_cell_types", "arm_declarations",
            "root_resolve_sites", "registry_lookup_sites", "registry_insert_sites",
        ]
        print(f"{'metric':28} {'baseline':>10} {'current':>10} {'delta':>8}")
        drift = []
        for key in keys:
            base = baseline.get(key, 0)
            cur = current.get(key, 0)
            print(f"{key:28} {base:10} {cur:10} {cur - base:+8}")
            if cur != base:
                drift.append(f"{key}: baseline {base}, current {cur}")
        failures = []
        if current["bytes_max"] > 57344:
            failures.append(f"max term bytes {current['bytes_max']} > 57344")
        if current["depth_max"] > 96:
            failures.append(f"max depth {current['depth_max']} > 96")
        if current["contracts_max"] > 36:
            failures.append(f"max contracts {current['contracts_max']} > 36")
        if drift and not args.allow_drift:
            failures.extend(f"drift {item}" for item in drift)
        elif drift:
            print("metrics: drift allowed for this refactor", file=sys.stderr)
        for failure in failures:
            print(f"metrics: {failure}", file=sys.stderr)
        return 1 if failures else 0

    print(json.dumps(current, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
