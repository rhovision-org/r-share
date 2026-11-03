#!/usr/bin/env python3
"""Regenerate the Pareto scorecard from measured metrics, coverage and trust assumptions."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "tools"))

from metrics import collect  # noqa: E402
from check_coverage import parse as parse_coverage  # noqa: E402
from check_no_oracle import assumptions  # noqa: E402

SCORECARD = ROOT / "spec" / "pareto-scorecard.json"


def counts() -> dict:
    classes = parse_coverage(ROOT / "spec" / "coverage.yaml")
    out = {"total": 0, "passing": 0, "mapping": 0, "design": 0, "not_implemented": 0}
    for entry in classes:
        for status in entry["claims"].values():
            out["total"] += 1
            out[status] += 1
    return out


def scorecard() -> dict:
    m = collect()
    c = counts()
    trust = assumptions()
    return {
        "generated_by": "tools/pareto.py",
        "status": "current measured; target rows estimated",
        "coverage": c,
        "trust_assumptions": trust,
        "trust_count": len(trust),
        "metrics": {
            "terms": m["terms"],
            "components": m["components"],
            "state_cell_types": m["state_cell_types"],
            "bytes_max": m["bytes_max"],
            "arm_declarations": m["arm_declarations"],
            "root_resolve_sites": m["root_resolve_sites"],
            "registry_lookup_sites": m["registry_lookup_sites"],
        },
        "candidates": [
            {
                "name": "current",
                "coverage_passing": c["passing"],
                "coverage_total": c["total"],
                "trust": len(trust),
                "bytes_max": m["bytes_max"],
                "components": m["components"],
                "state_cell_types": m["state_cell_types"],
                "measured": True,
            },
            {
                "name": "bilateral-bond",
                "coverage_target": 30,
                "trust": len(trust),
                "extra_bytes": "4-6 KiB (estimate)",
                "cells_per_booking": "buyer + provider legs + booking",
                "dispute_steps": "2 / 3",
                "measured": False,
            },
            {
                "name": "oracle-sla",
                "coverage_target": 35,
                "trust": len(trust) + 1,
                "extra_bytes": "10-15 KiB (estimate)",
                "dispute_steps": "3 / 5",
                "measured": False,
            },
            {
                "name": "bilateral+anchor",
                "coverage_target": 32,
                "trust": len(trust),
                "extra_bytes": "5-8 KiB over current (estimate)",
                "dispute_steps": "2 / 3",
                "measured": False,
            },
        ],
    }


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--write", action="store_true")
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args(argv)
    current = scorecard()
    if args.write:
        SCORECARD.write_text(json.dumps(current, indent=2) + "\n")
        print(f"pareto: wrote {SCORECARD.relative_to(ROOT)}")
        return 0
    if args.check:
        if not SCORECARD.is_file():
            print(f"pareto: missing {SCORECARD}", file=sys.stderr)
            return 1
        expected = json.loads(SCORECARD.read_text())
        if expected != current:
            print("pareto: scorecard is stale; run `make pareto`", file=sys.stderr)
            return 1
        print("pareto: scorecard current")
        return 0
    print(json.dumps(current, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
