#!/usr/bin/env python3
"""Per-scenario conservation gate for the booking base layer.

r-cash exposes no account enumeration, so the gate tracks a closed set: the scenario's
guest, provider and platform accounts plus the Bond it creates. Funding happens before the
first snapshot. Every terminal path must leave that sum constant, the Bond at zero, and
every slot either free or booked.

The settle token is private, so "paid once" is proved by replay, `Bond("info").paid` and a
zero Bond balance rather than a direct second `pay`.
"""

from __future__ import annotations

import argparse
import json

import actors
from assertions import expect_refusal
from market_fixture import Market, started
from preflight import require_devnet

import deploy as node


def snapshot(market: Market, bond_uri: str | None = None) -> dict:
    return {
        "guest": actors.balance(market.guest),
        "provider": actors.balance(market.provider),
        "platform": actors.balance(market.platform),
        "bond": actors.bond_balance(bond_uri) if bond_uri else 0,
    }


def total(values: dict) -> int:
    return sum(values.values())


def fee_for(rent: int, fee_bps: int) -> int:
    return rent * fee_bps // 10_000


def expect_delta(label: str, actual: int, expected: int) -> None:
    if actual != expected:
        raise AssertionError(f"{label}: expected {expected}, got {actual}")
    print(f"  ok {label} {expected:+d}")


def check_bond(bond_uri: str, rent: int, deposit: int) -> None:
    info = actors.bond_info(bond_uri)
    if info.get("expected") != rent + deposit:
        raise AssertionError(f"bond expected: {info!r}")
    if info.get("paid") is not True or info.get("remaining") != 0:
        raise AssertionError(f"bond not closed: {info!r}")
    print("  ok bond paid once and closed")


def check_slots(market: Market, expected: str) -> None:
    state = market.slot_state()
    if not (isinstance(state, list) and state and state[0] == expected):
        raise AssertionError(f"slot state: {state!r} (expected {expected})")
    print(f"  ok slot {expected}")


def check_no_pending(market: Market) -> None:
    for op_id, entry in market.ops().items():
        if isinstance(entry, dict) and entry.get("pending") is True:
            raise AssertionError(f"ledger op still pending: {op_id}")
    print("  ok no pending ledger op")


def open_market(name: str, **kwargs):
    market = started(name, **kwargs)
    before = snapshot(market)
    op_id = market.op("book")
    booked = market.book(op_id)
    booking_uri = booked[2]
    info = market.booking_info(booking_uri)
    bond_uri = info["bondUri"]
    rent, deposit = info["rent"], info["deposit"]
    escrow = actors.bond_balance(bond_uri)
    if escrow != rent + deposit:
        raise AssertionError(f"bond escrow: {escrow} != {rent + deposit}")
    return market, before, op_id, booking_uri, info, bond_uri


def finish(market: Market, before: dict, bond_uri: str, rent: int,
           deposit: int, expected_slots: str) -> dict:
    after = snapshot(market, bond_uri)
    if total(after) != total(before):
        raise AssertionError(f"conservation: {total(before)} -> {total(after)} "
                             f"({before!r} -> {after!r})")
    print(f"  ok conserved {total(before)}")
    check_bond(bond_uri, rent, deposit)
    check_slots(market, expected_slots)
    check_no_pending(market)
    return after


def settle_expectations(rent: int, deposit: int, fee_bps: int,
                        extra_deposit: int = 0) -> dict:
    fee = fee_for(rent, fee_bps)
    return {
        "guest": -(rent + extra_deposit),
        "provider": rent - fee + extra_deposit,
        "platform": fee,
    }


def expect_deltas(before: dict, after: dict, expected: dict) -> None:
    expect_delta("guest delta", after["guest"] - before["guest"], expected["guest"])
    expect_delta("provider delta", after["provider"] - before["provider"],
                 expected["provider"])
    expect_delta("platform delta", after["platform"] - before["platform"],
                 expected["platform"])


def scenario_cancel() -> None:
    print("  scenario cancel_full_refund")
    market, before, _, uri, info, bond = open_market(
        "cancel", instant_book=False, slot_lead=300)
    market.cancel(uri, market.op("cancel"))
    after = finish(market, before, bond, info["rent"], info["deposit"], "free")
    expect_deltas(before, after, {"guest": 0, "provider": 0, "platform": 0})


def debug_cancel() -> None:
    """Run one cancel with the policy split and balances printed. Not a loop."""
    print("  scenario cancel_full_refund (instrumented)")
    market, before, _, uri, info, bond = open_market(
        "cancel-debug", instant_book=False, slot_lead=300)
    now = actors.block_height()
    remedy = actors.resolve("component:remedy")
    split = actors.invoke(
        remedy, "cancel",
        f'"{info["kind"]}", {json.dumps(info["cancellation"])}, {now}, '
        f'{info["checkInHeight"]}, {info["rent"]}, {info["deposit"]}, "guest"')
    print(f"  booking rent={info['rent']} deposit={info['deposit']} "
          f"checkIn={info['checkInHeight']} now={now}")
    print(f"  remedy split: {split!r}")
    market.cancel(uri, market.op("cancel"))
    after = snapshot(market, bond)
    print(f"  before: {before!r}")
    print(f"  after:  {after!r}")
    print(f"  bond:   {actors.bond_info(bond)!r}")
    print(f"  slot:   {market.slot_state()!r}")


