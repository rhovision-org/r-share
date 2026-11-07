#!/usr/bin/env python3
"""CoverPolicy: reserve, bind, pay, settle, expire and the claim token."""

from __future__ import annotations

import json
import time

import actors
import market_fixture
from assertions import expect, expect_ok, expect_refusal, one
from preflight import require_devnet


def cover_offer(provider_id: str, **overrides) -> dict:
    base = {
        "offerId": f"offer:policy:{time.time_ns()}",
        "subjectKind": "booking",
        "useCase": "lodging",
        "product": "asset_protection",
        "trigger": "damage",
        "insuredRole": "provider",
        "beneficiaryRole": "provider",
        "payerRole": "provider",
        "payer": "insured",
        "assetClass": "property",
        "assetId": "REV",
        "coverageCap": 200,
        "deductible": 10,
        "claimWindowBlocks": 1000,
        "evidenceSchema": "inspection/v1",
        "depositEffect": "reduce_to_deductible",
        "kassetUri": "",
        "engineeringBps": 4000,
        "insurerVarianceBps": 5000,
        "loadBps": 2000,
        "expectedLoss": 6,
        "premiumFloor": 6,
        "premiumTotal": 8,
        "gapBps": 1000,
        "pricingMode": "insurer-only",
        "publisherId": provider_id,
    }
    base.update(overrides)
    return base


def reserve_term(pool_uri: str, payer_name: str, payer_id: str,
                 amount: int) -> str:
    return f"""
new rl(`rho:registry:lookup`), rootCh, afCh, ac, oCh, wdCh, pCh, ret,
    deployerId(`rho:rchain:deployerId`), deployId(`rho:rchain:deployId`) in {{
  rl!(`{actors.rcash_root_uri()}`, *rootCh) |
  for (Root <- rootCh) {{
    Root!("resolve", "component:account-factory", 1, *afCh) |
    for (@(true, afUri, _) <- afCh) {{
      rl!(afUri, *ac) |
      for (AccountFactory <- ac) {{
        AccountFactory!("getOrCreate", "{actors.public(payer_name)}".hexToBytes(), *oCh) |
        for (@(true, account, ownerId) <- oCh) {{
          @account!("withdraw", *deployerId, "REV", {amount}, *wdCh) |
          for (@(true, payment) <- wdCh) {{
            rl!(`{pool_uri}`, *pCh) |
            for (Pool <- pCh) {{
              Pool!("reserve", *deployerId, "{payer_id}", payment, *ret) |
              for (@r <- ret) {{ deployId!(r) }}
            }}
          }}
        }}
      }}
    }}
  }}
}}
"""


def rho_value(value):
    if isinstance(value, (bytes, bytearray)):
        return f'"{value.hex()}".hexToBytes()'
    return json.dumps(value)


def rho_map(value: dict) -> str:
    return "{" + ", ".join(
        f"{json.dumps(key)}: {rho_value(item)}" for key, item in value.items()
    ) + "}"


def bind_term(pool_uri: str, subject: dict, offer_id: str, cap: int,
              policy_id: str, attach: bool = False) -> str:
    subject_id = subject["subjectId"]
    attach_block = "deployId!([r])" if not attach else f"""
              rl!(policyUri, *polCh) |
              for (Pol <- polCh) {{
                Pol!("attach", subjectId, subjectId, 1000000, *attachKey, *tokCh) |
                for (@a <- tokCh) {{
                  match a {{
                    (true, Nil) => {{ deployId!([(true, policyId, policyUri)]) }}
                    _ => {{ deployId!([a]) }}
                  }}
                }}
              }}
"""
    return f"""
new rl(`rho:registry:lookup`), rootCh, pCh, ret, bCh, polCh, tokCh, infoCh,
    deployerId(`rho:rchain:deployerId`), deployId(`rho:rchain:deployId`),
    attachKey, subjectId, policyUri in {{
  subjectId!({json.dumps(subject_id)}) |
  for (@subjectId <- subjectId) {{
    rl!(`{pool_uri}`, *pCh) |
    for (Pool <- pCh) {{
      Pool!("bindReserved", *deployerId, {rho_map(subject)},
            "{offer_id}", {cap}, "{policy_id}", *attachKey, *ret) |
      for (@r <- ret) {{
        match r {{
          (true, policyId, policyUri) => {{
            {attach_block}
          }}
          _ => {{ deployId!([r]) }}
        }}
      }}
    }}
  }}
}}
"""


