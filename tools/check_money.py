#!/usr/bin/env python3
"""Money is integer minor units. Forbid float literals in contracts."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TARGET = ROOT / "contracts" / "src"
FLOAT = re.compile(r"(?<![\w.])\d+\.\d+(?![\w.])")


def strip_comments_and_strings(text: str) -> str:
    out = []
    in_string = False
    escaped = False
    in_comment = False
    for ch in text:
        if in_comment:
            if ch == "\n":
                in_comment = False
                out.append(ch)
            else:
                out.append(" ")
            continue
        if in_string:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == '"':
                in_string = False
            out.append(" ")
            continue
        if ch == '"':
            in_string = True
            out.append(" ")
        elif ch == "/" and out and out[-1] == "/":
            in_comment = True
            out[-1] = " "
            out.append(" ")
        else:
            out.append(ch)
    return "".join(out)


def main() -> int:
    bad = []
    for path in sorted(TARGET.rglob("*.rho")):
        clean = strip_comments_and_strings(path.read_text())
        for match in FLOAT.finditer(clean):
            line = clean.count("\n", 0, match.start()) + 1
            bad.append(f"{path.relative_to(ROOT)}:{line}: {match.group(0)}")
    if bad:
        for item in bad:
            print(f"check-money: float literal at {item}", file=sys.stderr)
        return 1
    print("check-money: integer money only")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
