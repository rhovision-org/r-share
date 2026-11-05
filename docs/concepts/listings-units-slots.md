# Listings, units and slots

> **Status:** Implemented. **Applies to:** all resource classes.

The provider side is a provider contract and a per-listing allocator. The guest side is one
ledger and one booking contract. This chapter is the map; the [arm reference](../reference/arms.md) is the
canonical signature list.

## Provider flow

1. `ProviderBook.register(dId, profile)` creates the provider and ensures its r-cash account.
2. `ListingFactory.create(dId, providerUri, listingId, kind, terms)` validates the terms,
   hashes them, and spawns a `ListingContract` in `draft`.
3. `ListingContract.ensureUnit(dId, unitId, attrs, slots, pricePerSlot)` validates the
   schedule and inserts the unit and its free slot records into the listing's allocation
   cell.
4. `ListingContract.activate(dId)` makes the listing bookable.

```
ProviderBook --register--> ProviderContract
                              |
ListingFactory --create-----> ListingContract (draft)
                              |
                         ensureUnit
                              v
                  listingState {units, slots}
                              |
                          activate
                              v
                        ListingContract (active)
```

Figure: provisioning. Units and slots are data in one per-listing allocation cell. Bookings
on different listings never serialize; bookings on one listing do.

## The unit

A unit stores:

- `unitId`, `listingId`, `providerId`, `kind`;
- `attrs` — free-form metadata the off-chain product needs;
- `slots` — `slotId -> slot URI`, never a live capability;
- `schedule` — `slotId -> {startHeight, endHeight, startHintMs, endHintMs}`;
- `maxHoldBlocks`, `status`, `admins`, `threshold`.

The unit record is data inside `listingState`. `ListingContract("unit", unitId)` and
`ListingContract("schedule", unitId)` are the read arms. The stateless `UnitFactory` only
validates a schedule and scans a unit's slot map; it owns no cell.

## The slot

A slot record lives at `slots[unitId][slotId]` in `listingState`:

```
["free", generation]
["held", bookingId, untilHeight, releaseToken]
["booked", bookingId, releaseToken]
```

`acquire` accepts free or expired-held records, caps the hold at `now + maxHoldBlocks`, and
writes every requested slot in one take/await/put. `confirm` turns every requested held slot into
booked only for the same token. `release` frees every requested slot held by the token and
ignores reclaimed keys. `reset` is the provider's cleanup arm and needs the listing owner;
it clears only a free slot or an expired hold. `ensureUnit` rejects refund and transfer
cutoffs that do not fit the slot window (`"refund window"`), so a cutoff can no longer run
before a slot starts.

The token is a fresh unforgeable name created in the booking deploy. It is what prevents an
old booking from releasing a slot that a newer booking has since reclaimed.

## The listing

A listing stores:

- `listingId`, `providerId`, `kind`, `terms`, `termsHash`;
- `status` — `draft`, `active`, `paused`, `delisted`;
- `units` — `unitId -> unit record`;
- `slots` — `unitId -> {slotId -> slot record}`;
- `admins`, `threshold`, `nonce`.

`termsHash` is BLAKE2b-256 over the ordered `rshare:terms:v1|...` string. A guest can pass
the expected hash to `book`, so the booking is tied to the terms the guest saw.

## Guest flow

`GuestBook.getOrCreate(dId)` creates a `GuestLedger`. `GuestLedger.book(dId, listingUri,
unitSlots, opId, expectedTermsHash)`:

1. reserves the `opId`;
2. reads the listing and checks it is active and the terms hash matches;
3. calls `ListingContract.acquire` with the whole unit/slot set and a fresh release token;
   the allocator validates every unit, window and slot and writes all slots or none;
4. uses the acquire summary's rent and windows to withdraw rent + deposit into a
   per-booking escrow purse;
5. creates a per-booking `Bond` that owns the purse, then spawns a `BookingContract` with
   the Bond URI and a fresh settle token;
6. records the result against the `opId`.

`unitSlots` is a list of maps:

```
[{"unitId": "room-1", "slots": ["2026-11-05", "2026-11-06"],
  "checkInHeight": 120, "checkOutHeight": 180}]
```

## What the chain never stores

- live slot capabilities or release tokens in any public projection;
- calendar dates (only slot IDs and heights);
- KYC data, telemetry or insurance evidence;
- a mutable availability cache (it would serialize every booking).

> **Where to go deeper.** [Time without clocks](time-without-clocks.md) covers schedules.
> [Bookings and lifecycle](bookings-and-lifecycle.md) covers the guest contract.
> [Contract map](../reference/contract-map.md) lists every component and state cell.
