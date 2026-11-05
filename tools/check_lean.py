#!/usr/bin/env python3
"""Build the Lean model, reject proof escapes, and audit the axioms.

The project is pinned by `spec/lean/lean-toolchain` (Lean 4.12) and has no
Mathlib dependency. The audit accepts only the three core axioms Lean may
introduce (`propext`, `Classical.choice`, `Quot.sound`).
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEAN = ROOT / "spec" / "lean"
AUDIT = "RShare/Audit.lean"

FORBIDDEN = re.compile(r"(?<!#)\b(sorry|admit|native_decide)\b|^\s*axiom\s+[A-Za-z_]")
ALLOWED = {"propext", "Classical.choice", "Quot.sound"}
THEOREM = re.compile(
    r"^[ \t]*(?:@\[[^\]]*\][ \t]*)*(?:theorem|lemma)[ \t]+([A-Za-z0-9_]+)",
    re.M,
)
PRINT = re.compile(r"#print axioms\s+([A-Za-z0-9_.]+)")


def source_scan() -> list[str]:
    problems: list[str] = []
    for path in sorted(LEAN.rglob("*.lean")):
        if ".lake" in path.parts:
            continue
        for number, line in enumerate(path.read_text().splitlines(), start=1):
            code = line.split("--", 1)[0]
            if FORBIDDEN.search(code):
                problems.append(f"{path.relative_to(ROOT)}:{number}: {line.strip()}")
    lakefile = LEAN / "lakefile.lean"
    if lakefile.is_file() and re.search(r"^\s*require\b", lakefile.read_text(), re.M):
        problems.append("lakefile.lean: external package requirements are not allowed")
    return problems


def run(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=LEAN, text=True, capture_output=True)


def theorem_names() -> set[str]:
    names: set[str] = set()
    for path in sorted(LEAN.rglob("*.lean")):
        if ".lake" in path.parts or path.name in {"Audit.lean", "Emit.lean"}:
            continue
        names.update(THEOREM.findall(path.read_text()))
    return names


def audit_names() -> set[str]:
    return set(PRINT.findall((LEAN / AUDIT).read_text()))


def main() -> int:
    problems = source_scan()
    for problem in problems:
        print(f"forbidden proof escape: {problem}", file=sys.stderr)

    build = run(["lake", "build"])
    if build.returncode != 0:
        sys.stderr.write(build.stdout)
        sys.stderr.write(build.stderr)
        print("lean: build failed", file=sys.stderr)
        return 1

    audit = run(["lake", "env", "lean", AUDIT])
    if audit.returncode != 0:
        sys.stderr.write(audit.stdout)
        sys.stderr.write(audit.stderr)
        print("lean: audit failed", file=sys.stderr)
        return 1

    source = theorem_names()
    audited = audit_names()
    audited_short = {name.rsplit(".", 1)[-1] for name in audited}
    missing = sorted(source - audited_short)
    if missing:
        for name in missing:
            print(f"lean: theorem not named in {AUDIT}: {name}", file=sys.stderr)
        problems.append(f"{len(missing)} unaudited theorems")

    lines = [line for line in audit.stdout.splitlines()
             if "depends on axioms" in line or "does not depend on any axioms" in line]
    if len(lines) < len(audited):
        print(f"lean: audit produced {len(lines)} lines for {len(audited)} names",
              file=sys.stderr)
        problems.append("incomplete axiom audit")
    for line in lines:
        match = re.search(r"depends on axioms: \[(.*?)\]", line)
        if not match:
            continue
        used = {axiom.strip() for axiom in match.group(1).split(",") if axiom.strip()}
        extra = used - ALLOWED
        if extra:
            print(f"lean: unexpected axiom(s) {sorted(extra)} in {line}", file=sys.stderr)
            return 1
        if "sorryAx" in used:
            print(f"lean: sorryAx in {line}", file=sys.stderr)
            return 1

    if problems:
        return 1
    print(f"lean: built, {len(lines)} axiom audits, {len(source)} source theorems, "
          "no sorry/admit/axiom/native_decide")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
