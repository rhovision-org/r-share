#!/usr/bin/env python3
"""Validate the audit registers and their cross-links.

The checker is the executable schema. It does not decide whether a finding is a real
vulnerability; it fails on missing required fields, duplicate or dangling IDs, a P0/P1
finding without evidence or a proposed change, and a proof claim without its declared
artifact.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
REGISTERS = ROOT / "audit" / "registers"
LEAN = ROOT / "spec" / "lean"
THEOREM = re.compile(
    r"^[ \t]*(?:@\[[^\]]*\][ \t]*)*(?:theorem|lemma)[ \t]+([A-Za-z0-9_]+)",
    re.M,
)


def load(name: str) -> dict:
    path = REGISTERS / name
    if not path.is_file():
        raise SystemExit(f"check-audit: missing {path}")
    return yaml.safe_load(path.read_text())


def ids(items: list[dict], label: str, problems: list[str]) -> set[str]:
    seen: set[str] = set()
    for item in items:
        item_id = item.get("id")
        if not item_id:
            problems.append(f"{label}: entry without id")
            continue
        if item_id in seen:
            problems.append(f"{label}: duplicate id {item_id}")
        seen.add(item_id)
    return seen


def refs(values, known: set[str], label: str, problems: list[str], *, optional=False) -> None:
    if values is None:
        if optional:
            return
        problems.append(f"{label}: missing references")
        return
    for value in values:
        if value not in known:
            problems.append(f"{label}: dangling reference {value}")


def main() -> int:
    findings_doc = load("findings.yaml")
    hazards_doc = load("hazards.yaml")
    theorems_doc = load("theorems.yaml")
    improvements_doc = load("improvements.yaml")
    tests_doc = load("tests.yaml")
    arms_doc = load("arms.yaml")

    findings = findings_doc.get("findings", [])
    hazards = hazards_doc.get("hazards", [])
    theorems = theorems_doc.get("theorems", [])
    improvements = improvements_doc.get("improvements", [])
    tests = tests_doc.get("tests", [])
    arms = arms_doc.get("arms", [])

    problems: list[str] = []
    finding_ids = ids(findings, "findings", problems)
    hazard_ids = ids(hazards, "hazards", problems)
    theorem_ids = ids(theorems, "theorems", problems)
    improvement_ids = ids(improvements, "improvements", problems)
    test_ids = ids(tests, "tests", problems)

    if len(arms) != 164:
        problems.append(f"arms: expected 164, found {len(arms)}")
    if len({(a.get('component'), a.get('arm')) for a in arms}) != len(arms):
        problems.append("arms: duplicate component/arm pair")

    for finding in findings:
        label = f"finding {finding.get('id')}"
        required = ("title", "severity", "class", "status", "confidence", "components",
                    "arms", "hazards", "invariants", "evidence_level", "evidence",
                    "description", "preconditions", "impact", "proposed_fix", "reviewer",
                    "verified_at")
        for field in required:
            if field not in finding:
                problems.append(f"{label}: missing {field}")
        if finding.get("severity") not in {"P0", "P1", "P2", "P3"}:
            problems.append(f"{label}: bad severity {finding.get('severity')!r}")
        if finding.get("status") not in {"confirmed", "suspected", "disputed", "fixed",
                                          "deferred", "accepted"}:
            problems.append(f"{label}: bad status {finding.get('status')!r}")
        if finding.get("evidence_level") not in {"E0", "E1", "E2", "E3", "E4"}:
            problems.append(f"{label}: bad evidence level {finding.get('evidence_level')!r}")
        refs(finding.get("hazards"), hazard_ids, label, problems)
        refs([finding.get("proposed_fix")], improvement_ids, label, problems)
        if finding.get("severity") in {"P0", "P1"}:
            if not finding.get("evidence"):
                problems.append(f"{label}: P0/P1 without evidence")
            if not finding.get("hazards"):
                problems.append(f"{label}: P0/P1 without hazard")
        if finding.get("proposed_fix") and finding["proposed_fix"] not in improvement_ids:
            problems.append(f"{label}: proposed_fix does not resolve")

    for hazard in hazards:
        label = f"hazard {hazard.get('id')}"
        for field in ("node", "guide_word", "parameter", "deviation", "causes",
                      "consequences", "severity", "likelihood", "risk", "controls",
                      "findings", "tests", "disposition", "owner", "evidence"):
            if field not in hazard:
                problems.append(f"{label}: missing {field}")
        if hazard.get("disposition") not in {"avoid", "mitigate", "transfer", "accept",
                                              "open"}:
            problems.append(f"{label}: bad disposition {hazard.get('disposition')!r}")
        refs(hazard.get("findings"), finding_ids, label, problems, optional=True)
        refs(hazard.get("tests"), test_ids, label, problems, optional=True)
        if hazard.get("risk") in {"high", "critical"} and not hazard.get("findings"):
            if not hazard.get("controls"):
                problems.append(f"{label}: high-risk without finding or control")

    source_theorems: set[str] = set()
    for path in LEAN.rglob("*.lean"):
        if ".lake" in path.parts or path.name in {"Audit.lean", "Emit.lean"}:
            continue
        source_theorems.update(THEOREM.findall(path.read_text()))
    registry_theorems = {t.get("name") for t in theorems}
    for name in sorted(source_theorems - registry_theorems):
        problems.append(f"theorems: source theorem not in register: {name}")
    for name in sorted(registry_theorems - source_theorems):
        problems.append(f"theorems: register name not in source: {name}")

    for theorem in theorems:
        label = f"theorem {theorem.get('id')}"
        for field in ("name", "module", "kind", "status", "statement", "axioms", "scope",
                      "evidence", "reviewer"):
            if field not in theorem:
                problems.append(f"{label}: missing {field}")
        if theorem.get("status") not in {"proved-model", "proved-model-bounded", "modeled",
                                          "assumption", "owed", "proved-tied", "open"}:
            problems.append(f"{label}: bad status {theorem.get('status')!r}")
        if theorem.get("status") == "proved-tied" and not theorem.get("corpus"):
            problems.append(f"{label}: proved-tied without corpus")
        if theorem.get("status") == "proved-model" and not theorem.get("audit_line", False):
            problems.append(f"{label}: proved-model without audit line")

    for improvement in improvements:
        label = f"improvement {improvement.get('id')}"
        for field in ("priority", "findings", "hazards", "change", "type", "breaking",
                      "migration", "compatibility_window", "tests", "rollout_stage",
                      "rollback", "status"):
            if field not in improvement:
                problems.append(f"{label}: missing {field}")
        refs(improvement.get("findings"), finding_ids, label, problems, optional=True)
        refs(improvement.get("hazards"), hazard_ids, label, problems, optional=True)
        refs(improvement.get("tests"), test_ids, label, problems, optional=True)
        if improvement.get("breaking") and not improvement.get("migration"):
            problems.append(f"{label}: breaking change without migration")

    for test in tests:
        label = f"test {test.get('id')}"
        for field in ("name", "file", "status", "focus"):
            if field not in test:
                problems.append(f"{label}: missing {field}")
        if test.get("status") not in {"planned", "implemented", "passing", "failing",
                                       "not_run"}:
            problems.append(f"{label}: bad status {test.get('status')!r}")

    print(f"check-audit: {len(findings)} findings, {len(hazards)} hazards, "
          f"{len(theorems)} theorems, {len(improvements)} improvements, {len(tests)} tests, "
          f"{len(arms)} arms")
    for problem in problems:
        print(f"check-audit: {problem}", file=sys.stderr)
    return 1 if problems else 0


if __name__ == "__main__":
    raise SystemExit(main())
