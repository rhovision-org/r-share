# Create a listing and book it

> **Status:** Implemented, not yet exercised on a healthy devnet.

This is the shortest path from a provider to a confirmed booking. It uses the same arms as
the [holiday-letting walkthrough](../case-studies/holiday-letting/index.md), without the
narrative.

## The sequence

| Step | Call | Who | Reply |
|---|---|---|---|
| 1 | `ProviderBook("register", dId, profile)` | provider | `(true, providerUri, providerId)` |
| 2 | `ListingFactory("create", dId, providerUri, listingId, kind, terms)` | provider | `(true, listingUri)` |
| 3 | `ListingContract("ensureUnit", dId, unitId, attrs, slots)` | provider | `(true, unitUri)` |
| 4 | `ListingContract("activate", dId)` | provider | `(true, Nil)` |
| 5 | `GuestBook("getOrCreate", dId)` | guest | `(true, ledgerUri, guestId)` |
| 6 | `GuestLedger("book", dId, listingUri, unitSlots, opId, termsHash)` | guest | `(true, bookingId, bookingUri)` |
| 7 | `BookingContract("accept", dId, opId)` | provider | `(true, Nil)` |
| 8 | `BookingContract("checkIn", dId, opId)` | provider | `(true, Nil)` |
| 9 | `BookingContract("checkOut", dId, opId)` | provider | `(true, Nil)` |
| 10 | `BookingContract("settle", opId)` | anyone | `(true, Nil)` |

Steps 7 is skipped when `instantBook: true`; the ledger confirms the slots and the booking
starts `confirmed`.

## A minimal terms map

```text
{"assetId": "REV",
 "pricePerSlot": 100,
 "deposit": 50,
 "minSlots": 1,
 "maxSlots": 14,
 "maxUnits": 1,
 "instantBook": false,
 "contiguous": true,
 "holdBlocks": 20,
 "depositReviewBlocks": 48,
 "disputeWindowBlocks": 48,
 "noShowBlocks": 6,
 "fullRefundBeforeBlocks": 48,
 "partialRefundBps": 5000,
 "partialRefundBeforeBlocks": 12,
 "feeBps": 500,
 "maxLeadBlocks": 10000,
 "maxSlotBlocks": 1440,
 "eventTransferCutoffBlocks": 0}
```

`Terms.validate` validates it. Keep the numbers in minor units.

## A minimal unit

```text
unitId: "unit-1"
attrs:  {"name": "Unit 1"}
slots:  [{"slotId": "unit-1:2026-11-05",
          "startHeight": 12000, "endHeight": 12288}]
```

For lodging, `contiguous: true` requires the next slot to start where the previous ended. For
events, set `contiguous: false` and use one slot per show.

## The guest's selection

```text
unitSlots = [{"unitUri": "<unit URI>", "slots": ["unit-1:2026-11-05"]}]
```

The guest can select multiple units (up to `maxUnits`) and multiple slots (up to `maxSlots`).
For a RaaS session, select the machine unit and the operator unit together.

## What to verify in your client

- The listing is `active`.
- The unit's `schedule` returns the heights you expect.
- The unit's `probe` returns `["free", gen]` for every requested slot.
- The terms hash you display matches the listing.
- The guest's r-cash account has enough for `rent + deposit`.
- The provider's account exists (registration ensures it).

## Failure shortlist

`"slot held"`, `"terms"`, `"listing inactive"`, `"payments unavailable"`, `"slots"`,
`"units"`, `"not owner"`, `"not guest"`, `"closed"`, `"replay"`. The
[arm reference](../reference/arms.md) groups the full vocabulary.

> **Where to go deeper.** [Holiday letting](../case-studies/holiday-letting/index.md) is the
> full path. [Write a client](write-a-client.md) covers signing and reads. [Arm
> reference](../reference/arms.md) has every signature.
