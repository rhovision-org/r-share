#!/usr/bin/env python3
"""Forbid bundles over externally resolved names and write-unfriendly bundle modes."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BUILD = ROOT / "contracts" / "build"
BUNDLE = re.compile(r"bundle\+\s*\{\s*\*([A-Za-z_][A-Za-z0-9_]*)\s*\}")


def main() -> int:
    errors = []
    for path in sorted(BUILD.glob("*.rho")):
        text = path.read_text()
        for mode in ("bundle0", "bundle-"):
            if mode in text:
                errors.append(f"{path.name}: {mode} is not allowed; use bundle+ for registration")
        for name in BUNDLE.findall(text):
            local = re.search(
                rf"(contract\s+{re.escape(name)}\s*[\(\"]|\bnew\b[^;{{]*\b{re.escape(name)}\b)",
                text,
            )
            if not local:
                errors.append(
                    f"{path.name}: bundle+{{*{name}}} captures a name that is not defined "
                    "locally (never bundle a registry-resolved capability)"
                )
    if errors:
        for error in errors:
            print(f"check-captures: {error}", file=sys.stderr)
        return 1
    print("check-captures: no forbidden bundles")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
