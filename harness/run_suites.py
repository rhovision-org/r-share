#!/usr/bin/env python3
"""Run behavioural suites and record an honest pass / fail / not-run result.

Exit codes: 0 all pass, 1 any failure, 2 any not-run and no failure. The result file is
`deployments/suite-results.json` (gitignored) and is uploaded by CI.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RESULTS = ROOT / "deployments" / "suite-results.json"

SUITES = {
    "root": "unit_root.py",
    "payments": "unit_payments.py",
    "lifecycle": "unit_lifecycle.py",
    "claims": "unit_claims.py",
    "pricing": "unit_pricing.py",
    "guestbook": "unit_guestbook.py",
    "booking": "unit_booking.py",
    "pause": "unit_pause.py",
    "allocator": "unit_allocator.py",
    "admin": "unit_admin.py",
    "events": "unit_events.py",
    "generality": "unit_generality.py",
    "provider": "unit_provider.py",
    "listing": "unit_listing.py",
    "behavior": "unit_behavior.py",
    "conservation": "unit_conservation.py",
    "reset_safety": "unit_reset_safety.py",
    "settle_release": "unit_settle_release.py",
    "no_show_contest": "unit_no_show_contest.py",
    "event_timing": "unit_event_timing.py",
    "admin_keys": "unit_admin_keys.py",
    "pause_scope": "unit_pause_scope.py",
    "cell_liveness": "unit_cell_liveness.py",
    "factory_auth": "unit_factory_auth.py",
    "provider_index": "unit_provider_index.py",
    "cutoffs": "unit_cutoffs.py",
    "bond_refusals": "unit_bond_refusals.py",
    "cover": "unit_cover.py",
    "cover_policy": "unit_cover_policy.py",
    "cover_booking": "unit_cover_booking.py",
    "cover_disabled": "unit_cover_disabled.py",
    "hold_deposit": "unit_hold_deposit.py",
}
EXPERIMENTAL = {"cover", "cover_policy", "cover_booking"}
SAFE_ONLY = {"cover_disabled"}
AUDIT = set()
SAFE_CORE = sorted(set(SUITES) - EXPERIMENTAL - SAFE_ONLY - AUDIT)
DEFAULT = SAFE_CORE + sorted(SAFE_ONLY)


def run(name: str) -> dict:
    script = ROOT / "harness" / SUITES[name]
    started = time.time()
    proc = subprocess.run(
        [sys.executable, str(script)],
        cwd=ROOT,
        text=True,
        capture_output=True,
    )
    output = (proc.stdout or "") + (proc.stderr or "")
    if proc.returncode == 0:
        status = "pass"
    elif proc.returncode == 2:
        status = "not_run"
    else:
        status = "fail"
    print(f"--- {name}: {status} ({proc.returncode}) ---")
    print(output.rstrip())
    return {
        "suite": name,
        "script": SUITES[name],
        "status": status,
        "returncode": proc.returncode,
        "seconds": round(time.time() - started, 3),
        "output": output[-4000:],
    }


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--suites", default=None,
                    help="comma-separated suite names (default: profile suites)")
    ap.add_argument("--experimental", action="store_true",
                    help="run SAFE_CORE plus the experimental cover suites")
    ap.add_argument("--list", action="store_true")
    args = ap.parse_args(argv)

    if args.list:
        for name, script in SUITES.items():
            print(f"{name:12} {script}")
        return 0

    default_names = SAFE_CORE + sorted(EXPERIMENTAL) if args.experimental else DEFAULT
    selected = args.suites if args.suites is not None else ",".join(default_names)
    names = [name.strip() for name in selected.split(",") if name.strip()]
    unknown = [name for name in names if name not in SUITES]
    if unknown:
        print(f"run_suites: unknown suites {unknown}", file=sys.stderr)
        return 1

    results = [run(name) for name in names]
    payload = {
        "node": os.environ.get("RSHARE_NODE", "http://localhost:40403"),
        "profile": "cover-demo" if args.experimental else "safe",
        "experimental": args.experimental,
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "results": results,
    }
    RESULTS.parent.mkdir(parents=True, exist_ok=True)
    RESULTS.write_text(json.dumps(payload, indent=2) + "\n")
    print(f"run_suites: wrote {RESULTS.relative_to(ROOT)}")

    if any(item["status"] == "fail" for item in results):
        return 1
    if any(item["status"] == "not_run" for item in results):
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
