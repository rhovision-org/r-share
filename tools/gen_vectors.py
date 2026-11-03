#!/usr/bin/env python3
"""Emit or check the Lean vectors consumed by the conformance suites."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEAN = ROOT / "spec" / "lean"
OUT = ROOT / "spec" / "vectors" / "allocator.json"


def emit() -> dict:
    proc = subprocess.run(
        ["lake", "env", "lean", "RShare/Emit.lean"],
        cwd=LEAN, text=True, capture_output=True,
    )
    if proc.returncode != 0:
        sys.stderr.write(proc.stdout)
        sys.stderr.write(proc.stderr)
        raise SystemExit("vectors: Lean emitter failed")
    vectors = {}
    for line in proc.stdout.splitlines():
        parts = line.strip().split("|")
        if len(parts) != 3:
            continue
        name, inputs, output = parts
        vectors[name] = {"input": inputs, "output": output}
    if not vectors:
        raise SystemExit("vectors: emitter produced no vectors")
    return {
        "generated_by": "spec/lean/RShare/Emit.lean",
        "vectors": dict(sorted(vectors.items())),
    }


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--write", action="store_true")
    args = ap.parse_args(argv)

    payload = emit()
    text = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    if args.write:
        OUT.parent.mkdir(parents=True, exist_ok=True)
        OUT.write_text(text)
        print(f"vectors: wrote {OUT.relative_to(ROOT)} ({len(payload['vectors'])} vectors)")
        return 0
    if not OUT.is_file() or OUT.read_text() != text:
        print(f"vectors: {OUT.relative_to(ROOT)} is stale; run --write", file=sys.stderr)
        return 1
    print(f"vectors: {len(payload['vectors'])} vectors current")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
