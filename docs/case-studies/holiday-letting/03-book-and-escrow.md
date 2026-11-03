# Book and hold escrow

> **Status:** Implemented, not yet exercised.

Ben books three nights. Everything below happens inside **one signed deploy**: the slots are
held, the money is withdrawn, the escrow purse is created, and the booking contract is
spawned. If any step fails, the deploy rolls back.

## 1. Create the guest ledger

```rholang
GuestBook!("getOrCreate", *deployerId, *ret)
```

Replies `(true, ledgerUri, guestId)`. `GuestBook` derives `guestId` from Ben's deploy key,
ensures his r-cash account, and spawns a `GuestLedger`. A second call returns the same
ledger.

## 2. Read before booking

```rholang
ListingContract!("info", *ret)          // status, terms, termsHash, units
UnitContract!("schedule", slotIds, *ret) // slotId -> {startHeight, endHeight}
UnitContract!("probe", slotIds, *ret)    // slotId -> ["free", gen] | ["held", ...] | ["booked", ...]
```

The off-chain product should show the guest the terms hash and the cancellation windows
before they sign. The booking call can enforce the hash.

## 3. Book

```rholang
GuestLedger!("book", *deployerId, listingUri,
             [{"unitUri": room1Uri,
               "slots": ["room-1:2026-11-05",
                         "room-1:2026-11-06",
                         "room-1:2026-11-07"]}],
             "ben-op-001", termsHash, *ret)
```

`unitSlots` may contain more than one unit (up to `maxUnits`), which is how a RaaS booking
can hold a machine and an operator together. For lodging it is one room.

## What the ledger does

1. **Reserves the opId.** A replay returns the stored booking; a pending replay returns
   `"pending"`.
2. **Checks the listing.** `active`, and `expectedTermsHash` matches (if supplied).
3. **Scans the units.** For each unit it reads the schedule and probes the requested slots.
4. **Validates the count and the window.** `minSlots <= slotCount <= maxSlots`,
   `unitSlots.length <= maxUnits`, every start is in the future and within `maxLeadBlocks`,
   every slot duration is within `maxSlotBlocks`.
5. **Holds the slots.** One fresh release token; on any failure every requested slot is
   released and the op is dropped.
6. **Escrows the money.**
   - resolve `component:payments`;
   - `issuer(assetId)` → brand + issuer;
   - `ensureAccount(deployerId)` → Ben's account;
   - `makeEmptyPurse`;
   - `account.withdraw(deployerId, assetId, rent + deposit)`;
   - measure with `getAmountOf`; a mismatch is `1/0`;
   - `purse.deposit(payment)`.
7. **Confirms if instant book.** Otherwise the slots stay held.
8. **Spawns the BookingContract** with the purse, release token, brand and data; inserts its
   URI.
9. **Records the result** against the opId and replies `(true, bookingId, bookingUri)`.

The guest never sees the escrow purse or the release token. They live in the booking
contract's closure.

## 4. Accept

For a request booking, Ana accepts before `holdUntilHeight`:

```rholang
BookingContract!("accept", *deployerId, "ana-op-001", *ret)
```

`accept` confirms every held slot with the booking's release token. If a slot was reclaimed,
it refuses `"slot lost"` and leaves the booking `requested`; Ben can `expire` or `cancel`.
If accepted, status becomes `confirmed`.

For `instantBook: true`, the ledger already confirmed the slots and the booking starts
`confirmed`.

## What can go wrong

| Refusal | Meaning | What to do |
|---|---|---|
| `"slot held"` | another guest holds or booked a slot | choose another date or retry later |
| `"schedule"` / `"lead"` / `"duration"` | the slot is in the past, too far ahead, or too long | fix the calendar mapping |
| `"terms"` | the terms hash changed since the guest saw them | show the new terms and re-book |
| `"listing inactive"` | paused or delisted | choose another listing |
| `"payments unavailable"` | r-cash cannot be resolved | retry when the rail is healthy |
| `"slots"` / `"units"` | wrong count for the terms | fix the selection |
| `"pending"` | the same opId is in flight | wait for the first booking |
| withdraw refusal | insufficient funds or no account | fund the account |

A withdrawal refusal releases the slots and refuses without moving value. A failure after
value moves is `1/0`, which rolls back the whole deploy.

> **Where to go deeper.** [Escrow and settlement](../../concepts/escrow-and-settlement.md)
> explains the money. [Time without clocks](../../concepts/time-without-clocks.md) explains
> the heights. [Arm reference](../../reference/arms.md) has the exact signatures.
