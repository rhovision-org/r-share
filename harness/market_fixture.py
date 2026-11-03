#!/usr/bin/env python3
"""A real provider/listing/unit/guest-ledger market for settlement and conservation.

Unlike `booking_fixture`, this fixture funds real r-cash accounts, creates real slots and
lets the ledger create a real Bond. It is the setup shared by the conservation, race and
generality suites.
"""

from __future__ import annotations

import json
import os
import time

import actors

DEFAULT_TERMS = {
    "assetId": "REV",
    "pricePerSlot": 100,
    "deposit": 50,
    "minSlots": 1,
    "maxSlots": 4,
    "instantBook": True,
    "contiguous": False,
    "holdBlocks": 1000,
    "depositReviewBlocks": 0,
    "disputeWindowBlocks": 0,
    "noShowBlocks": 0,
    "fullRefundBeforeBlocks": 0,
    "partialRefundBps": 0,
    "partialRefundBeforeBlocks": 0,
    "feeBps": 0,
    "maxLeadBlocks": 1_000_000,
    "maxSlotBlocks": 1_000,
    "maxUnits": 2,
    "eventTransferCutoffBlocks": 0,
}


TERM_ALIASES = {
    "asset_id": "assetId",
    "instant_book": "instantBook",
    "min_slots": "minSlots",
    "max_slots": "maxSlots",
    "max_units": "maxUnits",
    "hold_blocks": "holdBlocks",
    "deposit_review_blocks": "depositReviewBlocks",
    "dispute_window_blocks": "disputeWindowBlocks",
    "no_show_blocks": "noShowBlocks",
    "full_refund_before_blocks": "fullRefundBeforeBlocks",
    "partial_refund_bps": "partialRefundBps",
    "partial_refund_before_blocks": "partialRefundBeforeBlocks",
    "fee_bps": "feeBps",
    "max_lead_blocks": "maxLeadBlocks",
    "max_slot_blocks": "maxSlotBlocks",
    "event_transfer_cutoff_blocks": "eventTransferCutoffBlocks",
}


def terms(**overrides) -> dict:
    value = dict(DEFAULT_TERMS)
    for key, item in overrides.items():
        value[TERM_ALIASES.get(key, key)] = item
    return value


def _ok(label: str, reply):
    if not (isinstance(reply, tuple) and reply[0] is True):
        raise AssertionError(f"{label}: {reply!r}")
    return reply


