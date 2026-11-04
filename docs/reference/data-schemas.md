# Data schemas

> **Status:** Implemented. The contracts are the authority; this chapter is the field list.

All money is integer minor units. All maps are Rholang maps; keys are strings. A field shown
as `Nil` is optional or absent.

## Terms

| Field | Meaning |
|---|---|
| `assetId` | r-cash asset id; `"REV"` initially |
| `pricePerSlot` | rent per slot, minor units |
| `deposit` | refundable security deposit |
| `minSlots`, `maxSlots` | slot count bounds per booking |
| `maxUnits` | units per booking (a RaaS session may use two) |
| `instantBook` | confirm at booking instead of provider accept |
| `contiguous` | the schedule requires each slot to start where the previous ended |
| `holdBlocks` | hold lifetime from `now` |
| `depositReviewBlocks` | deposit review after checkout |
| `disputeWindowBlocks` | dispute window after a claim |
| `noShowBlocks` | grace after `checkInHeight` |
| `fullRefundBeforeBlocks` | full-refund cutoff before start |
| `partialRefundBps`, `partialRefundBeforeBlocks` | partial refund rate and cutoff |
| `feeBps` | platform fee, 0–10000 |
| `maxLeadBlocks` | furthest future slot |
| `maxSlotBlocks` | longest slot duration |
| `eventTransferCutoffBlocks` | event ticket transfer cutoff |

`termsHash` is BLAKE2b-256 over the ordered `rshare:terms:v1|<value>|...` string.

## Listing state

```
{"listingId", "providerUri", "providerId", "kind", "terms", "termsHash",
 "status": "draft" | "active" | "paused" | "delisted",
 "units": {unitId -> UnitRecord},
 "slots": {unitId -> {slotId -> SlotRecord}},
 "nonce", "admins", "threshold"}
```

## Unit record (inside ListingContract state)

```
{"unitId", "kind", "attrs",
 "schedule": {slotId -> {"startHeight", "endHeight"}},
 "pricePerSlot", "maxHoldBlocks", "status": "active" | "paused"}
```

The record carries no slot capability. Use
`ListingContract("unit", unitId)` for this projection.

`attrs` is free-form data. Do not put personal data or secrets in it.

## Slot record (inside `slots[unitId][slotId]`)

```
["free", generation]
["held", bookingId, untilHeight, releaseToken]
["booked", bookingId, releaseToken]
```

The per-listing allocator owns every slot in one cell. `acquire` writes all requested
slots in one commit; `confirm` and `release` are token-gated. `ListingContract("probe")`
returns the state with the release token redacted:
`["free", gen]`, `["held", bookingId, untilHeight]`, or `["booked", bookingId]`.

## Booking state

```
{"bookingId", "guestId", "providerId", "listingId", "listingUri", "kind",
 "unitSlots": [{"unitId", "slots": [slotId],
                 "checkInHeight", "checkOutHeight"}],
 "assetId", "rent", "deposit", "feeBps", "bondUri",
 "holdUntilHeight", "checkInHeight", "checkOutHeight",
 "cancellation": Terms,
 "admins", "threshold",
 "status": "requested" | "confirmed" | "checkedIn" | "checkedOut" |
           "redeemed" | "settled" | "rejected" | "cancelled" |
           "expired" | "noShow" | "disputed",
 "checkInAtHeight", "checkOutAtHeight", "depositReleaseHeight",
 "disputeAtHeight", "rejectReason",
 "ticket": {"holderId", "redeemed", "redeemedAtHeight", "transferCutoffHeight"},
 "claim": {"amount", "evidenceHash", "atHeight"},
 "ops": {opId -> result}, "nonce"}
```

`rent = Σ(unit.pricePerSlot * slots booked from that unit)`. Each unit stores its own
price; the deposit is separate. A single-price booking is the special case where every
unit carries the listing price.

## Bond state

```
{"paid": Bool, "expected": Int, "remaining": Int}
```

The Bond owns the ERTP purse and the settle token in its closure. One `pay` is terminal; a
second `pay` refuses `"closed"`.

## Ticket

The ticket is a projection of the booking, not a second cell:

```
{"holderId", "redeemed", "redeemedAtHeight", "transferCutoffHeight"}
```

## Claim

```
{"amount", "evidenceHash", "atHeight"}
```

`amount` is bounded by the deposit. `evidenceHash` is opaque.

## Settlement buckets

```
{"providerRent", "platformFee", "guestRefund", "providerDamage", "reason"}
```

The contract asserts
`providerRent + platformFee + guestRefund + providerDamage == rent + deposit`
before paying.

## Provider state

```
{"providerId", "profile", "status": "active" | "paused",
 "listings": {listingId -> {"uri", "kind", "termsHash", "status"}},
 "ops", "nonce", "admins", "threshold"}
```

## Guest ledger state

```
{"guestId", "ops": {opId -> {"pending", "ok", "bookingId", "uri"}}}
```

## Payments state

```
{"rcashRootUri", "minVersion", "platformOwnerId", "assets", "nonce"}
```

## Root entry

```
{key -> {"uri", "version", "kind", "status": "active" | "retired"}}
```

> **Where to go deeper.** [Arm reference](arms.md) is the signatures. [Data schemas in the
> source](https://github.com/rhovision-org/r-share/blob/main/contracts/src/deploy/22_booking_contract.rho)
> is the booking state. [Glossary](glossary.md) defines the terms.
