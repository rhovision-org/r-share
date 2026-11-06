# Why primitives

> **Status:** Design stance. Not a verification claim.

Most rental software is a product: a database with a calendar, a payment integration, an
admin panel, and a policy engine hidden in code. It works for one business and is hard to
reuse for the next. r-share takes the opposite position. It ships the smallest on-chain
layer that every rental needs, and leaves the product to the fork.

## The four primitives

1. **Capacity.** Something can be used by a bounded number of parties at a time.
2. **Interval.** The use is bounded in time. A night, a show, an hour, a day, a session.
3. **Escrow.** Money is committed before use and released after the facts are known.
4. **Policy.** Cancellation, no-show, deposit and damage rules are deterministic and agreed
   before the booking.

Everything else is a product decision: the UI, the calendar, the identity provider, the
support desk, the insurance, the telemetry.

## Why a chain

A rental is a multi-party state machine with money in the middle. The parties do not fully
trust each other, and the operator may disappear. A chain gives:

- one shared ordering of holds, confirms, cancels and settlements;
- atomicity: a failed booking does not leave a slot half-held;
- verifiable policy: the refund arithmetic is the same for every guest;
- an escrow that neither party controls alone;
- a public record of listings, bookings and settlements.

The chain is not the product. It is the settlement substrate under the product.

## Why a primitive repo, not a marketplace

A marketplace owns the customer, the pricing, the ranking and the dispute process. A
primitive repo owns none of those. That is the point:

- **Forkable.** Copy the contracts and the harness into your own repository. Rename it.
  Change the terms and the policy branch. Keep the parts that already work.
- **Auditable.** The safe surface is 14 deploy terms (19 with the experimental cover demo), four shared fragments and a small policy
  contract. You can read all of it.
- **Composable.** A letting product, an events product and a fleet product can share the
  same escrow and settlement code while having completely different front ends.
- **Community-owned.** A fix to concurrency or settlement benefits every fork. A new
  resource class is a case study and a fixture, not a rewrite.

## What "copy this" means

Three kinds of reuse, in increasing effort:

1. **Copy the terms and the policy.** Keep the contracts. Change `kind`, `attrs`, slot IDs
   and the terms. This covers lodging, events, bikes, cars and most machines.
2. **Copy a policy branch.** If your cancellation arithmetic differs, add a pure branch to
   `Remedy` and keep the booking state machine.
3. **Copy the repo.** If your trust model, money flow or regulation differs, fork and change
   the contracts. Keep the provenance and license notices.

If you find yourself adding a booking arm for a new inventory type, stop. The primitive is
wrong, or the requirement is a new primitive that deserves a design discussion.

## The tradeoffs we accepted

- **A reference client, no server.** The repo ships a Rust SDK, CLI, WASM wallet and React
  reference app, but no backend server, indexer or off-chain database. A fork owns its
  product UI and operations.
- **Opaque time.** Slot IDs are provider-chosen strings and all enforcement is in block
  heights. Calendar semantics live off-chain. This avoids trusting a proposer's clock, at
  the cost of a provider mapping dates to heights.
- **Public data.** Bookings, addresses, dates and prices are on-chain. There is no privacy.
- **No timers.** Expiry is lazy and needs a caller. Guests are refund-incentivized, and
  `expire`/`settle`/`closeEvent` are permissionless.
- **A small admin surface.** 2-of-3 pause, delist and arbitration, not a full governance
  system.
- **AGPL.** Network operation of a modified version carries the source-offer obligation.

These are documented, not hidden. A fork can change them, but it should do so deliberately.

> **Where to go deeper.** [Capacity over an interval](../concepts/capacity-over-an-interval.md)
> is the model. [The use-case ladder](use-case-ladder.md) is the progression. [Provenance and
> license](../appendix/provenance-and-license.md) covers the AGPL.
