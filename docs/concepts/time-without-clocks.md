# Time without clocks

> **Status:** Implemented. **Applies to:** all deadlines and intervals.

Rholang has no trusted wall clock. `rho:block:data` returns `(blockNumber, senderBytes,
timestampMs)`, but the timestamp is the proposer's informational wall clock and is not a
consensus input. r-share never compares it. All enforcement is in **block heights**.

## Opaque slot IDs

A slot ID is a provider-chosen string, 1–64 bytes, unique within a unit:

```
room-1:2026-11-05
GA-A:12
bike-17:2026-11-05T09:00
car-7:2026-11-05
arm-3:2026-11-05T14:00
```

The chain does not parse the date. The provider supplies the slot's `startHeight` and
`endHeight` when provisioning the unit. The ID is the human-facing key; the heights are the
enforcement.

`startHintMs` and `endHintMs` exist only for display. They are never compared on-chain.

## What uses heights

- **Holds.** A hold has `untilHeight = now + holdBlocks`. After that, another booking may
  reclaim the slot.
- **Cancellation.** The policy compares `now` to `startHeight - fullRefundBeforeBlocks`,
  `startHeight - partialRefundBeforeBlocks`, and `startHeight`.
- **No-show.** The provider can claim after `checkInHeight + noShowBlocks`.
- **Check-in and check-out.** Explicit actions inside the slot window.
- **Deposit release.** `checkOut` sets `depositReleaseHeight = now +
  depositReviewBlocks`.
- **Event transfer cutoff.** `checkOutHeight - eventTransferCutoffBlocks`.
- **Expiry.** `expire` is permissionless after `holdUntilHeight`.

## Lazy expiry, not timers

Rholang has no scheduled callbacks. A held record carries its own deadline. A later hold that
sees `["held", oldId, until]` with `now >= until` may consume and replace it. The old
booking's `expire` is permissionless: it refunds the guest and attempts a release with the old
token. The release returns `"not holder"` if the slot was already reclaimed, and that is
ignored because the refund is independent.

A booking that is never expired leaves escrow in its purse. Guests are refund-incentivized;
`expire`, `settle` and `closeEvent` are permissionless, so anyone can finish a stalled
booking.

## The provider's calendar problem

Mapping dates to heights is a provider responsibility. Block times vary, chains stall, and a
genesis reset moves every height. A provider should:

- keep `maxLeadBlocks` conservative;
- publish the height mapping alongside the listing;
- treat `startHintMs` as display-only;
- re-provision slots after a chain reset.

The alternative — trusting the proposer's timestamp — trades a provider responsibility for a
consensus risk. r-share chooses the provider.

## What a guest should check before booking

- the slot IDs match the dates they want;
- the listing is active;
- the terms hash matches the terms shown off-chain;
- the slot's `startHeight` is in the future and within `maxLeadBlocks`;
- the cancellation windows are the ones they expect.

> **Where to go deeper.** [Listings, units and slots](listings-units-slots.md) shows the slot
> record. [Bookings and lifecycle](bookings-and-lifecycle.md) uses the heights. [Platform
> constraints](../reference/platform-constraints.md) lists the clock rules.
