# Model the stay

> **Status:** Implemented, not yet exercised.

Before any contract is deployed, decide four things: the unit, the slots, the terms, and the
policy arithmetic. Getting these right is most of the product design.

## The unit

A room is a unit. Ana has two, so the listing owns two units:

```
room-1  attrs: {"name": "Sea view", "beds": 2, "maxGuests": 2}
room-2  attrs: {"name": "Garden", "beds": 1, "maxGuests": 2}
```

The attrs are free-form data. The chain never interprets them; the off-chain product does.
Do not put personal data or secrets in `attrs`.

## The slots

A night is a slot. Use a stable, human-readable ID and map it to heights:

| Slot ID | startHeight | endHeight |
|---|---|---|
| `room-1:2026-11-05` | 12000 | 12288 |
| `room-1:2026-11-06` | 12288 | 12576 |
| `room-1:2026-11-07` | 12576 | 12864 |

The heights are illustrative. The provider maps them from the current chain height. Keep
`maxLeadBlocks` conservative; block times vary and a chain reset moves every height.

`contiguous: true` is for lodging: the schedule validator requires each night to start where
the previous one ended. A gap is a refused schedule, not a silently split stay.

## The terms

```text
assetId                    "REV"
pricePerSlot               100
deposit                    50
minSlots                   1
maxSlots                   14
maxUnits                   1
instantBook                false
contiguous                 true
holdBlocks                 20
depositReviewBlocks        48
disputeWindowBlocks        48
noShowBlocks               6
fullRefundBeforeBlocks     48
partialRefundBps           5000
partialRefundBeforeBlocks  12
feeBps                     500
maxLeadBlocks              10000
maxSlotBlocks              1440
eventTransferCutoffBlocks  0
```

The terms are validated by `Terms.validate` when the listing is created. The validator
bounds every field; an out-of-range or missing field is refused with `"terms"` or the field
name. The listing stores the terms and their hash.

## The policy arithmetic

Let `rent = pricePerSlot * slotCount`. For a three-night stay, rent is 300. With a deposit of
50, the escrow holds 350.

| Guest cancels | When | Guest refund | Provider rent | Platform fee |
|---|---|---|---|---|
| Before the full-refund cutoff | `now <= start - 48` | 350 | 0 | 0 |
| Before the partial cutoff | `now <= start - 12` | 150 + 50 = 200 (50% of rent + deposit) | 150 | 0 |
| Late | `now > start - 12` | 50 (deposit only) | 300 | 0 |
| Provider cancels or rejects | any time | 350 | 0 | 0 |

On a normal stay, settlement pays 285 to Ana, 15 to the platform, and 50 back to Ben. The
platform fee applies only to earned rent, never to a cancellation refund.

## The state the guest will see

The booking starts as `requested` (or `confirmed` for instant book), then moves through
`checkedIn`, `checkedOut` and `settled`. The full machine is in
[Bookings and lifecycle](../../concepts/bookings-and-lifecycle.md).

## Decide before you deploy

- Is the unit a room, a bed, or a whole house? The unit is what competes for the slot.
- Is the slot a night, a day, or an hour? The slot is the atomic conflict unit.
- Is the deposit refundable, and on what evidence?
- Who arbitrates a damage dispute, and what evidence hash will they see?
- What is the platform fee, if any?

> **Where to go deeper.** [Time without clocks](../../concepts/time-without-clocks.md)
> explains heights. [Data schemas](../../reference/data-schemas.md) is the field list.
> [Extend a policy](../../guides/extend-a-policy.md) is the arithmetic branch.
