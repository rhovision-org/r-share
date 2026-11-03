#!/usr/bin/env python3
"""Check the mdBook source: SUMMARY coverage, relative links, includes, README and
reference coverage. No network access.

Known limits: external links are not fetched, anchors are not validated, HTML links are not
parsed.
"""

from __future__ import annotations

import argparse
import ast
import json
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "docs"
SUMMARY = DOCS / "SUMMARY.md"
BOOK = ROOT / "book.toml"
README = ROOT / "README.md"
CATALOGUE = ROOT / "contracts" / "components.json"
INSTALL = ROOT / "harness" / "install.py"
RCASH = ROOT / "configs" / "rcash.json"
CONTRACT_MAP = DOCS / "reference" / "contract-map.md"
INSTALL_KEYS = DOCS / "reference" / "install-order-and-keys.md"

LINK = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")
REF = re.compile(r"^\s*\[[^\]]+\]:\s*(\S+)")
INCLUDE = re.compile(r"\{\{#include\s+([^}\s:]+)(?::[^}]*)?\}\}")
H1 = re.compile(r"^#\s+\S")
SKIP = ("http://", "https://", "mailto:", "tel:", "#")


def fail(errors: list[str], path: Path, line: int, message: str) -> None:
    rel = path.relative_to(ROOT) if path.is_relative_to(ROOT) else path
    errors.append(f"{rel}:{line}: {message}")


def strip_code(text: str) -> str:
    """Remove fenced code blocks and inline code spans, preserving line count."""
    out = []
    fenced = False
    for line in text.splitlines():
        if line.strip().startswith("```"):
            fenced = not fenced
            out.append("")
            continue
        out.append("" if fenced else re.sub(r"`[^`]*`", "", line))
    return "\n".join(out)


def strip_comments(text: str) -> str:
    return re.sub(r"<!--.*?-->", "", text, flags=re.S)


def links_in(path: Path) -> list[tuple[int, str]]:
    text = strip_comments(strip_code(path.read_text()))
    found = []
    for lineno, line in enumerate(text.splitlines(), 1):
        for match in LINK.finditer(line):
            found.append((lineno, match.group(1).strip()))
        ref = REF.match(line)
        if ref:
            found.append((lineno, ref.group(1).strip()))
    return found


def summary_entries(errors: list[str]) -> list[Path]:
    if not SUMMARY.is_file():
        errors.append("docs/SUMMARY.md: missing")
        return []
    entries: list[Path] = []
    seen = set()
    for lineno, target in links_in(SUMMARY):
        if target.startswith(SKIP):
            fail(errors, SUMMARY, lineno, f"SUMMARY target must be a local .md: {target}")
            continue
        clean = target.split("#", 1)[0]
        if not clean.endswith(".md") or clean.startswith("/") or ".." in Path(clean).parts:
            fail(errors, SUMMARY, lineno, f"SUMMARY target must be a docs/*.md: {target}")
            continue
        resolved = (DOCS / clean).resolve()
        if not resolved.is_file():
            fail(errors, SUMMARY, lineno, f"SUMMARY target missing: {clean}")
            continue
        if resolved in seen:
            fail(errors, SUMMARY, lineno, f"duplicate SUMMARY entry: {clean}")
        seen.add(resolved)
        entries.append(resolved)
    return entries


def check_config(errors: list[str]) -> None:
    if not BOOK.is_file():
        errors.append("book.toml: missing")
        return
    doc = tomllib.loads(BOOK.read_text())
    book = doc.get("book", {})
    build = doc.get("build", {})
    html = doc.get("output", {}).get("html", {})
    if book.get("src") != "docs":
        fail(errors, BOOK, 1, "book.src must be docs")
    if build.get("build-dir") != "docs/book":
        fail(errors, BOOK, 1, "build.build-dir must be docs/book")
    if not str(html.get("git-repository-url", "")).startswith("https://"):
        fail(errors, BOOK, 1, "output.html.git-repository-url must be https")
    if "{path}" not in str(html.get("edit-url-template", "")):
        fail(errors, BOOK, 1, "edit-url-template must contain {path}")
    if any(key.startswith("preprocessor") for key in doc):
        fail(errors, BOOK, 1, "preprocessors are not used")
    for key in ("additional-css", "additional-js"):
        if key in html:
            fail(errors, BOOK, 1, f"{key} is not used")
    ignore = ROOT / ".gitignore"
    text = ignore.read_text() if ignore.is_file() else ""
    if not any(line.strip() in ("/docs/book/", "docs/book/", "docs/book")
               for line in text.splitlines()):
        fail(errors, ignore, 1, "missing /docs/book/ ignore entry")


