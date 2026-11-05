#!/usr/bin/env python3
"""Compare the source arm inventory with the audit register and flow manifest.

This is deliberately narrower than a proof check: it fails on inventory drift and on a
flow rule that names an arm the source does not declare. Proof-target quality is checked by
the Lean/theorem registry work, not by pretending every declared name is a theorem.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
DEPLOY = ROOT / "contracts" / "src" / "deploy"
REGISTER = ROOT / "audit" / "registers" / "arms.yaml"
FLOWS = ROOT / "spec" / "flows"

ARM_RE = re.compile(r'contract\s+([A-Za-z0-9_]+)\s*\(\s*@?"([^"]+)"')
FLOW_ARM_RE = re.compile(r'^([A-Za-z0-9_]+)@"([^"]+)"$')


def source_arms() -> dict[tuple[str, str], tuple[str, int]]:
    found: dict[tuple[str, str], tuple[str, int]] = {}
    for path in sorted(DEPLOY.glob("*.rho")):
        text = path.read_text()
        for match in ARM_RE.finditer(text):
            component, arm = match.group(1), match.group(2)
            line = text.count("\n", 0, match.start()) + 1
            found[(component, arm)] = (path.name, line)
    return found


def register_arms() -> dict[tuple[str, str], dict]:
    document = yaml.safe_load(REGISTER.read_text())
    return {(entry["component"], entry["arm"]): entry
            for entry in document.get("arms", [])}


def flow_rules() -> list[dict]:
    rules: list[dict] = []
    for path in sorted(FLOWS.glob("*.yaml")):
        document = yaml.safe_load(path.read_text())
        for rule in document.get("rules", []):
            if rule.get("arm"):
                rules.append(rule)
    return rules


def main() -> int:
    if not REGISTER.is_file():
        print(f"check-arm-inventory: missing {REGISTER}", file=sys.stderr)
        return 1

    source = source_arms()
    register = register_arms()
    rules = flow_rules()
    problems: list[str] = []

    for key, (filename, line) in sorted(source.items()):
        if key not in register:
            problems.append(f"source arm not in audit/registers/arms.yaml: "
                            f"{key[0]}@\"{key[1]}\" ({filename}:{line})")

    for key, entry in sorted(register.items()):
        if key not in source:
            problems.append(f"register arm not in source: {key[0]}@\"{key[1]}\"")
        rule = entry.get("flow_rule")
        if rule:
            matching = [r for r in rules if r.get("id") == rule]
            if not matching:
                problems.append(f"{key[0]}@\"{key[1]}\": flow_rule {rule} not in manifest")
            elif matching[0].get("arm") != f'{key[0]}@"{key[1]}"':
                problems.append(f"{key[0]}@\"{key[1]}\": flow_rule {rule} points at "
                                f"{matching[0].get('arm')}")

    for rule in rules:
        match = FLOW_ARM_RE.match(rule["arm"])
        if not match:
            problems.append(f"flow rule {rule.get('id')}: bad arm {rule.get('arm')!r}")
            continue
        key = (match.group(1), match.group(2))
        if key not in source:
            problems.append(f"flow rule {rule.get('id')}: arm not in source: {rule['arm']}")

    print(f"check-arm-inventory: {len(source)} source arms, {len(register)} registered, "
          f"{len(rules)} flow rules")
    for problem in problems:
        print(f"check-arm-inventory: {problem}", file=sys.stderr)
    return 1 if problems else 0


if __name__ == "__main__":
    raise SystemExit(main())
