#!/usr/bin/env python3
"""Render and deploy the r-share terms in order.

The r-cash Root URI and platform owner are read from configs/rcash.json and the r-cash
devnet manifest. `--dry-run` renders only, so static checks can parse the rendered terms
without a node.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "harness"))

import deploy as node  # noqa: E402
import preflight  # noqa: E402

ORDER = [
    "00_root.rho",
    "01_kernel.rho",
    "02_admin.rho",
    "40_payments.rho",
    "03_terms.rho",
    "04_remedy.rho",
    "10_provider.rho",
    "13_unit.rho",
    "14_listing_contract.rho",
    "15_bond.rho",
    "22_booking_contract.rho",
    "23_guest_ledger.rho",
    "20_booking.rho",
]


def load_json(path: Path) -> dict:
    if not path.is_file():
        raise SystemExit(f"install: missing {path}")
    return json.loads(path.read_text())


def render(text: str, values: dict[str, str]) -> str:
    for key, value in values.items():
        text = text.replace(key, value)
    return text


def decode(value):
    if isinstance(value, list):
        return [decode(item) for item in value]
    if isinstance(value, dict):
        if "ExprString" in value:
            return value["ExprString"]["data"]
        if "ExprUri" in value:
            return value["ExprUri"]["data"]
        if "ExprInt" in value:
            return int(value["ExprInt"]["data"])
        if "ExprBool" in value:
            return bool(value["ExprBool"]["data"])
        if "ExprTuple" in value:
            return tuple(decode(item) for item in value["ExprTuple"]["data"])
        if "ExprList" in value:
            return [decode(item) for item in value["ExprList"]["data"]]
        if "ExprSet" in value:
            return [decode(item) for item in value["ExprSet"]["data"]]
        if "ExprByteArray" in value:
            return bytes.fromhex(value["ExprByteArray"]["data"])
        return {key: decode(item) for key, item in value.items()}
    return value


def find_uri(value) -> str | None:
    if isinstance(value, str):
        return value if value.startswith("rho:id:") else None
    if isinstance(value, (list, tuple)):
        for item in value:
            found = find_uri(item)
            if found:
                return found
    if isinstance(value, dict):
        for item in value.values():
            found = find_uri(item)
            if found:
                return found
    return None


def admin_literal(keys: dict, dev_single_key: bool) -> str:
    names = ["admin1", "admin2", "admin3"]
    picked = [keys[name]["public"] for name in names if name in keys]
    distinct = list(dict.fromkeys(picked))
    if len(distinct) < 3:
        if not dev_single_key:
            raise SystemExit(
                "install: need admin1, admin2 and admin3 with distinct keys; "
                "pass --dev-single-key only on a throwaway devnet"
            )
        distinct = [keys["deployer"]["public"]] * 3
    return "[" + ", ".join(f'"{key}".hexToBytes()' for key in distinct[:3]) + "]"


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--node", default=None)
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--dev-single-key", action="store_true",
                    help="allow a 2-of-3 set built from one devnet key (never for production)")
    args = ap.parse_args(argv)

    if args.node:
        node.NODE = args.node

    if not args.dry_run and not preflight.require_devnet("install"):
        return 2

    cfg = load_json(ROOT / "configs" / "devnet.json")
    rcash = load_json(ROOT / "configs" / "rcash.json")
    if args.dry_run and not node.keys_path().is_file():
        keys = {
            "deployer": {"address": "1111dryrun",
                         "public": "02" + "11" * 32,
                         "secret": "00" * 32},
            "admin1": {"public": "02" + "22" * 32},
            "admin2": {"public": "02" + "33" * 32},
            "admin3": {"public": "02" + "44" * 32},
        }
    else:
        keys = node.load_keys()
    if "deployer" not in keys:
        raise SystemExit("install: keys file has no deployer key")

    rcash_manifest = Path(cfg.get("rcash_manifest", rcash["root_manifest_path"]))
    if rcash_manifest.is_file():
        rcash_root = load_json(rcash_manifest)["root_uri"]
    elif args.dry_run:
        rcash_root = "rho:id:dryrun-rcash"
    else:
        raise SystemExit(f"install: missing r-cash manifest {rcash_manifest}; "
                         "run `make rcash-install` first")

    common = {
        "__ADMINS__": admin_literal(keys, args.dev_single_key),
        "__THRESHOLD__": "2",
        "__RCASH_ROOT_URI__": rcash_root,
        "__PLATFORM_OWNER__": keys["deployer"]["address"],
    }

    build = ROOT / "contracts" / "build"
    rendered_dir = ROOT / ".devnet" / "rendered"
    rendered_dir.mkdir(parents=True, exist_ok=True)
    if args.dry_run:
        for stale in rendered_dir.glob("*.rho"):
            if stale.name not in ORDER:
                stale.unlink()
    state: dict[str, str] = {}
    root_uri = "rho:id:dryrun" if args.dry_run else None
    secret = keys["deployer"]["secret"]

    for name in ORDER:
        source = build / name
        if not source.is_file():
            raise SystemExit(f"install: missing {source}; run `make build`")
        values = dict(common)
        values["__ROOT_URI__"] = root_uri or "rho:id:pending"
        term = render(source.read_text(), values)
        (rendered_dir / name).write_text(term)

        if args.dry_run:
            print(f"install: rendered {name}")
            continue

        deploy_id, result = node.deploy_and_wait(term, secret)
        decoded = decode(result)
        print(f"install: {name} -> {deploy_id}")
        if name == "00_root.rho":
            root_uri = find_uri(decoded)
            if not root_uri:
                raise SystemExit(f"install: could not find root URI in {decoded}")
            state["root_uri"] = root_uri
            print(f"install: root {root_uri}")
        else:
            state.setdefault("components", {})[name] = decoded
            if name == "40_payments.rho":
                pay_uri = find_uri(decoded)
                if not pay_uri:
                    raise SystemExit(f"install: no payments URI in {decoded}")
                ensure = (
                    "new rl(`rho:registry:lookup`), ch, "
                    "dId(`rho:rchain:deployerId`), ret, "
                    "deployId(`rho:rchain:deployId`) in {\n"
                    f"  rl!(`{pay_uri}`, *ch) |\n"
                    "  for (P <- ch) {\n"
                    "    P!(\"ensureAccount\", *dId, *ret) |\n"
                    "    for (@r <- ret) { deployId!(r) }\n"
                    "  }\n"
                    "}\n"
                )
                node.deploy_and_wait(ensure, secret)
                print("install: platform account ensured")

    if not args.dry_run:
        manifest = ROOT / cfg.get("manifest", "deployments/devnet.json")
        manifest.parent.mkdir(parents=True, exist_ok=True)
        manifest.write_text(json.dumps({"root_uri": root_uri, **state}, indent=2) + "\n")
        print(f"install: wrote {manifest}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
