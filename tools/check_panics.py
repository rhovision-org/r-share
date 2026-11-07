#!/usr/bin/env python3
"""Check that every `1/0` in a deploy term is registered.

`audit/registers/panics.yaml` records the intentional abort arms. Sites are keyed by
file, enclosing contract and ordinal within that contract, so moving lines does not
churn the register. `1/0` in the trailing Root registration block is a register guard
and is counted per file.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
DEPLOY = ROOT / "contracts" / "src" / "deploy"
REGISTER = ROOT / "audit" / "registers" / "panics.yaml"
CONTRACT = re.compile(r"contract\s+([A-Za-z0-9_]+)")


def load() -> dict:
    if not REGISTER.is_file():
        raise SystemExit(f"check-panics: missing {REGISTER}")
    return yaml.safe_load(REGISTER.read_text())


def scan(path: Path) -> tuple[dict[tuple[str, int], str], int]:
    """Return ({(contract, ordinal): text}, register_guard_count)."""
    text = path.read_text()
    lines = text.splitlines()
    marker = text.rfind('Root!("register"')
    register_line = text[:marker].count("\n") + 1 if marker >= 0 else len(lines) + 1
    sites: dict[tuple[str, int], str] = {}
    seen: dict[str, int] = {}
    guards = 0
    for index, line in enumerate(lines, start=1):
        if "1/0" not in line:
            continue
        if index >= register_line:
            guards += 1
            continue
        contract = "?"
        for back in range(index - 1, -1, -1):
            match = CONTRACT.search(lines[back])
            if match:
                contract = match.group(1)
                break
        seen[contract] = seen.get(contract, 0) + 1
        sites[(contract, seen[contract])] = line.strip()
    return sites, guards


def main() -> int:
    doc = load()
    problems: list[str] = []
    registered = {
        (entry["file"], entry["contract"], int(entry["ordinal"]))
        for entry in doc.get("sites", [])
    }
    guards = {entry["file"]: int(entry["count"]) for entry in doc.get("register_guards", [])}
    found: set[tuple[str, str, int]] = set()
    for path in sorted(DEPLOY.glob("*.rho")):
        sites, guard_count = scan(path)
        for (contract, ordinal) in sites:
            found.add((path.name, contract, ordinal))
        expected = guards.get(path.name, 0)
        if guard_count != expected:
            problems.append(
                f"{path.name}: {guard_count} register guard(s), register says {expected}"
            )
    for site in sorted(found - registered):
        problems.append(f"unregistered 1/0: {site[0]}:{site[1]}#{site[2]}")
    for site in sorted(registered - found):
        problems.append(f"stale register entry: {site[0]}:{site[1]}#{site[2]}")
    if problems:
        for problem in problems:
            print(f"check-panics: {problem}")
        return 1
    print(f"check-panics: {len(found)} site(s) and {sum(guards.values())} register guard(s)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
