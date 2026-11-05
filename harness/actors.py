#!/usr/bin/env python3
"""Actors, balances and slot probes for the conservation and race suites.

The devnet bakes in one funded deployer. Every other key needs a faucet drip before it can
sign a deploy; r-cash's keygen funds them during setup, and this module repairs a stale key
set by dripping on demand.
"""

from __future__ import annotations

import json
import time
from pathlib import Path

from assertions import one, root_call, root_uri

import deploy as node

PING = 'new deployId(`rho:rchain:deployId`) in { deployId!(1) }'

def _nonce_reply(reply, label: str) -> int:
    if (
        isinstance(reply, tuple)
        and len(reply) >= 2
        and reply[0] is True
        and isinstance(reply[1], int)
        and reply[1] >= 0
    ):
        return reply[1]
    raise AssertionError(f"{label} nonce: {reply!r}")


def admin_nonce_read() -> int:
    """The current global admin nonce, read from ShareAdmin("nonce")."""
    admin = resolve("component:admin")
    return _nonce_reply(invoke(admin, "nonce"), "admin")


def root_nonce_read() -> int:
    """The current governance nonce, read from ShareRoot("nonce")."""
    return _nonce_reply(one(root_call("nonce")), "root")


def keys() -> dict:
    return node.load_keys()


def public(name: str = "deployer") -> str:
    return keys()[name]["public"]


def address(name: str = "deployer") -> str:
    return keys()[name]["address"]


def call_term(uri: str, binding: str, args: str | None = None) -> str:
    head = json.dumps(binding)
    if args is not None:
        head += f", {args}"
    return f"""
new rl(`rho:registry:lookup`), ch, ret,
    dId(`rho:rchain:deployerId`), deployId(`rho:rchain:deployId`) in {{
  rl!(`{uri}`, *ch) |
  for (C <- ch) {{
    C!({head}, *ret) |
    for (@r <- ret) {{ deployId!(r) }}
  }}
}}
"""


def invoke(uri: str, binding: str, args: str | None = None, key: str = "deployer",
           phlo_limit: int = 2_000_000):
    return one(call_term(uri, binding, args), key=key, phlo_limit=phlo_limit)


def resolve(component: str, key: str = "deployer"):
    reply = invoke(root_uri(), "resolve", f'"{component}", 1', key=key)
    if not (isinstance(reply, tuple) and reply[0] is True):
        raise AssertionError(f"resolve {component}: {reply!r}")
    return reply[1]


def ensure_vault(name: str, min_phlo: int = 6_000_000) -> None:
    """Prove the key can precharge a race-sized deploy; drip the faucet if it cannot."""
    if name == "deployer":
        return
    try:
        one(PING, key=name, phlo_limit=min_phlo)
        return
    except node.DevnetError as error:
        if "insufficient funds" not in str(error):
            raise

    last: Exception | None = None
    for _ in range(6):
        try:
            node.faucet(address(name))
        except Exception as error:  # rate limited or already queued
            last = error
        time.sleep(2)
        try:
            one(PING, key=name, phlo_limit=min_phlo)
            return
        except node.DevnetError as error:
            if "insufficient funds" not in str(error):
                raise
            last = error
    raise AssertionError(f"actor {name} has no phlo: {last}")


def block_height() -> int:
    term = """
new bd(`rho:block:data`), ch, deployId(`rho:rchain:deployId`) in {
  bd!(*ch) |
  for (@n, @sender, @timestamp <- ch) { deployId!(n) }
}
"""
    height = one(term)
    if not isinstance(height, int):
        raise AssertionError(f"block height: {height!r}")
    return height


def wait_height(target: int, timeout: float = 180.0) -> int:
    deadline = time.time() + timeout
    while time.time() < deadline:
        current = node.height()
        if current >= target:
            return current
        one(PING, phlo_limit=1_000_000)
    raise AssertionError(f"height {target} not reached in {timeout}s")


def ensure_account(name: str) -> str:
    term = f"""
new rl(`rho:registry:lookup`), rootCh, afCh, ac, oCh,
    deployId(`rho:rchain:deployId`) in {{
  rl!(`{rcash_root_uri()}`, *rootCh) |
  for (Root <- rootCh) {{
    Root!("resolve", "component:account-factory", 1, *afCh) |
    for (@(true, afUri, _) <- afCh) {{
      rl!(afUri, *ac) |
      for (AccountFactory <- ac) {{
        AccountFactory!("getOrCreate", "{public(name)}".hexToBytes(), *oCh) |
        for (@(true, account, ownerId) <- oCh) {{ deployId!(ownerId) }}
      }}
    }}
  }}
}}
"""
    owner = one(term, key=name, phlo_limit=2_000_000)
    if not isinstance(owner, str):
        raise AssertionError(f"account for {name}: {owner!r}")
    return owner


