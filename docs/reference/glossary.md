# Glossary

> **Status:** Reference. Terms used throughout the book.

**arbitration** — the 2-of-3 admin action that splits a deposit after a dispute.

**attrs** — free-form data on a unit (station, class, certification hash, and so on). The
chain does not interpret it.

**block height** — the monotonic number used for every deadline and interval. Not the block
timestamp.

**booking** — a guest's claim on one or more slots, with rent and deposit in escrow, moving
through a lifecycle.

**bilateral bond** — a bond with a buyer leg and a provider leg; each returns to its funder
unless a remedy forfeits it within a class bound.

**bucket** — one of `providerRent`, `platformFee`, `guestRefund`, `providerDamage` in a
settlement.

**buyer leg** — the buyer's rent + deposit in a bilateral bond.

**claim class** — a named remedy row: buyer-use breach, provider-delivery breach, no-fault
cancellation, no-show, damage or partial performance.

**cure window** — the provider's bounded chance to fix a failed delivery before a buyer claim.

**claim** — a provider's recorded damage amount, bounded by the deposit, with an evidence
hash.

**component key** — a `component:<name>` entry in the r-share Root.

**deposit facet** — a receive-only r-cash account capability used for settlement payouts.

**deploy** — one signed Rholang term; atomic with rollback.

**escrow purse** — the per-booking ERTP purse held by the Bond.

**evidence anchor** — a hash plus kind, submitter and height; existence at a height, not
truth.

**generation** — the counter in a free slot record, bumped on release.

**guest** — the party booking capacity.

**hold** — a time-bounded reservation of slots before confirmation.

**install term** — a deployable Rholang file under `contracts/src/deploy/`.

**lazy expiry** — reclaiming an expired hold when a later holder encounters it, instead of a
timer.

**listing** — a provider's published offer: kind, terms, status and units.

**minor units** — integer units of an asset. No floats.

**no-show** — a confirmed booking whose guest never checked in; the provider claims after the
grace window.

**phlo** — the node's deploy cost unit.

**provider** — the party that owns or operates the capacity.

**provider leg** — the provider's performance bond in a bilateral bond.

**Pareto dominance** — candidate A dominates B when A covers at least as much, trusts no more
and is no worse on bytes, cells, dispute steps or liveness, with one strict improvement.

**release token** — a fresh unforgeable name that authorizes a slot release or confirm.

**reset token** — the unit-held token that authorizes a provider reset of a slot.

**RaaS** — robotics-as-a-service; the north star.

**rent** — `pricePerSlot * slotCount`.

**settlement** — the deterministic split of the escrow into buckets.

**slot** — the atom of capacity: a named interval with a start and end height.

**slot ID** — the provider-chosen string identifying a slot within a unit.

**ticket** — a booking projection for events: holder, redeemed, cutoff.

**unit** — the bookable thing: a room, seat, bike, vehicle or machine.

**terms hash** — BLAKE2b-256 over the ordered terms string.

> **Where to go deeper.** [Data schemas](data-schemas.md) is the field list. [Arm
> reference](arms.md) is the signatures. [Capacity over an interval](../concepts/capacity-over-an-interval.md)
> is the model.
