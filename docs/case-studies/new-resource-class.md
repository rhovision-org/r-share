# Adding a new resource class

> **Status:** How-to. There is no dedicated behavioural suite for a new class; add one.

The extension rule is deliberately strict: a new resource class may change **`kind`**,
**`attrs`**, the slot IDs, the terms, and a **pure policy branch**. It must not add a
booking arm. If it needs one, the primitive is wrong or the requirement is a new primitive
that deserves a design discussion.

## The recipe

Use a boat rental as the example.

### 1. Choose the kind

```text
kind = "boat"
```

`Terms.validate` currently accepts `lodging`, `event`, `bike`, `car` and `robot`. Add `boat`
to that list in `contracts/src/deploy/04_remedy.rho`. This is the only kind-specific code
in the shared path.

### 2. Define the unit and the slot

- **Unit:** the boat.
- **Slot:** a day or a half-day window.

```text
unit:  "boat-4"
attrs: {"marina": "harbour", "hullHash": "...", "licenceClass": "coastal"}
slot:  "boat-4:2026-11-05", startHeight 12000, endHeight 12288
```

Encode the marina and the hull hash in `attrs`; do not put the hull registration documents
on-chain.

### 3. Choose the terms

```text
assetId                    "REV"
pricePerSlot               300
deposit                    500
minSlots                   1
maxSlots                   3
maxUnits                   1
instantBook                false
contiguous                 true
holdBlocks                 20
depositReviewBlocks        96
disputeWindowBlocks        96
noShowBlocks               6
fullRefundBeforeBlocks     48
partialRefundBps           5000
partialRefundBeforeBlocks  12
feeBps                     500
maxLeadBlocks              10000
maxSlotBlocks              1440
eventTransferCutoffBlocks  0
```

Validate the fields with `Terms.validate`. If your cancellation arithmetic is the standard
one, no `Remedy.cancel` change is needed. If it differs (for example, a weather clause),
add a pure `boat` branch; do not touch the booking arms.

### 4. Add a fixture and a suite

- Add `harness/scenarios/boat_book.rho` and a fixture builder.
- Add `harness/unit_generality.py` coverage for book → accept → check-out → settle.
- Add allocator conformance cases: all-or-none multi-slot acquire, one held slot refuses the
  set, token-only confirm/release, and an expired hold reclaimed.
- Add the conservation assertions: the escrow is empty and the buckets sum to
  `rent + deposit`.

### 5. Add the docs

- A case-study chapter mapping the five primitives to the boat.
- Entries in [Contract map](../reference/contract-map.md) only if you added a component
  (you should not have).
- Entries in [Data schemas](../reference/data-schemas.md) for the new `attrs`.
- A glossary line for the new slot ID shape.

### 6. Change the reference coverage

`tools/check_docs.py` checks that every component and deploy file is in the contract map and
every install term and component key is in the install-order chapter. Adding a `kind` does
not add a component, so those checks stay green. Adding a contract does add one; update the
reference chapter in the same change.

## What not to do

- **Do not add a booking arm** for the new class. The lifecycle is shared.
- **Do not add a shared counter** for availability. Use one slot cell per slot.
- **Do not add a mutable availability cache.** It would serialize every booking.
- **Do not store a live capability in a data map.** Store a URI and resolve it.
- **Do not use `select`** for exclusivity. The node desugars it to parallel receives.
- **Do not trust the block timestamp.** Use heights.
- **Do not skip the status label.** A new class with no suite is "mapping only".

## The one-file change

If your class behaves like lodging with different labels, the only contract change is the
`kind` list in `Terms.validate`. Everything else is `attrs`, slots, terms, a fixture and the
off-chain product.

## When it is genuinely a new primitive

- Subscriptions or recurring access, not a finite interval.
- Usage metering where the price is only known after the session.
- Revenue splitting among several providers.
- Transferable ownership of the unit itself.
- A different trust model (custodial, insured, regulated).

Those are design discussions, not a `kind` addition. Fork if you need them, and tell the
community what you learned.

> **Where to go deeper.** [The use-case ladder](../orientation/use-case-ladder.md) places the
> class. [Extend a policy](../guides/extend-a-policy.md) is the arithmetic branch. [Test your
> fork](../guides/test-your-fork.md) is the gate.