def check_orphans(errors: list[str], entries: list[Path]) -> None:
    listed = {path.resolve() for path in entries}
    for path in sorted(DOCS.rglob("*.md")):
        if path.resolve() == SUMMARY.resolve():
            continue
        if "book" in path.relative_to(DOCS).parts[:1]:
            continue
        if path.resolve() not in listed:
            fail(errors, path, 1, "chapter is not in SUMMARY.md")


def check_links(errors: list[str]) -> int:
    count = 0
    for path in sorted(DOCS.rglob("*.md")):
        if "book" in path.relative_to(DOCS).parts[:1]:
            continue
        for lineno, target in links_in(path):
            if target.startswith(SKIP):
                continue
            clean = target.split("#", 1)[0].split("?", 1)[0]
            if not clean:
                continue
            count += 1
            resolved = (path.parent / clean).resolve()
            if not resolved.exists():
                fail(errors, path, lineno, f"broken relative link: {target}")
            elif clean.endswith(".md") and not resolved.is_relative_to(DOCS.resolve()):
                fail(errors, path, lineno, f"chapter link escapes docs/: {target}")
    return count


def check_includes(errors: list[str]) -> int:
    count = 0
    for path in sorted(DOCS.rglob("*.md")):
        if "book" in path.relative_to(DOCS).parts[:1]:
            continue
        text = strip_code(path.read_text())
        for lineno, line in enumerate(text.splitlines(), 1):
            for match in INCLUDE.finditer(line):
                count += 1
                target = (path.parent / match.group(1)).resolve()
                if not target.is_file():
                    fail(errors, path, lineno, f"broken include: {match.group(1)}")
    return count


def check_readme(errors: list[str]) -> None:
    if not README.is_file():
        errors.append("README.md: missing")
        return
    for lineno, target in links_in(README):
        if target.startswith(SKIP):
            continue
        clean = target.split("#", 1)[0].split("?", 1)[0]
        if not clean:
            continue
        resolved = (ROOT / clean).resolve()
        if not resolved.exists():
            fail(errors, README, lineno, f"broken README link: {target}")
        if (DOCS / "book") in resolved.parents:
            fail(errors, README, lineno, f"README links to generated output: {target}")


def has_h1(path: Path, seen: set[Path] | None = None) -> bool:
    seen = seen or set()
    if path in seen or not path.is_file():
        return False
    seen.add(path)
    text = strip_comments(strip_code(path.read_text()))
    if any(H1.match(line) for line in text.splitlines()):
        return True
    for match in INCLUDE.finditer(text):
        if has_h1((path.parent / match.group(1)).resolve(), seen):
            return True
    return False


def body_size(path: Path) -> int:
    text = strip_comments(strip_code(path.read_text()))
    text = re.sub(r"^#+\s.*$", "", text, flags=re.M)
    text = re.sub(r"^\s*\|[-: |]+\|\s*$", "", text, flags=re.M)
    return len(re.sub(r"\s", "", text))


def check_nonempty(errors: list[str], entries: list[Path]) -> None:
    for path in entries:
        if not has_h1(path):
            fail(errors, path, 1, "missing H1")
        if body_size(path) < 200 and not INCLUDE.search(path.read_text()):
            fail(errors, path, 1, "chapter body is shorter than 200 non-space characters")


