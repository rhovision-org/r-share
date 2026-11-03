#!/usr/bin/env python3
"""Only known install placeholders may survive the build; unknown ones fail."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BUILD = ROOT / "contracts" / "build"
ALLOWED = {
    "__ROOT_URI__",
    "__RCASH_ROOT_URI__",
    "__PLATFORM_OWNER__",
    "__ADMINS__",
    "__THRESHOLD__",
    "__INSTALLER__",
}
PLACEHOLDER = re.compile(r"__[A-Z0-9_]+__")


def main() -> int:
    unknown = []
    for path in sorted(BUILD.glob("*.rho")):
        for match in PLACEHOLDER.finditer(path.read_text()):
            if match.group(0) not in ALLOWED:
                unknown.append(f"{path.name}: {match.group(0)}")
    if unknown:
        for item in unknown:
            print(f"check-placeholders: {item}", file=sys.stderr)
        return 1
    print("check-placeholders: all placeholders are install-rendered")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