class Market:
    """One provider, one listing, one unit and one guest ledger."""

    def __init__(self, name: str, *, kind: str = "robot",
                 provider: str = "business", guest: str = "sender",
                 platform: str = "deployer", price: int = 100,
                 deposit: int = 50, slot_lead: int = 15,
                 slot_ids: list[str] | None = None, **term_overrides):
        self.name = name
        self.token = f"{int(time.time() * 1000)}-{os.getpid()}"
        self.kind = kind
        self.provider = provider
        self.guest = guest
        self.platform = platform
        self.price = price
        self.deposit = deposit
        self.slot_lead = slot_lead
        self.slot_ids = slot_ids or ["s1"]
        self.term_overrides = term_overrides
        self.terms = terms(pricePerSlot=price, deposit=deposit, **term_overrides)
        self.provider_uri = ""
        self.listing_uri = ""
        self.unit_id = ""
        self.ledger_uri = ""
        self.start = 0
        self.end = 0
        self._ops = 0

    def op(self, label: str) -> str:
        self._ops += 1
        return f"{self.name}:{self.token}:{label}:{self._ops}"

    def setup(self) -> "Market":
        actors.ensure_vault(self.provider)
        actors.ensure_vault(self.guest)

        provider_book = actors.resolve("component:provider-book")
        registered = _ok("provider register", actors.invoke(
            provider_book, "register", '*dId, {"name": "fixture"}', key=self.provider))
        self.provider_uri = registered[1]
        provider_state = _ok("provider info", actors.invoke(
            self.provider_uri, "info"))[1]
        if provider_state.get("status") != "active":
            _ok("provider resume", actors.invoke(
                self.provider_uri, "resume", "*dId", key=self.provider))

        listing_factory = actors.resolve("component:listing-factory")
        listing_id = f"listing:{self.name}:{int(time.time() * 1000)}"
        created = _ok("listing create", actors.invoke(
            listing_factory, "create",
            f'*dId, "{self.provider_uri}", "{listing_id}", "{self.kind}", '
            f"{json.dumps(self.terms)}",
            key=self.provider))
        self.listing_uri = created[1]

        guest_book = actors.resolve("component:guest-book")
        ledger = _ok("guest ledger", actors.invoke(
            guest_book, "getOrCreate", "*dId", key=self.guest))
        self.ledger_uri = ledger[1]

        if actors.balance(self.guest) < self.price + self.deposit + 100:
            actors.fund(self.guest, 2_000)
        actors.ensure_account(self.platform)

        # Only now fix the slot window, so setup time cannot push the window into the past
        # before the booking is made.
        fresh = actors.block_height()
        self.start = fresh + self.slot_lead
        self.end = self.start + 100
        slots = [{"slotId": slot, "startHeight": self.start,
                  "endHeight": self.end} for slot in self.slot_ids]
        unit = _ok("unit create", actors.invoke(
            self.listing_uri, "ensureUnit",
            f'*dId, "unit:{listing_id}", {{"name": "{self.kind}"}}, '
            f"{json.dumps(slots)}, {self.price}",
            key=self.provider))
        self.unit_id = unit[1]

        _ok("listing activate", actors.invoke(
            self.listing_uri, "activate", "*dId", key=self.provider))
        return self

    def unit_slots(self, slot_ids: list[str] | None = None) -> list[dict]:
        return [{"unitId": self.unit_id, "slots": slot_ids or self.slot_ids,
                 "checkInHeight": self.start,
                 "checkOutHeight": self.end - 1}]

    def book_term(self, op_id: str, ledger_uri: str | None = None,
                  slot_ids: list[str] | None = None) -> str:
        return actors.call_term(
            ledger_uri or self.ledger_uri, "book",
            f'*dId, "{self.listing_uri}", {json.dumps(self.unit_slots(slot_ids))}, '
            f'"{op_id}", Nil')

    def book(self, op_id: str, slot_ids: list[str] | None = None):
        return _ok("book", actors.invoke(
            self.ledger_uri, "book",
            f'*dId, "{self.listing_uri}", {json.dumps(self.unit_slots(slot_ids))}, '
            f'"{op_id}", Nil',
            key=self.guest))

    def booking_info(self, uri: str) -> dict:
        return _ok("booking info", actors.invoke(uri, "info"))[1]

    def wait_in(self) -> None:
        actors.wait_height(self.start)

    # ---- booking arms ----------------------------------------------------------------

    def accept(self, uri: str, op_id: str):
        return _ok("accept", actors.invoke(
            uri, "accept", f'*dId, "{op_id}"', key=self.provider))

    def reject(self, uri: str, op_id: str, reason: str = "provider rejected"):
        return _ok("reject", actors.invoke(
            uri, "reject", f'*dId, "{op_id}", "{reason}"', key=self.provider))

    def cancel(self, uri: str, op_id: str):
        return _ok("cancel", actors.invoke(
            uri, "cancel", f'*dId, "{op_id}"', key=self.guest))

    def cancel_by_provider(self, uri: str, op_id: str):
        return _ok("cancelByProvider", actors.invoke(
            uri, "cancelByProvider", f'*dId, "{op_id}"', key=self.provider))

    def check_in(self, uri: str, op_id: str):
        return _ok("checkIn", actors.invoke(
            uri, "checkIn", f'*dId, "{op_id}"', key=self.provider))

    def check_out(self, uri: str, op_id: str):
        return _ok("checkOut", actors.invoke(
            uri, "checkOut", f'*dId, "{op_id}"', key=self.provider))

    def settle(self, uri: str, op_id: str):
        return _ok("settle", actors.invoke(uri, "settle", f'"{op_id}"'))

    def expire(self, uri: str, op_id: str):
        return _ok("expire", actors.invoke(uri, "expire", f'"{op_id}"'))

    def claim_no_show(self, uri: str, op_id: str):
        return _ok("claimNoShow", actors.invoke(
            uri, "claimNoShow", f'*dId, "{op_id}"', key=self.provider))

    def claim_deposit(self, uri: str, op_id: str, amount: int,
                      evidence_hash: str = "evidence"):
        return _ok("claimDeposit", actors.invoke(
            uri, "claimDeposit",
            f'*dId, "{op_id}", {amount}, "{evidence_hash}"', key=self.provider))

    def resolve_deposit(self, uri: str, op_id: str):
        return _ok("resolveDeposit", actors.invoke(
            uri, "resolveDeposit", f'"{op_id}"'))

    def dispute(self, uri: str, key: str, op_id: str, reason_hash: str = "hash"):
        return _ok("dispute", actors.invoke(
            uri, "dispute", f'*dId, "{op_id}", "{reason_hash}"', key=key))

    def slot_states(self, slot_ids: list[str] | None = None) -> dict:
        request = [{"unitId": self.unit_id, "slots": slot_ids or self.slot_ids}]
        reply = actors.invoke(self.listing_uri, "probe", json.dumps(request))
        if not (isinstance(reply, tuple) and reply[0] is True):
            raise AssertionError(f"probe: {reply!r}")
        return reply[1][self.unit_id]

    def slot_state(self) -> list:
        return self.slot_states(["s1"])["s1"]

    def ops(self) -> dict:
        reply = actors.invoke(self.ledger_uri, "ops", "*dId", key=self.guest)
        if not (isinstance(reply, tuple) and reply[0] is True):
            raise AssertionError(f"ledger ops: {reply!r}")
        return reply[1]


def ledger_for(guest: str) -> str:
    """Return a funded guest ledger for an actor that is not the market's guest."""
    actors.ensure_vault(guest)
    guest_book = actors.resolve("component:guest-book")
    reply = _ok("guest ledger", actors.invoke(
        guest_book, "getOrCreate", "*dId", key=guest))
    if actors.balance(guest) < 500:
        actors.fund(guest, 2_000)
    return reply[1]


def started(name: str, **kwargs) -> Market:
    return Market(name, **kwargs).setup()