def fund(name: str, amount: int = 2_000) -> int:
    """Mint REV into an actor's r-cash account and return the new balance."""
    if amount < 1 or amount > 100_000:
        raise ValueError("funding amount must be 1..100000")
    term = f"""
new rl(`rho:registry:lookup`), rootCh, afCh, ac, oCh, fCh, bCh,
    ertpCh, revIssuerCh, purseCh, fundCh, wdCh, depCh,
    deployId(`rho:rchain:deployId`), deployerId(`rho:rchain:deployerId`) in {{
  rl!(`{rcash_root_uri()}`, *rootCh) |
  rl!(`rho:rchain:ertp`, *ertpCh) |
  for (Root <- rootCh; @(_, ERTP) <- ertpCh) {{
    Root!("resolve", "component:account-factory", 1, *afCh) |
    for (@(true, afUri, _) <- afCh) {{
      rl!(afUri, *ac) |
      for (AccountFactory <- ac) {{
        AccountFactory!("getOrCreate", "{public(name)}".hexToBytes(), *oCh) |
        for (@(true, account, ownerId) <- oCh) {{
          @account!("depositFacet", "REV", *fCh) |
          for (@fr <- fCh) {{
            match fr {{
              (true, facet) => {{
                @ERTP!("getRevIssuer", *revIssuerCh) |
                for (@(revBrand, revIssuer) <- revIssuerCh) {{
                  @revIssuer!("makeEmptyPurse", *purseCh) |
                  for (@(true, purse) <- purseCh) {{
                    @purse!("revFund", *deployerId, 100000, *fundCh) |
                    for (@_ <- fundCh) {{
                      @purse!("withdraw", (revBrand, {amount}), *wdCh) |
                      for (@(true, payment) <- wdCh) {{
                        @facet!("deposit", payment, *depCh) |
                        for (@d <- depCh) {{
                          match d {{
                            (true, balance) => {{ deployId!([ownerId, balance]) }}
                            _ => {{ deployId!(d) }}
                          }}
                        }}
                      }}
                    }}
                  }}
                }}
              }}
              _ => {{ deployId!(fr) }}
            }}
          }}
        }}
      }}
    }}
  }}
}}
"""
    reply = one(term, key=name, phlo_limit=2_000_000)
    if not (isinstance(reply, list) and len(reply) == 2 and isinstance(reply[1], int)):
        raise AssertionError(f"fund {name}: {reply!r}")
    return reply[1]


def balance(name: str = "deployer") -> int:
    term = f"""
new rl(`rho:registry:lookup`), rootCh, afCh, ac, oCh, bCh,
    deployId(`rho:rchain:deployId`), deployerId(`rho:rchain:deployerId`) in {{
  rl!(`{rcash_root_uri()}`, *rootCh) |
  for (Root <- rootCh) {{
    Root!("resolve", "component:account-factory", 1, *afCh) |
    for (@(true, afUri, _) <- afCh) {{
      rl!(afUri, *ac) |
      for (AccountFactory <- ac) {{
        AccountFactory!("getOrCreate", "{public(name)}".hexToBytes(), *oCh) |
        for (@(true, account, ownerId) <- oCh) {{
          @account!("balance", *deployerId, "REV", *bCh) |
          for (@b <- bCh) {{ deployId!(b) }}
        }}
      }}
    }}
  }}
}}
"""
    reply = one(term, key=name, phlo_limit=2_000_000)
    if not (isinstance(reply, tuple) and reply[0] is True and isinstance(reply[1], int)):
        raise AssertionError(f"balance {name}: {reply!r}")
    return reply[1]


def bond_balance(uri: str) -> int:
    reply = invoke(uri, "balance")
    if not (isinstance(reply, tuple) and reply[0] is True and isinstance(reply[1], int)):
        raise AssertionError(f"bond balance: {reply!r}")
    return reply[1]


def bond_info(uri: str) -> dict:
    reply = invoke(uri, "info")
    if not (isinstance(reply, tuple) and reply[0] is True and isinstance(reply[1], dict)):
        raise AssertionError(f"bond info: {reply!r}")
    return reply[1]


def probe_slots(unit_uri: str, slot_ids: list[str], key: str = "deployer") -> dict:
    reply = invoke(unit_uri, "probe", json.dumps(slot_ids), key=key)
    if not (isinstance(reply, tuple) and reply[0] is True and isinstance(reply[1], dict)):
        raise AssertionError(f"probe {unit_uri}: {reply!r}")
    return reply[1]


def rcash_root_uri() -> str:
    from rcash_paths import configured_manifest

    cfg = json.loads((Path(__file__).resolve().parent.parent
                      / "configs" / "devnet.json").read_text())
    manifest = configured_manifest(cfg["rcash_manifest"])
    return json.loads(manifest.read_text())["root_uri"]
