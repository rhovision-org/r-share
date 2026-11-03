# Listings, units and slots

> **Status:** Implemented. **Applies to:** all resource classes.

The provider side is three contracts and one factory. The guest side is one ledger and one
booking contract. This chapter is the map; the [arm reference](../reference/arms.md) is the
canonical signature list.

## Provider flow

1. `ProviderBook.register(dId, profile)` creates the provider and ensures its r-cash account.
2. `ListingFactory.create(dId, providerUri, listingId, kind, terms)` validates the terms,
   hashes them, and spawns a `ListingContract` in `draft`.
3. `ListingContract.ensureUnit(dId, unitId, attrs, slots)` validates the schedule, spawns a
   `UnitContract`, and creates one `SlotContract` per slot.
4. `ListingContract.activate(dId)` makes the listing bookable.

```
ProviderBook --register--> ProviderContract
                              |
ListingFactory --create-----> ListingContract (draft)
                              |
                         ensureUnit
                              v
                     UnitContract --one per slot--> SlotContract
                              |
                          activate
                              v
                        ListingContract (active)
```

Figure: provisioning. Slots are separate contracts with private cells, so bookings on
different slots never serialize.

## The unit

A unit stores:

- `unitId`, `listingId`, `providerId`, `kind`;
- `attrs` — free-form metadata the off-chain product needs;
- `slots` — `slotId -> slot URI`, never a live capability;
- `schedule` — `slotId -> {startHeight, endHeight, startHintMs, endHintMs}`;
- `maxHoldBlocks`, `status`, `admins`, `threshold`.

The unit's hot arms (`hold`, `release`, `confirm`, `probe`) resolve the requested slot URIs
and call the SlotContracts. The unit's state cell is peeked on every hold for its status and
schedule; it is consumed only when slots are created, paused or reset. Multi-unit
`holdAll`/`releaseAll`/`confirmAll` orchestration lives in the stateless `UnitFactory`, so
the booking and ledger do not re-implement the loops.

## The slot

A slot owns one private cell:

```
["free", generation]
["held", bookingId, untilHeight, releaseToken]
["booked", bookingId, releaseToken]
```

`hold` accepts a free record or an expired held record, caps the hold at
`now + maxHoldBlocks`, and posts the held record. `confirm` turns a held record into a booked
record only for the same release token. `release` turns either into free only for the same
token. `reset` is the provider's emergency path and needs the unit's reset token.

The token is a fresh unforgeable name created in the booking deploy. It is what prevents an
old booking from releasing a slot that a newer booking has since reclaimed.

## The listing

A listing stores:

- `listingId`, `providerId`, `kind`, `terms`, `termsHash`;
- `status` — `draft`, `active`, `paused`, `delisted`;
- `units` — `unitId -> unit URI`;
- `admins`, `threshold`, `nonce`.

`termsHash` is BLAKE2b-256 over the ordered `rshare:terms:v1|...` string. A guest can pass
the expected hash to `book`, so the booking is tied to the terms the guest saw.

## Guest flow

`GuestBook.getOrCreate(dId)` creates a `GuestLedger`. `GuestLedger.book(dId, listingUri,
unitSlots, opId, expectedTermsHash)`:

1. reserves the `opId`;
2. reads the listing and checks it is active and the terms hash matches;
3. reads each unit's schedule and slot states;
4. holds the slots under one fresh release token;
5. withdraws rent + deposit from the guest's r-cash account into a per-booking escrow purse;
6. creates a per-booking `Bond` that owns the purse, then spawns a `BookingContract` with the
   Bond URI and a fresh settle token;
7. records the result against the `opId`.

`unitSlots` is a list of maps:

```
[{"unitUri": "rho:id:...", "slots": ["room-1:2026-11-05", "room-1:2026-11-06"]}]
```

## What the chain never stores

- live slot capabilities in data maps (only URIs);
- calendar dates (only slot IDs and heights);
- KYC data, telemetry or insurance evidence;
- a mutable availability cache (it would serialize every booking).

> **Where to go deeper.** [Time without clocks](time-without-clocks.md) covers schedules.
> [Bookings and lifecycle](bookings-and-lifecycle.md) covers the guest contract.
> [Contract map](../reference/contract-map.md) lists every component and state cell.
