# Primitives

r-share models every rental as **capacity over an interval**. The same Listing → Unit →
Slot → Booking path serves lodging, events, bike rental, car rental and robotics rental.

## On-chain types

```
Provider   ownerId = REV address derived from the deploy key (r-cash AccountFactory)
Listing    { listingId, providerId, kind, terms, termsHash, status, units{} }
Unit       { unitId, listingId, providerId, kind, attrs, slots{slotId -> slotUri},
             schedule{slotId -> {startHeight,endHeight,startHintMs,endHintMs}},
             maxHoldBlocks, status, admins, threshold }
Terms      { assetId, pricePerSlot, deposit, minSlots, maxSlots, maxUnits,
             instantBook, contiguous, holdBlocks, depositReviewBlocks,
             disputeWindowBlocks, noShowBlocks, fullRefundBeforeBlocks,
             partialRefundBps, partialRefundBeforeBlocks, feeBps,
             maxLeadBlocks, maxSlotBlocks, eventTransferCutoffBlocks }
Booking    { bookingId, guestId, providerId, listingId, listingUri, kind,
             unitSlots[{unitUri, slots[]}], assetId, rent, deposit, feeBps,
             status, holdUntilHeight, checkInHeight, checkOutHeight,
             depositReleaseHeight, ticket, claim, ops, admins, threshold }
Settlement { providerRent, platformFee, guestRefund, providerDamage, reason }
```

Money is integer minor units. A booking's `rent` is `pricePerSlot * slotCount`; the deposit
is separate. `termsHash` is BLAKE2b-256 over the ordered `rshare:terms:v1|...` string.

## The five classes

| Primitive | Lodging | Events | Bike rental | Car rental | Robotics (RaaS) |
|---|---|---|---|---|---|
| Listing | property/room type | event | fleet/class | vehicle class | robot model/class |
| Unit | room | tier/section | bike | vehicle | machine |
| Slot | room-night | seat / show | station + hour window | day/time window | operating window |
| Booking | stay | ticket order | ride | rental | session/job |
| Ticket | check-in pass | seat ticket | unlock credential | key/fob | session credential |

A new class adds only `kind`, `attrs`, and a pure `Policy` branch. If a class needs a new
booking arm, the primitive is wrong. `tests` for M7 add bike, car and robot fixtures without
touching the booking, listing, provider or unit contracts.

## Slot IDs and calendar time

Slot IDs are opaque provider-chosen strings (1–64 bytes, unique per unit):

```
room-1:2026-11-05
GA-A:12
bike-17:2026-11-05T09:00
arm-3:2026-11-05T14:00
```

The chain never parses dates. The provider maps its calendar to `startHeight`/`endHeight`
when provisioning the unit. `startHintMs`/`endHintMs` are display metadata and are never
compared on-chain, because `rho:block:data`'s timestamp is the proposer's unverified wall
clock. All enforcement — cancellation cutoffs, no-show grace, hold expiry, check-in/out — is
in block heights. Height drift and chain resets are documented provider risks.

## Provider flow

1. `ProviderBook.register(dId, profile)` creates the provider and its r-cash account.
2. `ListingFactory.create(dId, providerUri, listingId, kind, terms)` validates the terms,
   hashes them, and spawns a ListingContract in `draft`.
3. `ListingContract.ensureUnit(dId, unitId, attrs, slots)` creates a UnitContract and one
   SlotContract per slot.
4. `ListingContract.activate(dId)` makes the listing bookable.

## Guest flow

`GuestBook.getOrCreate(dId)` creates a GuestLedger. `GuestLedger.book(dId, listingUri,
unitSlots, opId, expectedTermsHash)` reserves the opId, scans the units' schedules and slot
states, holds the slots, withdraws rent + deposit from the guest's r-cash account into a
per-booking escrow purse, and spawns a BookingContract.

`unitSlots` is a list of maps:

```
[{"unitUri": "rho:id:...", "slots": ["room-1:2026-11-05", "room-1:2026-11-06"]}]
```
