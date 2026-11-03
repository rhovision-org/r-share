# Space-time primitives

> **Status:** Design proposal. Phase 1 implements capacity, slots, terms, a buyer bond and
> deterministic remedies; the bilateral obligation model below is the Phase 2 target.

A rental is a right to use capacity over an interval. The universal core makes that right
explicit, attaches the two obligations that come with it, and leaves every fact that cannot
be reproduced on every node off-chain.

## The model

```
Party      (ownerId, pubKey, role)
Interval   [start, end)                 block heights
Capacity   (resourceId, unitIds, attrs, slots)
Unit       (unitId, listingId, providerId, kind, attrs, schedule, status)
Slot       (unitId, slotId, start, end) free | held | booked
Terms      (schema, values, hash)       ordered, versioned
Obligation (obligor, kind, trigger, deadline, evidenceSchema, claimClass, cureBlocks)
Bond       (bondId, bookingId, legs)    buyer and provider legs
Evidence   (hash, kind, submittedBy, atHeight)
Remedy     (terms, state, now, claimClass, splitBps) -> buckets
Lifecycle  the state machine over all of the above
```

## Why these are the primitives

- **Capacity and interval** are the only two dimensions every rental shares. A room-night,
  a seat at a show, a bike-hour, a car-day and a machine window are the same pair.
- **Terms** is the agreed price and policy, content-addressed so a guest can prove what they
  booked.
- **Obligation** names the two duties that make a rental more than a reservation: the
  provider must deliver, and the buyer must use responsibly.
- **Bond** is the money that answers for a breach. A buyer leg and a provider leg use the same
  contract.
- **Evidence** anchors a fact at a height without asking the chain to judge it.
- **Remedy** is a pure function from terms, state, time and a claim class to money buckets.
- **Lifecycle** is the state machine that ties capacity, obligations and deadlines together.

## What stays off-chain

Quality, cleanliness, telemetry, weather, certification, identity, insurance, licensing,
consumer law and safety. The chain holds a hash if a dispute needs an anchor; it never reads
the content. A remedy may only depend on inputs identical on every node: heights, state and
verified signatures.

## How the pieces compose

- A listing publishes capacity and terms.
- A unit owns slots; each slot is an independent concurrency cell.
- A booking holds one or more slots, funds a bond and starts a lifecycle.
- The lifecycle records check-in, check-out, claims and arbitration.
- A remedy turns the facts into buckets; the bond pays them once.
- Evidence hashes travel with the claim; the arbiter decides a bounded split, not the facts.

## What this does not become

It does not become an on-chain oracle, a court, an insurer or a general policy VM. Those are
different trust models and different products. The core stays small so a fork can add them
without rewriting the booking path.

> **Where to go deeper.** [Obligations and bonds](obligations-and-bonds.md) applies the model
> to the two duties. [Remedies and claims](remedies-and-claims.md) is the money table.
> [Space-time spec](https://github.com/PatrickMockridge/r-share/blob/main/spec/space-time.md)
> is the normative proposal.
