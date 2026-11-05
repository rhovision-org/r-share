#!/usr/bin/env python3
"""Complete the r-cash core needed by r-share on an isolated devnet.

The r-cash v4 installer builds and deploys every product, including the AMM pool term.
That term exceeds the target node's parser depth on this devnet and aborts the install
after the core steps. r-share uses only the account factory, asset resolver, clock and
transfer rail, so this helper:

1. verifies the core installer steps are present in the r-cash state;
2. deploys the small `09_identity` term (account factory) from the r-cash checkout;
3. writes a minimal r-cash manifest for the r-share adapter.

It does not touch r-cash's source or its default devnet files.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "harness"))

CORE_STEPS = ("00_root", "01_kernel", "14_asset_resolver", "40_transfer")


def first_uri(value) -> str | None:
    if isinstance(value, str):
        return value if value.startswith("rho:id:") else None
    if isinstance(value, (list, tuple)):
        for item in value:
            found = first_uri(item)
            if found:
                return found
    if isinstance(value, dict):
        for item in value.values():
            found = first_uri(item)
            if found:
                return found
    return None


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--rcash-checkout", default="/home/patrick/Rholphin")
    ap.add_argument("--state", required=True)
    ap.add_argument("--manifest", required=True)
    args = ap.parse_args(argv)

    checkout = Path(args.rcash_checkout)
    state_path = Path(args.state)
    manifest_path = Path(args.manifest)
    state = json.loads(state_path.read_text())
    steps = state.get("steps", {})
    missing = [name for name in CORE_STEPS if name not in steps]
    if missing:
        print(f"rcash-core: missing installer steps {missing}", file=sys.stderr)
        return 2

    root = state["root_uri"]
    resolver = steps["14_asset_resolver"]["uris"][0]
    clock = steps["01_kernel"]["uris"][0]
    account = state.get("account_factory_uri")

    if account and manifest_path.is_file():
        manifest = json.loads(manifest_path.read_text())
        if manifest.get("root_uri") == root and manifest.get("account_factory_uri") == account:
            print(f"rcash-core: already complete ({account})")
            return 0

    term_path = checkout / "contracts" / "build" / "deploy" / "v3" / "09_identity.rho"
    term = term_path.read_text()
    term = term.replace("__ROOT_URI__", root).replace("__ASSET_RESOLVER_URI__", resolver)
    leftover = re.findall(r"__[A-Z0-9_]+__", term)
    if leftover:
        print(f"rcash-core: unresolved placeholders {sorted(set(leftover))}", file=sys.stderr)
        return 2

    import assertions

    reply = assertions.one(term)
    account = first_uri(reply)
    if not account:
        print(f"rcash-core: identity deploy returned no URI: {reply!r}", file=sys.stderr)
        return 1

    state["account_factory_uri"] = account
    state["clock_uri"] = clock
    state["asset_resolver_uri"] = resolver
    state_path.write_text(json.dumps(state, indent=2) + "\n")
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.write_text(json.dumps({
        "root_uri": root,
        "clock_uri": clock,
        "account_factory_uri": account,
        "asset_resolver_uri": resolver,
    }, indent=2) + "\n")
    print(f"rcash-core: account factory {account}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
