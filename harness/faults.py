#!/usr/bin/env python3
"""Test-only component fault injection through the deployed Root.

The helper rotates a component key to a stub URI and restores the original in the caller's
``finally``. It never edits contract source and never touches a production manifest.
"""

from __future__ import annotations

import actors
from assertions import one, root_uri
from unit_pause import admin_sigs

import deploy as node


def deploy_stub(term: str) -> str:
    uri = one(term)
    if not isinstance(uri, str) or not uri.startswith("rho:id:"):
        raise AssertionError(f"stub URI: {uri!r}")
    return uri


def fake_guest_ledger_factory() -> str:
    return deploy_stub("""
new GuestLedgerFactory, rs(`rho:registry:insertArbitrary`), ch,
    deployId(`rho:rchain:deployId`) in {
  contract GuestLedgerFactory(@"create", dId, @guestId, ret) = {
    ret!((false, "unavailable"))
  } |
  rs!(bundle+{*GuestLedgerFactory}, *ch) |
  for (@uri <- ch) { deployId!(uri) }
}
""")


def fake_bond_factory() -> str:
    return deploy_stub("""
new BondFactory, rs(`rho:registry:insertArbitrary`), ch,
    deployId(`rho:rchain:deployId`) in {
  contract BondFactory(@"createFromAccount", dId, @assetId, @expected,
                       settleToken, ret) = {
    ret!((false, "unavailable"))
  } |
  rs!(bundle+{*BondFactory}, *ch) |
  for (@uri <- ch) { deployId!(uri) }
}
""")


def rotate_component(key: str, new_uri: str, new_version: int = 1):
    root = root_uri()
    current = actors.invoke(root, "resolve", f'"{key}", 1')
    if not (isinstance(current, tuple) and current[0] is True):
        raise AssertionError(f"resolve {key}: {current!r}")
    original_uri, original_version = current[1], current[2]
    nonce = actors.root_nonce_read() + 1
    args = f"{key},{original_version},{new_uri},{new_version}"
    sigs = admin_sigs(node.load_keys(), "root", nonce, "rotate", args)
    reply = actors.invoke(
        root, "rotate",
        f'"{key}", {original_version}, "{new_uri}", {new_version}, {nonce}, {sigs}')
    if not (isinstance(reply, tuple) and reply[0] is True):
        raise AssertionError(f"rotate {key}: {reply!r}")
    return original_uri, original_version


def restore_component(key: str, original_uri: str, original_version: int) -> None:
    root = root_uri()
    nonce = actors.root_nonce_read() + 1
    args = f"{key},1,{original_uri},{original_version}"
    sigs = admin_sigs(node.load_keys(), "root", nonce, "rotate", args)
    reply = actors.invoke(
        root, "rotate",
        f'"{key}", 1, "{original_uri}", {original_version}, {nonce}, {sigs}')
    if not (isinstance(reply, tuple) and reply[0] is True):
        raise AssertionError(f"restore {key}: {reply!r}")
