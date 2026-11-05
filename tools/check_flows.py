#!/usr/bin/env python3
"""Check the cornerstone flow manifest against source arms and Lean declarations.

By default, a `proved-model` rule whose target is a definition or constructor is a
warning: the audit records it, but the historical manifest still passes. `--strict` turns
those warnings into failures; that is the target for the L0 proof-quality gate.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
FLOWS = ROOT / "spec" / "flows"
LEAN = ROOT / "spec" / "lean"
STATUSES = {"proved-model", "proved-model-bounded", "modeled", "assumption",
            "proved-tied", "owed", "open"}
ARM_RE = re.compile(r"^(?P<contract>[A-Za-z0-9_]+)@\"(?P<arm>[^\"]+)\"$")
DECL_RE = re.compile(
    r"^[ \t]*(?:@\[[^\]]*\][ \t]*)*(theorem|lemma|def|inductive|structure)[ \t]+"
    r"([A-Za-z0-9_]+)",
    re.M,
)


def lean_declarations() -> tuple[set[str], dict[str, str]]:
    names: set[str] = set()
    kinds: dict[str, str] = {}
    for path in LEAN.rglob("*.lean"):
        if ".lake" in path.parts:
            continue
        text = path.read_text()
        for kind, name in DECL_RE.findall(text):
            names.add(name)
            if kind in {"theorem", "lemma"}:
                kinds[name] = "theorem"
            else:
                kinds.setdefault(name, kind)
        for match in re.finditer(r"^\s*\|\s*([A-Za-z0-9_]+)\s*:", text, re.M):
            names.add(match.group(1))
            kinds.setdefault(match.group(1), "constructor")
    return names, kinds


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--strict", action="store_true",
                    help="treat proved-model targets that are not theorems as failures")
    args = ap.parse_args(argv)

    problems: list[str] = []
    warnings: list[str] = []
    names, kinds = lean_declarations()
    rules: list[dict] = []
    for path in sorted(FLOWS.glob("*.yaml")):
        doc = yaml.safe_load(path.read_text())
        for glob in doc.get("source_globs", []):
            if not (ROOT / glob).exists():
                problems.append(f"{path.name}: missing source_glob {glob}")
        for key, value in (doc.get("invariants") or {}).items():
            if not value.get("lean_status") and value.get("statement"):
                problems.append(f"{path.name}: invariant {key} has no lean_status")
            if value.get("lean"):
                short = value["lean"].rsplit(".", 1)[-1]
                if short not in names:
                    problems.append(f"{path.name}: invariant {key} Lean name "
                                    f"{value['lean']} not found")
        rules.extend(doc.get("rules", []))

    arms: set[str] = set()
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
            elif status == "proved-model" and kinds.get(short) != "theorem":
                message = (f"{rule.get('id')}: proved-model target {lean} is a "
                           f"{kinds.get(short, 'declaration')}, not a theorem")
                warnings.append(message)
                if args.strict:
                    problems.append(message)
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

    core: set[str] = set()
    for path in sorted(FLOWS.glob("*.yaml")):
        core.update(yaml.safe_load(path.read_text()).get("core_arms", []))
    for arm in core:
        if arm not in arms:
            problems.append(f"core arm missing from flows: {arm}")
        else:
            rule = next(r for r in rules if r.get("arm") == arm)
            if rule.get("lean_status") != "proved-model":
                problems.append(f"core arm not proved-model: {arm}")

    for warning in warnings:
        print(f"check-flows: warning: {warning}", file=sys.stderr)
    for problem in problems:
        print(f"check-flows: {problem}", file=sys.stderr)
    if problems:
        return 1
    print(f"check-flows: {len(rules)} rules, {len(arms)} arms, "
          f"{len(core)} core arms proved-model, {len(warnings)} proof-quality warnings")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
