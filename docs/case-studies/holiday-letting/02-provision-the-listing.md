# Provision the listing

> **Status:** Implemented, not yet exercised.

Ana registers once, creates a listing, adds the two rooms and their nights, and activates
the listing. Every owner arm takes Ana's `deployerId`; no address is passed as an argument.

## 1. Register the provider

```rholang
ProviderBook!("register", *deployerId, {"name": "Casa do Mar"}, *ret)
```

Replies `(true, providerUri, providerId)`. `ProviderBook` derives `providerId` from Ana's
deploy key, ensures her r-cash account exists, and spawns a `ProviderContract`. Calling
`register` again returns the existing URI.

Refusals: `"not owner"`, `"account"`, `"profile"`.

## 2. Create the listing

```rholang
ListingFactory!("create", *deployerId, providerUri,
                "casa-do-mar", "lodging", terms, *ret)
```

The factory:

1. checks Ana owns the provider contract;
2. resolves `component:remedy`, `component:codec` and `component:admin`;
3. resolves `component:terms` and calls `Terms!("validate", kind, terms)` and
   `Terms!("hash", kind, terms)`;
4. calls `ListingContractFactory!("create", ...)` to spawn the listing in `draft`;
5. registers the listing URI on the provider contract.

Replies `(true, listingUri)`. Refusals: `"not provider"`, `"terms"`, `"codec"`,
`"listing factory unavailable"`.

## 3. Add the rooms

For each room:

```rholang
ListingContract!("ensureUnit", *deployerId, "room-1",
                 {"name": "Sea view", "beds": 2, "maxGuests": 2},
                 [{"slotId": "room-1:2026-11-05", "startHeight": 12000,
                   "endHeight": 12288},
                  {"slotId": "room-1:2026-11-06", "startHeight": 12288,
                   "endHeight": 12576},
                  {"slotId": "room-1:2026-11-07", "startHeight": 12576,
                   "endHeight": 12864}],
                 100,
                 *ret)
```

`ensureUnit`:

1. checks Ana owns the listing;
2. refuses if the listing is paused or delisted, or the unit already exists;
3. resolves `component:clock` and `component:unit-factory`;
4. calls `UnitFactory!("validate", kind, slots, maxLeadBlocks, maxSlotBlocks)`;
5. inserts the unit record and one `["free", 0]` record per slot into the listing's
   allocation cell;
6. stores `unitId -> unit record` and `slots[unitId]` on the listing.

Replies `(true, unitId)`. Refusals: `"not active"`, `"not owner"`, `"unit exists"`,
`"schedule"`, `"slots"`, `"slot id"`, `"slot gap"`, `"slot lead"`, `"slot duration"`.

Repeat for `room-2`.

## 4. Activate

```rholang
ListingContract!("activate", *deployerId, *ret)
```

Requires at least one unit. Status becomes `active`. Guests can now book.

## The shape after provisioning

```
ProviderContract (Ana)
  listings: { "casa-do-mar" -> listing URI }

ListingContract (active)
  units: { "room-1" -> unit record, "room-2" -> unit record }
  slots: { "room-1" -> { "room-1:2026-11-05" -> ["free", 0], ... } }
  schedule: { "room-1:2026-11-05" -> {12000, 12288}, ... }
```

The per-listing allocation cell owns every slot. One `acquire` writes all requested slots
or none; bookings on one listing serialize on that cell.

## Checks before you hand the listing to guests

- `ListingContract!("isActive")` returns `true`.
- `ListingContract!("units")` lists every room.
- `ListingContract!("schedule", unitId)` returns the heights you expect.
- `ListingContract!("probe", unitSlots)` returns `["free", 0]` for each slot.
- The terms hash you show off-chain matches `ListingContract!("info")`.

> **Where to go deeper.** [Listings, units and slots](../../concepts/listings-units-slots.md)
> is the model. [Arm reference](../../reference/arms.md) has the signatures. [Create a
> listing and book it](../../guides/create-a-listing-and-book-it.md) is the shorter guide.