def check_coverage(errors: list[str]) -> tuple[int, int]:
    catalogue = json.loads(CATALOGUE.read_text())
    components = catalogue.get("components", {})
    contract_map = CONTRACT_MAP.read_text() if CONTRACT_MAP.is_file() else ""
    for name, entry in components.items():
        if name not in contract_map:
            fail(errors, CONTRACT_MAP, 1, f"missing component: {name}")
        if entry.get("file") and entry["file"] not in contract_map:
            fail(errors, CONTRACT_MAP, 1, f"missing deploy file: {entry['file']}")

    install_text = INSTALL.read_text()
    tree = ast.parse(install_text)
    order = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Assign) and any(
                isinstance(t, ast.Name) and t.id == "ORDER" for t in node.targets):
            order = ast.literal_eval(node.value)
    keys_text = INSTALL_KEYS.read_text() if INSTALL_KEYS.is_file() else ""
    for filename in order:
        if filename not in keys_text:
            fail(errors, INSTALL_KEYS, 1, f"missing install term: {filename}")

    root_keys = set()
    for path in (ROOT / "contracts" / "src" / "deploy").glob("*.rho"):
        for match in re.finditer(r'Root!\("register",\s*"([^"]+)"', path.read_text()):
            root_keys.add(match.group(1))
    for key in sorted(root_keys):
        if key not in keys_text:
            fail(errors, INSTALL_KEYS, 1, f"missing component key: {key}")

    rcash = json.loads(RCASH.read_text())
    for key in rcash.get("required_components", []):
        if key not in keys_text:
            fail(errors, INSTALL_KEYS, 1, f"missing r-cash key: {key}")
    return len(components), len(root_keys)


def check_hygiene(errors: list[str]) -> None:
    forbidden = ("/home/patrick/", "/Users/", "C:\\")
    paths = [README] + [p for p in DOCS.rglob("*.md")
                        if "book" not in p.relative_to(DOCS).parts[:1]]
    for path in paths:
        if not path.is_file():
            continue
        for lineno, line in enumerate(path.read_text().splitlines(), 1):
            for token in forbidden:
                if token in line:
                    fail(errors, path, lineno, f"absolute local path: {token}")
            if "http://" in line:
                print(f"{path.relative_to(ROOT)}:{lineno}: warning: plain http link",
                      file=sys.stderr)


def main(argv: list[str] | None = None) -> int:
    global ROOT, DOCS, SUMMARY, BOOK, README, CATALOGUE, INSTALL, RCASH
    global CONTRACT_MAP, INSTALL_KEYS
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default=str(ROOT))
    ap.add_argument("--skip-contracts", action="store_true")
    ap.add_argument("--quiet", action="store_true")
    args = ap.parse_args(argv)

    ROOT = Path(args.root).resolve()
    DOCS = ROOT / "docs"
    SUMMARY = DOCS / "SUMMARY.md"
    BOOK = ROOT / "book.toml"
    README = ROOT / "README.md"
    CATALOGUE = ROOT / "contracts" / "components.json"
    INSTALL = ROOT / "harness" / "install.py"
    RCASH = ROOT / "configs" / "rcash.json"
    CONTRACT_MAP = DOCS / "reference" / "contract-map.md"
    INSTALL_KEYS = DOCS / "reference" / "install-order-and-keys.md"

    errors: list[str] = []
    check_config(errors)
    entries = summary_entries(errors)
    check_orphans(errors, entries)
    link_count = check_links(errors)
    include_count = check_includes(errors)
    check_readme(errors)
    check_nonempty(errors, entries)
    component_count = key_count = 0
    if not args.skip_contracts:
        component_count, key_count = check_coverage(errors)
    check_hygiene(errors)

    if errors:
        for error in errors:
            print(f"check-docs: {error}", file=sys.stderr)
        print(f"check-docs: {len(errors)} violation(s)", file=sys.stderr)
        return 1
    if not args.quiet:
        print(f"check-docs: {len(entries)} chapters, {link_count} relative links, "
              f"{include_count} includes, {component_count} components, "
              f"{key_count} component keys")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
