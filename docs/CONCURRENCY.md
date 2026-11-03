# Concurrency doctrine

The goal is to make the tuple space do the arbitration, not a lock or a global cell.

## Rules

1. **One private single-datum cell per slot.** A SlotContract owns `slotState` and never
   publishes it. A unit stores slot **URIs**, not live capabilities. Every hold, confirm,
   release and reset consumes and reposts the cell.
2. **Per-slot sharding.** Two bookings on different nights of the same room, or different
   seats of the same event, touch different SlotContracts and different cells. They never
   serialize.
3. **Dynamic multi-slot acquisition is a same-deploy transaction.** Rholang joins have
   static arity, so a dynamic list cannot be consumed in one `for`. `Unit.hold` acquires the
   requested slots one by one; on any failure it releases every requested slot before
   replying. This is all-or-nothing to every other deploy because rholo executes deploys
   sequentially and a deploy is atomic with rollback. It is **not** one comm event; do not
   claim that in docs or tests.
4. **Peek is a hint, never a lock.** `GuestLedger` probes slots before spending phlo on a
   withdrawal, then `Unit.hold` revalidates under consume. A stale peek causes a clean
   refusal, never a double booking. Never carry a peeked record across an external await
   into a write.
5. **Per-booking contracts.** Every BookingContract has its own state cell, escrow purse,
   ticket projection and op map. Lifecycle operations on different bookings never share a
   cell.
6. **No global counters.** `bookingId = guestId ++ ":" ++ opId`; listing/unit/slot IDs are
   client/provider-supplied and uniqueness-checked in the owning cell.
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

- `hold` accepts free and expired-held records and bounds `until <= now + maxHoldBlocks`.
- `confirm` turns held into booked only for the same token.
- `release` turns held/booked into free only for the same token.
- `reset` is the provider's emergency path and requires the unit's reset token.

## Where serialization is accepted

| Point | Contention | Why acceptable |
|---|---|---|
| ShareRoot register/rotate | installer/admin | cold path |
| ProviderBook register | once per provider | cold path |
| ProviderContract / ListingContract | provisioning, pause, delist | off the booking path |
| Unit state cell | slot creation and provider reset | holds touch slot contracts |
| GuestLedger op reservation | per guest, short | prevents duplicate bookings |
| BookingContract state cell | per booking | exactly-once transitions |
| r-cash account purse | per owner withdraw/payout | guest and provider have separate accounts |

No hot path is shared between different guests booking different slots.
