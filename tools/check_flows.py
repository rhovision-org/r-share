#!/usr/bin/env python3
"""Check the cornerstone flow manifest against the source arms and the Lean names."""

from __future__ import annotations

import re
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
FLOWS = ROOT / "spec" / "flows"
LEAN = ROOT / "spec" / "lean"
STATUSES = {"proved-model", "proved-tied", "owed", "open"}
ARM_RE = re.compile(r"^(?P<contract>[A-Za-z0-9_]+)@\"(?P<arm>[^\"]+)\"$")
DECL_RE = re.compile(r"(?:theorem|def|inductive|structure)\s+([A-Za-z0-9_]+)")


def lean_names() -> set[str]:
    names = set()
    for path in LEAN.rglob("*.lean"):
        if ".lake" in path.parts:
            continue
        text = path.read_text()
        names.update(DECL_RE.findall(text))
        for match in re.finditer(r"^\s*\|\s*([A-Za-z0-9_]+)\s*:", text, re.M):
            names.add(match.group(1))
    return names


def main() -> int:
    problems = []
    names = lean_names()
    rules = []
    for path in sorted(FLOWS.glob("*.yaml")):
        doc = yaml.safe_load(path.read_text())
        rules.extend(doc.get("rules", []))

    arms = set()
    for rule in rules:
        status = rule.get("lean_status", "owed")
        if status not in STATUSES:
            problems.append(f"{rule.get('id')}: unknown lean_status {status}")
        if status == "proved-tied" and not rule.get("corpus"):
            problems.append(f"{rule.get('id')}: proved-tied needs a corpus")
        lean = rule.get("lean")
        if lean:
            short = lean.rsplit(".", 1)[-1]
            if short not in names:
                problems.append(f"{rule.get('id')}: Lean name {lean} not found")
        arm = rule.get("arm")
        if not arm:
            continue
        match = ARM_RE.match(arm)
        if not match:
            problems.append(f"{rule.get('id')}: bad arm {arm!r}")
            continue
        arms.add(arm)
        source = ROOT / rule.get("source", "")
        if not source.is_file():
            problems.append(f"{rule.get('id')}: missing source {source}")
            continue
        needle = f'contract {match.group("contract")}(@"{match.group("arm")}"'
        if needle not in source.read_text():
            problems.append(f"{rule.get('id')}: {arm} not found in {source.name}")

    core = set()
    for path in sorted(FLOWS.glob("*.yaml")):
        core.update(yaml.safe_load(path.read_text()).get("core_arms", []))
    for arm in core:
        if arm not in arms:
            problems.append(f"core arm missing from flows: {arm}")
        else:
            rule = next(r for r in rules if r.get("arm") == arm)
            if rule.get("lean_status") != "proved-model":
                problems.append(f"core arm not proved-model: {arm}")

    for problem in problems:
        print(f"check-flows: {problem}", file=sys.stderr)
    if problems:
        return 1
    print(f"check-flows: {len(rules)} rules, {len(arms)} arms, "
          f"{len(core)} core arms proved-model")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
