# Concurrency doctrine

> **Status:** Implemented. **Applies to:** every contract and every review.

The goal is to make the tuple space do the arbitration, not a lock or a global cell. This
chapter is near-verbatim the repository's concurrency doctrine; the code is the authority.

## Rules

1. **One allocation cell per listing.** `ListingContract` owns units and slots in one
   `listingState` cell. A slot is data in that cell, not a separate contract. No slot URI
   or live capability is published; `acquire`, `confirm` and `release` are the only writers.
2. **Per-listing serialization.** Bookings on the same listing serialize on the allocator
   cell. This is the accepted price of atomic multi-slot acquisition. Bookings on different
   listings never share a cell. The old per-slot sharding doctrine is retired.
3. **Acquisition is one commit.** `acquire` validates the whole request and writes every
   requested slot in one take/commit. There is no reachable state in which some requested
   slots are held and others are not. This is structural, not a consequence of sequential
   deploy execution.
4. **Peek is a hint, never a lock.** `GuestLedger` probes slots before spending phlo on a
   withdrawal, then `Unit.hold` revalidates under consume. A stale peek causes a clean
   refusal, never a double booking. Never carry a peeked record across an external await
   into a write.
5. **Per-booking contracts.** Every BookingContract has its own state cell, ticket
   projection and op map; every booking has its own Bond holding the escrow purse.
   Lifecycle operations on different bookings never share a cell.
6. **No global counters.** `bookingId = guestId ++ ":" ++ opId`; listing, unit and slot IDs
   are client- or provider-supplied and uniqueness-checked in the listing cell. Multi-slot
   acquisition is one allocator commit; `UnitFactory` only scans and validates.
7. **Read-mostly directories.** ShareRoot, ProviderBook and GuestBook are indexes. Clients
   holding a listing/unit/booking URI go directly to it; the authoritative contract
   re-checks ownership. Registry URIs are resolved per arm and never cached in a bundle.
8. **Lazy expiry, no timers.** A held record carries `untilHeight`. A later hold that sees
   `["held", oldId, until]` with `now >= until` may reclaim the slot. The old booking's
   `expire` is permissionless, refunds the guest, and ignores `"not holder"` on release
   because the refund is independent.
9. **`select` is forbidden.** In the target node, `rholang/src/normalizer.rs:256-274`
   desugars `PChoice` into parallel `for` receives. It is not committed choice. Exclusivity
   comes from consuming one datum. `make check-select` enforces this.

## Slot records

```
["free", generation]
["held", bookingId, untilHeight, releaseToken]
["booked", bookingId, releaseToken]
```

- `acquire` accepts free and expired-held records, bounds `until <= now + maxHoldBlocks`,
  checks listing and unit status, and writes all requested slots as one reservation.
- `confirm` turns every requested held slot into booked only for the same token; a wrong
  or expired key refuses the whole set.
- `release` frees every requested slot held or booked by the token and ignores reclaimed
  or foreign keys, so a reclaimed slot cannot strand the rest of the booking.
- `reset` is the provider's emergency path and requires the listing owner.

## Where serialization is accepted

| Point | Contention | Why acceptable |
|---|---|---|
| ShareRoot register/rotate | installer/admin | cold path |
| ProviderBook register | once per provider | cold path |
| ProviderContract / ListingContract | provisioning, pause, delist | off the booking path |
| Unit state cell | slot creation and provider reset | holds touch slot contracts |
| GuestLedger op reservation | per guest, short | prevents duplicate bookings |
| Listing allocation cell | every booking on one listing | atomic multi-slot acquisition; accepted hotspot |
| BookingContract state cell | per booking | exactly-once transitions |
| Bond state cell | per booking | one terminal payout; no cross-booking contention |
| r-cash account purse | per owner withdraw/payout | separate accounts per party |

Different listings never share a cell. Different slots of one listing do serialize; that is
the price of making partial multi-slot acquisition unrepresentable.

## How to review a change against the doctrine

- Does it add a second mutable cell on a listing allocation path? Reject or redesign.
- Does it use `select` for exclusivity? Reject.
- Does it peek, await, then write the peeked snapshot? Reject.
- Does it add a global counter? Reject; use a client-supplied id.
- Does it publish a slot capability or a release token? Reject; the allocator keeps both private.
- Does it add a timer? Reject; use a height and lazy expiry.

> **Where to go deeper.** [Time without clocks](time-without-clocks.md) covers lazy expiry.
> [Platform constraints](../reference/platform-constraints.md) lists the node limits. [Test
> your fork](../guides/test-your-fork.md) includes the race gate.
