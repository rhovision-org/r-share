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
    groups = {
        "SAFE_CORE": set(run_suites.SAFE_CORE),
        "SAFE_ONLY": set(run_suites.SAFE_ONLY),
        "EXPERIMENTAL": set(run_suites.EXPERIMENTAL),
        "AUDIT": set(run_suites.AUDIT),
    }
    seen: dict[str, str] = {}
    for name, group in groups.items():
        unknown = sorted(group - keys)
        if unknown:
            errors.append(f"{name} names unknown suites: {unknown}")
        for key in group:
            if key in seen:
                errors.append(f"suite {key} is in both {seen[key]} and {name}")
            seen[key] = name
    missing_group = sorted(keys - set(seen))
    if missing_group:
        errors.append(f"suites not assigned to a profile group: {missing_group}")
    safe_expected = groups["SAFE_CORE"] | groups["SAFE_ONLY"]
    if set(run_suites.DEFAULT) != safe_expected:
        missing = sorted(safe_expected - set(run_suites.DEFAULT))
        extra = sorted(set(run_suites.DEFAULT) - safe_expected)
        if missing:
            errors.append(f"suites missing from run_suites.DEFAULT: {missing}")
        if extra:
            errors.append(f"DEFAULT contains non-safe suites: {extra}")
    if groups["EXPERIMENTAL"] & set(run_suites.DEFAULT):
        errors.append("experimental suites must not be in run_suites.DEFAULT")
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
