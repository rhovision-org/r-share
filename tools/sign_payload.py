#!/usr/bin/env python3
"""Sign a canonical admin/arbiter payload for harness tests (devnet only).

Payload digest = blake2b256(utf8("rshare:admin:v1|domain|nonce|action|argsHash")).
Signature = DER-encoded secp256k1 over the 32-byte digest, hex encoded.
"""

from __future__ import annotations

import argparse
import hashlib


def digest(payload: str) -> bytes:
    return hashlib.blake2b(payload.encode("utf-8"), digest_size=32).digest()


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--secret", required=True, help="secp256k1 private key, hex")
    ap.add_argument("--payload", required=True)
    args = ap.parse_args(argv)

    from coincurve import PrivateKey

    key = PrivateKey(bytes.fromhex(args.secret))
    sig = key.sign(digest(args.payload), hasher=None)
    print(sig.hex())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
