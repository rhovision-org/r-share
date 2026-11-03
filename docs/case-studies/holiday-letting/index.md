# Holiday letting: a full walkthrough

> **Status:** Implemented, not yet exercised. The contract path exists and passes the static
> and parser gates; the provider, listing, booking and lifecycle suites are planned (M2–M4).

This is the reference case study. It follows one stay from the terms the host chooses to the
final settlement, using the actual arms and data shapes. It is deliberately concrete: a
two-room house, one guest, three nights, one deposit, one cancellation policy.

## The scenario

- **Provider:** Ana, who runs a two-room house by the coast.
- **Guest:** Ben, who wants three nights in room 1.
- **Asset:** `REV` (the devnet asset).
- **Price:** 100 minor units per night; deposit 50; platform fee 5%.
- **Policy:** full refund until 48 blocks before check-in; 50% of rent until 12 blocks
  before; after that the deposit only. No-show after 6 blocks of grace.

The numbers are illustrative. Every amount is integer minor units.

## The path

```
terms -> listing -> units -> slots -> active
                                      |
guest -> ledger -> hold + escrow -> booking -> accept -> checkIn -> checkOut -> settle
                                      |
                                      +-> reject / cancel / expire / noShow / dispute
```

Figure: the walkthrough path. Each arrow is an arm call; the chapters follow the order.

## Reading order

1. [Model the stay](01-model-the-stay.md) — choose the unit, slots, terms and policy.
2. [Provision the listing](02-provision-the-listing.md) — register the provider, create the
   listing, add rooms and slots, activate.
3. [Book and hold escrow](03-book-and-escrow.md) — create the guest ledger, book, accept,
   and understand what happens inside the one atomic deploy.
4. [Live the stay](04-live-the-stay.md) — check in, check out, cancel, no-show.
5. [Settle, dispute and clean up](05-settle-and-handle-failure.md) — settle, claim damage,
   dispute, arbitrate, expire.

## What this walkthrough does not claim

- No behavioural suite has passed on a healthy devnet. The commands in the chapters are the
  intended path; the suites that exercise them are placeholders until M2–M4.
- The off-chain booking UI, calendar, identity, support and consumer-law compliance are
  Ana's product, not this repository.
- The booking is public. Do not put a real guest's address or personal data on-chain.

> **Where to go deeper.** [Listings, units and slots](../../concepts/listings-units-slots.md)
> is the concept map. [Bookings and lifecycle](../../concepts/bookings-and-lifecycle.md) is
> the state machine. [Arm reference](../../reference/arms.md) has every signature.
