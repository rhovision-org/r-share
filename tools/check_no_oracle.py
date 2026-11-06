#!/usr/bin/env python3
"""Fail on an undeclared component key or an on-chain oracle/attestor contract."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEPLOY = ROOT / "contracts" / "src" / "deploy"
TRUST = ROOT / "spec" / "trust.md"

ALLOWED = {
    "component:root", "component:clock", "component:verifier", "component:codec",
    "component:admin-guard", "component:admin", "component:payments",
    "component:terms", "component:remedy",
    "component:provider-book", "component:unit-factory", "component:listing-factory",
    "component:bond-factory", "component:booking-factory",
    "component:guest-ledger-factory", "component:guest-book",
    "component:cover-terms", "component:cover-book", "component:cover-pool",
    "component:cover-policy-factory",
    "component:guest-ledger-cover", "component:booking-instance-factory",
}
REGISTER = re.compile(r'Root!\("register",\s*"([^"]+)"')
CONTRACT = re.compile(r"contract\s+([A-Za-z_][A-Za-z0-9_]*)")


def assumptions() -> list[str]:
    names = []
    inside = False
    for line in TRUST.read_text().splitlines():
        if line.startswith("## Assumptions"):
            inside = True
            continue
        if inside and line.startswith("## "):
            break
        if inside and line.startswith("- **"):
            names.append(line.split("**")[1])
    return names


def main() -> int:
    errors = []
    keys = set()
    for path in DEPLOY.glob("*.rho"):
        text = path.read_text()
        keys.update(REGISTER.findall(text))
        for name in CONTRACT.findall(text):
            lowered = name.lower()
            if "oracle" in lowered or "attestor" in lowered or "attestation" in lowered:
                errors.append(f"{path.name}: contract {name} looks like an attestor")
    for key in sorted(keys - ALLOWED):
        errors.append(f"undeclared component key {key}")
    names = assumptions()
    if len(names) != 3:
        errors.append(f"trust.md must declare exactly 3 assumptions, found {len(names)}")
    if errors:
        for error in errors:
            print(f"check-no-oracle: {error}", file=sys.stderr)
        return 1
    print(f"check-no-oracle: {len(keys)} keys, {len(names)} trust assumptions, no attestor")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
