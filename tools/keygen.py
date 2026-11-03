#!/usr/bin/env python3
"""Prepare the devnet key file for r-share.

The rholo devnet funds one baked-in deployer key. r-cash's devnet setup already writes a
keys file containing it, so by default this tool reuses that file rather than deriving keys.
If no source exists and `coincurve` is installed, it generates a fresh deployer record.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_SOURCE = ROOT.parent / "Rholphin" / ".devnet" / "keys.json"
DEPLOYER_KEY = "a68a6e6cca30f81bd24a719f3145d20e8424bd7b396309b0708a16c7d8000b76"
ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"


def b58encode(raw: bytes) -> str:
    n = int.from_bytes(raw, "big")
    out = ""
    while n:
        n, r = divmod(n, 58)
        out = ALPHABET[r] + out
    return "1" * (len(raw) - len(raw.lstrip(b"\0"))) + out


def keccak256(data: bytes) -> bytes:
    from Crypto.Hash import keccak

    h = keccak.new(digest_bits=256)
    h.update(data)
    return h.digest()


def record(name: str, secret_hex: str) -> dict:
    from coincurve import PrivateKey

    pub = PrivateKey(bytes.fromhex(secret_hex)).public_key.format(compressed=False)
    eth = keccak256(pub[1:])[-20:]
    payload = b"\x00\x00\x00\x00" + keccak256(eth)
    checksum = hashlib.blake2b(payload, digest_size=32).digest()[:4]
    return {
        "name": name,
        "secret": secret_hex,
        "public": pub.hex(),
        "address": b58encode(payload + checksum),
    }


def faucet(address: str, base: str) -> None:
    req = urllib.request.Request(
        f"{base}/api/v1/faucet",
        data=json.dumps({"address": address}).encode(),
        method="POST",
        headers={"Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            resp.read()
    except urllib.error.HTTPError as e:
        body = e.read().decode(errors="replace")
        if "already funded" not in body:
            raise RuntimeError(f"faucet {e.code}: {body}")


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=".devnet/keys.json")
    ap.add_argument("--from-path", default=str(DEFAULT_SOURCE),
                    help="reuse an existing devnet keys file (default: r-cash's)")
    ap.add_argument("--faucet", action="store_true")
    ap.add_argument("--http", default="http://localhost:40403")
    args = ap.parse_args(argv)

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    source = Path(args.from_path)
    if source.is_file():
        data = json.loads(source.read_text())
        if "deployer" not in data:
            print(f"keygen: {source} has no deployer key", file=sys.stderr)
            return 1
    else:
        try:
            data = {"deployer": record("deployer", DEPLOYER_KEY)}
        except Exception as error:  # coincurve or pycryptodome missing
            print(f"keygen: no source keys and cannot derive: {error}", file=sys.stderr)
            print("keygen: run `make keys` in ../Rholphin first", file=sys.stderr)
            return 1

    if args.faucet:
        for name, rec in data.items():
            if name == "deployer":
                continue
            try:
                faucet(rec["address"], args.http)
                print(f"keygen: funded {name}")
            except Exception as error:
                print(f"keygen: faucet {name}: {error}", file=sys.stderr)

    out.write_text(json.dumps(data, indent=2) + "\n")
    print(f"keygen: wrote {out} ({len(data)} keys)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
