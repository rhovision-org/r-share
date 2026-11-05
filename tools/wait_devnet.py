#!/usr/bin/env python3
"""Wait until a fresh devnet is actually producing blocks and processing deploys.

`harness/preflight.py` only checks reachability and `acceptsDeploys`; a node at height 1
can satisfy both before autopropose has produced a block, and an install started then fails
with `NotProcessed Unknown`. This gate waits for real height progress and a signed PING.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "harness"))

PING = "new deployId(`rho:rchain:deployId`) in { deployId!(1) }"


def status(node: str) -> tuple[int, bool]:
    with urllib.request.urlopen(f"{node}/api/v1/status", timeout=10) as response:
        data = json.load(response)
    # This node omits acceptsDeploys when admission is open; only an explicit
    # false is a refusal.
    return int(data["latestBlockNumber"]), data.get("acceptsDeploys") is not False


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--node", required=True)
    ap.add_argument("--blocks", type=int, default=3)
    ap.add_argument("--timeout", type=float, default=300.0)
    args = ap.parse_args(argv)

    os.environ["RSHARE_NODE"] = args.node
    import assertions

    deadline = time.time() + args.timeout
    first = None
    while time.time() < deadline:
        try:
            height, accepts = status(args.node)
        except Exception:
            time.sleep(2.0)
            continue
        if first is None:
            first = height
        if accepts and height >= first + args.blocks:
            try:
                value = assertions.one(PING, phlo_limit=2_000_000)
            except Exception as error:
                print(f"wait-devnet: ping not processed yet: {error}", file=sys.stderr)
                time.sleep(3.0)
                continue
            if value == 1:
                print(f"wait-devnet: height {first} -> {height}; signed deploy finalized")
                return 0
        time.sleep(2.0)
    print("wait-devnet: chain did not become ready", file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
