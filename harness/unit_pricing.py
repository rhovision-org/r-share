#!/usr/bin/env python3
"""Per-unit pricing suite: the unit stores its own price and refuses a bad one."""

from __future__ import annotations

from assertions import expect_ok, expect_refusal, one, root_uri
from preflight import require_devnet

import deploy as node


def create_unit(owner: str, price: int) -> str:
    return f"""
new rl(`rho:registry:lookup`), rootCh, rCh, ufCh, unitCh, infoCh, createCh,
    dId(`rho:rchain:deployerId`), deployId(`rho:rchain:deployId`) in {{
  rl!(`{root_uri()}`, *rootCh) |
  for (Root <- rootCh) {{
    Root!("resolve", "component:unit-factory", 1, *rCh) |
    for (@(true, uri, _) <- rCh) {{
      rl!(uri, *ufCh) |
      for (UnitFactory <- ufCh) {{
        UnitFactory!("create", *dId, "unit:{price}", "{owner}", "listing:test",
                     "robot", {{}},
                     [{{"slotId": "s1", "startHeight": 1, "endHeight": 2}}],
                     100, [], 1, {price}, *createCh) |
        for (@created <- createCh) {{
          match created {{
            (true, unitUri) => {{
              rl!(unitUri, *unitCh) |
              for (Unit <- unitCh) {{
                Unit!("info", *infoCh) |
                for (@i <- infoCh) {{ deployId!(i) }}
              }}
            }}
            _ => {{ deployId!(created) }}
          }}
        }}
      }}
    }}
  }}
}}
"""


def main() -> int:
    print("unit-pricing")
    if not require_devnet("unit-pricing"):
        return 2

    owner = node.load_keys()["deployer"]["address"]
    ten = one(create_unit(owner, 10))
    expect_ok("unit A", ten)
    if ten[1].get("pricePerSlot") != 10:
        raise AssertionError(f"unit A price: {ten!r}")

    twenty = one(create_unit(owner, 20))
    expect_ok("unit B", twenty)
    if twenty[1].get("pricePerSlot") != 20:
        raise AssertionError(f"unit B price: {twenty!r}")

    bad = one(create_unit(owner, 0))
    expect_refusal("unit price zero", bad, "price")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
