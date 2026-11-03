#!/usr/bin/env python3
"""Validate the component catalogue: files, unique cells, known deps, acyclic graph."""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CATALOGUE = ROOT / "contracts" / "components.json"
DEPLOY = ROOT / "contracts" / "src" / "deploy"


def main() -> int:
    doc = json.loads(CATALOGUE.read_text())
    components = doc.get("components", {})
    errors = []
    cells = {}
    files = set()

    for name, entry in components.items():
        file = entry.get("file")
        if not file:
            errors.append(f"{name}: no file")
            continue
        files.add(file)
        if not (DEPLOY / file).is_file():
            errors.append(f"{name}: missing contracts/src/deploy/{file}")
        for cell in entry.get("state_cells", []):
            if cell in cells:
                errors.append(f"state cell {cell} owned by both {cells[cell]} and {name}")
            cells[cell] = name
        for dep in entry.get("depends_on", []):
            if dep not in components:
                errors.append(f"{name}: unknown dependency {dep}")

    for path in DEPLOY.glob("*.rho"):
        if path.name not in files:
            errors.append(f"{path.name}: no component declares this deploy term")

    # Acyclic: DFS over depends_on.
    state = {}

    def visit(node: str) -> None:
        if state.get(node) == 1:
            errors.append(f"dependency cycle at {node}")
            return
        if state.get(node) == 2:
            return
        state[node] = 1
        for dep in components.get(node, {}).get("depends_on", []):
            if dep in components:
                visit(dep)
        state[node] = 2

    for name in components:
        visit(name)

    if errors:
        for error in errors:
            print(f"check-arch: {error}", file=sys.stderr)
        return 1
    print(f"check-arch: {len(components)} component(s), {len(cells)} state cell(s), acyclic")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