def scenario_provider_cancel() -> None:
    print("  scenario provider_cancel_full_refund")
    market, before, _, uri, info, bond = open_market(
        "provider-cancel", instant_book=False, slot_lead=300)
    market.cancel_by_provider(uri, market.op("cancelByProvider"))
    after = finish(market, before, bond, info["rent"], info["deposit"], "free")
    expect_deltas(before, after, {"guest": 0, "provider": 0, "platform": 0})


def scenario_reject() -> None:
    print("  scenario reject_full_refund")
    market, before, _, uri, info, bond = open_market(
        "reject", instant_book=False)
    market.reject(uri, market.op("reject"))
    after = finish(market, before, bond, info["rent"], info["deposit"], "free")
    expect_deltas(before, after, {"guest": 0, "provider": 0, "platform": 0})


def scenario_expire() -> None:
    print("  scenario expire_full_refund")
    market, before, _, uri, info, bond = open_market(
        "expire", instant_book=False, holdBlocks=1)
    actors.wait_height(info["holdUntilHeight"] + 1)
    market.expire(uri, market.op("expire"))
    after = finish(market, before, bond, info["rent"], info["deposit"], "free")
    expect_deltas(before, after, {"guest": 0, "provider": 0, "platform": 0})


def scenario_settle() -> None:
    print("  scenario settle_with_fee")
    market, before, _, uri, info, bond = open_market(
        "settle", instant_book=True, feeBps=100)
    market.wait_in()
    market.check_in(uri, market.op("checkIn"))
    market.check_out(uri, market.op("checkOut"))
    market.settle(uri, market.op("settle"))
    after = finish(market, before, bond, info["rent"], info["deposit"], "free")
    expect_deltas(before, after,
                  settle_expectations(info["rent"], info["deposit"], 100))


def scenario_no_show() -> None:
    print("  scenario no_show")
    market, before, _, uri, info, bond = open_market(
        "noshow", instant_book=True, feeBps=100, disputeWindowBlocks=10)
    market.wait_in()
    market.claim_no_show(uri, market.op("claimNoShow"))
    claim = market.booking_info(uri)
    actors.wait_height(claim["noShowClaimAtHeight"] + 11)
    actors.invoke(uri, "resolveNoShow", f'"{market.op("resolveNoShow")}"')
    after = finish(market, before, bond, info["rent"], info["deposit"], "free")
    expect_deltas(before, after,
                  settle_expectations(info["rent"], info["deposit"], 100))


def scenario_deposit_split() -> None:
    print("  scenario deposit_split")
    market, before, _, uri, info, bond = open_market(
        "split", instant_book=True, feeBps=100, depositReviewBlocks=100)
    market.wait_in()
    market.check_in(uri, market.op("checkIn"))
    market.check_out(uri, market.op("checkOut"))
    claim = 20
    market.claim_deposit(uri, market.op("claimDeposit"), claim)
    market.resolve_deposit(uri, market.op("resolveDeposit"))
    after = finish(market, before, bond, info["rent"], info["deposit"], "free")
    expect_deltas(before, after,
                  settle_expectations(info["rent"], info["deposit"], 100, claim))


def scenario_replay() -> None:
    print("  scenario replay")
    market, before, _, uri, info, bond = open_market(
        "replay", instant_book=False)
    cancel_op = market.op("cancel")
    market.cancel(uri, cancel_op)
    after = finish(market, before, bond, info["rent"], info["deposit"], "free")

    market.cancel(uri, cancel_op)
    again = snapshot(market, bond)
    if again != after:
        raise AssertionError(f"replay moved money: {after!r} -> {again!r}")
    print("  ok same op replayed without value movement")

    second_op = market.op("cancel2")
    refused = actors.invoke(
        uri, "cancel", f'*dId, "{second_op}"', key=market.guest)
    expect_refusal("new op after terminal", refused, "closed")
    final = snapshot(market, bond)
    if final != after:
        raise AssertionError(f"refusal moved money: {after!r} -> {final!r}")
    print("  ok refusal left balances unchanged")


def negative_control() -> int:
    """Omit the Bond from the tracked set; the sum must visibly drop."""
    market = started("negative", instant_book=True)
    before = snapshot(market)
    booked = market.book(market.op("book"))
    info = market.booking_info(booked[2])
    after_without_bond = snapshot(market)
    if total(after_without_bond) == total(before):
        raise AssertionError("negative control failed to detect untracked escrow")
    print(f"  ok negative control: untracked Bond detected "
          f"({total(before)} -> {total(after_without_bond)}, "
          f"escrow {info['rent'] + info['deposit']})")
    return 0


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--negative-control", action="store_true",
                    help="prove the sum assertion detects untracked escrow")
    ap.add_argument("--debug-cancel", action="store_true",
                    help="run one instrumented cancel and print the split")
    args = ap.parse_args(argv)

    if not require_devnet("conservation"):
        return 2
    print("conservation")
    if args.negative_control:
        negative_control()
        return 0
    if args.debug_cancel:
        debug_cancel()
        return 0

    scenario_cancel()
    scenario_provider_cancel()
    scenario_reject()
    scenario_expire()
    scenario_settle()
    scenario_no_show()
    scenario_deposit_split()
    scenario_replay()
    print("conservation: all scenarios conserved")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
