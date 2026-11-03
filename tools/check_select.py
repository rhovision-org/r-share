#!/usr/bin/env python3
"""Fail on `select`: the node desugars PChoice into parallel receives.

See rholang/src/normalizer.rs:256-274 in rchain-rust. There is no committed choice, so
`select` cannot arbitrate exclusivity.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TARGETS = [ROOT / "contracts" / "src", ROOT / "contracts" / "build"]


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
        elif ch == "/" and len(out) >= 1 and out[-1] == "/":
            # the previous '/' is part of the comment opener; keep it but blank the rest
            in_comment = True
            out[-1] = " "
            out.append(" ")
        else:
            out.append(ch)
    return "".join(out)


def main() -> int:
    bad = []
    for root in TARGETS:
        if not root.exists():
            continue
        for path in sorted(root.rglob("*.rho")):
            clean = strip_comments_and_strings(path.read_text())
            for match in re.finditer(r"\bselect\b", clean):
                line = clean.count("\n", 0, match.start()) + 1
                bad.append(f"{path.relative_to(ROOT)}:{line}")
    if bad:
        for item in bad:
            print(f"check-select: forbidden select at {item}", file=sys.stderr)
        return 1
    print("check-select: no select")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