def authorize_term(policy_uri: str, claim_id: str, loss_id: str,
                   amount: int, assessed: int, non_cover: int) -> str:
    return f"""
new rl(`rho:registry:lookup`), pCh, aCh, cCh, ret,
    deployerId(`rho:rchain:deployerId`), deployId(`rho:rchain:deployId`) in {{
  rl!(`{policy_uri}`, *pCh) |
  for (P <- pCh) {{
    P!("authorize", *deployerId, "{claim_id}", "{loss_id}", {amount},
      {assessed}, {non_cover}, "ev", *aCh) |
    for (@(true, Claim) <- aCh) {{
      @Claim!("pay", *cCh) |
      for (@r <- cCh) {{ deployId!([r]) }}
    }}
  }}
}}
"""


def info(pool_uri: str) -> dict:
    return actors.invoke(pool_uri, "info")[1]


def policy_info(policy_uri: str) -> dict:
    return actors.invoke(policy_uri, "info")[1]


def single(value):
    return value[0] if isinstance(value, list) and len(value) == 1 else value


def main() -> int:
    print("unit-cover-policy")
    if not require_devnet("unit-cover-policy"):
        return 2
    if not actors.require_profile("cover-demo"):
        return 2

    market = market_fixture.Market("coverpolicy").setup()
    provider = market.provider
    guest = market.guest
    provider_id = actors.address(provider)
    guest_id = actors.address(guest)
    actors.ensure_vault(provider)
    actors.ensure_account(provider)
    actors.fund(provider, 2000)
    actors.ensure_account(guest)

    listing = actors.invoke(market.listing_uri, "info")[1]
    offer = cover_offer(provider_id)
    pool = actors.resolve("component:cover-pool")
    book = actors.resolve("component:cover-book")

    published = actors.invoke(book, "publish", f'*dId, {json.dumps(offer)}',
                              key=provider)
    expect_ok("publish provider offer", published)

    reserve_before = info(pool).get("reserves").get(provider_id, 0)
    expect_ok("reserve provider premium", one(
        reserve_term(pool, provider, provider_id, 1000), key=provider, phlo_limit=4_000_000))
    expect("provider reserve", info(pool).get("reserves").get(provider_id),
           reserve_before + 1000)

    reserve_after = info(pool).get("reserves").get(provider_id)
    total_before = info(pool).get("reserveTotal")
    cap_before = info(pool).get("reserveCap", 1_000_000)
    provider_balance_before = actors.balance(provider)
    expect_ok("configure reserve cap", actors.invoke(
        pool, "configure", f"*dId, 1000000, 10000, {reserve_after + 1}"))
    expect_refusal("reserve above cap", one(
        reserve_term(pool, provider, provider_id, 2), key=provider,
        phlo_limit=4_000_000), "reserve cap")
    capped = info(pool)
    expect("reserve cap keeps reserve", capped.get("reserves").get(provider_id),
           reserve_after)
    expect("reserve cap keeps total", capped.get("reserveTotal"), total_before)
    expect("reserve cap refunds payer", actors.balance(provider),
           provider_balance_before)
    expect_ok("restore reserve cap", actors.invoke(
        pool, "configure", f"*dId, 1000000, 10000, {cap_before or 1000000}"))

    subject = {
        "subjectKind": "booking",
        "subjectId": f"{guest_id}:op:policy:{time.time_ns()}",
        "guestId": guest_id,
        "providerId": provider_id,
        "listingUri": market.listing_uri,
        "termsHash": listing.get("termsHash"),
        "insured": provider_id,
        "beneficiary": provider_id,
        "payer": "insured",
        "payerId": provider_id,
        "termsDeposit": market.deposit,
        "effectiveDeposit": offer["deductible"],
        "deductible": offer["deductible"],
    }

    policy_id = f"policy:{time.time_ns()}"
    pool_before = info(pool)
    # The bind needs a name attachKey; use a custom term instead of invoke.
    bind_reply = one(
        bind_term(pool, subject, offer["offerId"], 200, policy_id),
        key=guest, phlo_limit=4_000_000)
    bind_reply = single(bind_reply)
    expect_ok("bind reserved policy", bind_reply)
    policy_uri = bind_reply[2]

    entry = actors.invoke(pool, "policy", f'"{policy_id}"')[1]
    expect("policy status", entry.get("status"), "active")
    expect("policy premium", entry.get("premiumPaid"), offer["premiumTotal"])
    expect("pool outstanding", info(pool).get("outstanding"),
           pool_before.get("outstanding") + 200)
    expect("pool premium", info(pool).get("premiumIn"),
           pool_before.get("premiumIn") + offer["premiumTotal"])

    # Fixed beneficiary and operator payout.
    claim_id = f"claim:{time.time_ns()}"
    loss_id = f"loss:{time.time_ns()}"
    provider_before = actors.balance(provider)
    paid = actors.invoke(
        policy_uri, "pay",
        f'*dId, "{claim_id}", "{loss_id}", 50, 60, 10, "evidence:1"')
    expect_ok("operator pay", paid)
    expect("provider paid cover", actors.balance(provider), provider_before + 50)
    expect("policy paid", policy_info(policy_uri).get("paid"), 50)
    expect("pool claims after pay", info(pool).get("claimsPaid"),
           pool_before.get("claimsPaid") + 50)
    expect("pool outstanding after pay", info(pool).get("outstanding"),
           pool_before.get("outstanding") + 150)

    provider_after = actors.balance(provider)
    again = actors.invoke(
        policy_uri, "pay",
        f'*dId, "{claim_id}", "{loss_id}", 50, 60, 10, "evidence:1"')
    expect_ok("idempotent claim", again)
    expect("idempotent claim moves no value", actors.balance(provider),
           provider_after)

    expect_refusal("claim above remaining cap", actors.invoke(
        policy_uri, "pay",
        f'*dId, "claim:cap:{time.time_ns()}", "loss:cap:{time.time_ns()}", 200, 300, 0, "e"'), "amount")
    expect_refusal("same loss over recovery", actors.invoke(
        policy_uri, "pay",
        f'*dId, "claim:loss:{time.time_ns()}", "{loss_id}", 20, 60, 0, "e"'), "loss")

    expect_ok("operator settle", actors.invoke(policy_uri, "settle", "*dId"))
    expect("policy settled", policy_info(policy_uri).get("status"), "settled")
    expect("pool outstanding cleared", info(pool).get("outstanding"),
           pool_before.get("outstanding"))
    expect("pool claims conserved", info(pool).get("claimsPaid"),
           pool_before.get("claimsPaid") + 50)
    expect_ok("double settle", actors.invoke(policy_uri, "settle", "*dId"))

    # A second policy, expired and released without a claim.
    expire_offer = cover_offer(provider_id, claimWindowBlocks=2)
    expect_ok("publish expiring offer", actors.invoke(
        book, "publish", f'*dId, {json.dumps(expire_offer)}', key=provider))
    policy2 = f"policy:{time.time_ns()}"
    subject2 = dict(subject)
    subject2["subjectId"] = f"{guest_id}:op:policy:{time.time_ns()}"
    subject2["termsHash"] = listing.get("termsHash")
    bound2 = single(one(bind_term(
        pool, subject2, expire_offer["offerId"], 200, policy2), key=guest,
        phlo_limit=4_000_000))
    expect_ok("bind second policy", bound2)
    policy2_uri = bound2[2]
    actors.wait_height(actors.block_height() + 6)
    expect_ok("permissionless expire", actors.invoke(
        policy2_uri, "expire", key=guest))
    expect("expired status", policy_info(policy2_uri).get("status"), "expired")
    expect("expired returns cover", info(pool).get("outstanding"),
           pool_before.get("outstanding"))

    # Attach marks the policy attached; no release capability is returned.
    policy3 = f"policy:{time.time_ns()}"
    subject3 = dict(subject)
    subject3["subjectId"] = f"{guest_id}:op:policy:{time.time_ns()}"
    attached = one(
        bind_term(pool, subject3, offer["offerId"], 200, policy3, attach=True),
        key=guest, phlo_limit=4_000_000)
    attached = single(attached)
    expect_ok("attach policy", attached)
    expect("attached policy id", attached[1], policy3)
    entry3 = actors.invoke(pool, "policy", f'"{policy3}"')[1]
    expect("attached status", entry3.get("status"), "attached")

    # Operator authorize returns a fixed claim token; the token pays once.
    claim3 = f"claim:{time.time_ns()}"
    loss3 = f"loss:{time.time_ns()}"
    provider_before = actors.balance(provider)
    authorized = single(one(
        authorize_term(attached[2], claim3, loss3, 30, 40, 10),
        key="deployer", phlo_limit=4_000_000))
    expect_ok("authorized claim token pays", authorized)
    expect("claim token paid cover", actors.balance(provider),
           provider_before + 30)
    expect("policy claim recorded", policy_info(attached[2]).get("paid"), 30)

    # Refusals on bind.
    expect_refusal("unknown offer", single(one(bind_term(
        pool, dict(subject, subjectId=f"{guest_id}:op:bad:{time.time_ns()}"), "missing", 200,
        f"policy:{time.time_ns()}"), key=guest, phlo_limit=4_000_000)), "unknown offer")
    expect_refusal("cap above offer", single(one(bind_term(
        pool, dict(subject, subjectId=f"{guest_id}:op:cap:{time.time_ns()}"), offer["offerId"],
        201, f"policy:{time.time_ns()}"), key=guest, phlo_limit=4_000_000)), "cap")
    expect_refusal("wrong guest", single(one(bind_term(
        pool, dict(subject, subjectId=f"{guest_id}:op:wrong:{time.time_ns()}"), offer["offerId"],
        200, f"policy:{time.time_ns()}"), key=provider, phlo_limit=4_000_000)), "guest")

    print("unit-cover-policy: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
