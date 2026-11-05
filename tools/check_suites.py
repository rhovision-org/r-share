#!/usr/bin/env python3
"""Keep the suite runner and the documentation in step.

Every `harness/unit_*.py` must be mapped in `run_suites.py`, `DEFAULT` must contain every
mapped suite, and a backticked `unit_*` name in the docs or spec must be a mapped suite or
the `unit-all` make target.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "harness"))

import run_suites  # noqa: E402

ALLOWED_TARGETS = {"unit_all"}
OPT_IN_PREFIX = "audit_"
SCAN = [ROOT / "README.md", ROOT / "docs", ROOT / "spec", ROOT / "audit"]
SUITE_REF = re.compile(r"`(unit[-_][a-z0-9_-]+)`")


def mapped() -> set[str]:
    return {script.removesuffix(".py") for script in run_suites.SUITES.values()}


def check_files() -> list[str]:
    errors = []
    files = {path.stem for path in (ROOT / "harness").glob("unit_*.py")}
    missing = sorted(files - mapped())
    if missing:
        errors.append(f"suite files not mapped in run_suites.SUITES: {missing}")
    keys = set(run_suites.SUITES)
    default_expected = {key for key in keys if not key.startswith(OPT_IN_PREFIX)}
    missing_default = sorted(default_expected - set(run_suites.DEFAULT))
    if missing_default:
        errors.append(f"suites missing from run_suites.DEFAULT: {missing_default}")
    unknown_default = sorted(set(run_suites.DEFAULT) - keys)
    if unknown_default:
        errors.append(f"DEFAULT names unknown suites: {unknown_default}")
    if len(run_suites.DEFAULT) != len(default_expected):
        errors.append("run_suites.DEFAULT is not the full non-audit SUITES map")
    return errors


def iter_files():
    for item in SCAN:
        if item.is_file():
            yield item
        elif item.is_dir():
            yield from item.rglob("*.md")
            yield from item.rglob("*.yaml")


def check_docs() -> list[str]:
    errors = []
    for path in iter_files():
        for number, line in enumerate(path.read_text().splitlines(), 1):
            for match in SUITE_REF.finditer(line):
                name = match.group(1).replace("-", "_")
                if name in ALLOWED_TARGETS:
                    continue
                if name not in mapped():
                    errors.append(
                        f"{path.relative_to(ROOT)}:{number}: unknown suite `{name}`")
    return errors


def main() -> int:
    errors = check_files() + check_docs()
    if errors:
        for error in errors:
            print(f"check-suites: {error}", file=sys.stderr)
        return 1
    print(f"check-suites: {len(mapped())} suites mapped and documented")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
