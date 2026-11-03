# Universal space-time primitives with bilateral obligations

**Status:** Design proposal. Phase 1 (the modular base layer) is implemented; the bilateral
obligation model in this document is the Phase 2 target. It is not implemented or verified.

## 1. Purpose and scope

Define the smallest on-chain primitive set that covers a rental of capacity over an interval
with two obligations:

- the provider must deliver the service (access, machine, room, seat) for the interval;
- the buyer must use the capacity responsibly and pay.

The model must cover holiday letting, events/ticketing, municipal bike share, car rental and
robotics-as-a-service without adding a booking arm per class. It deliberately does not
adjudicate facts, quality or liability on-chain.

## 2. Notation and types

```
Party      = (ownerId, pubKey, role)              role in {provider, buyer, platform, arbiter}
Interval   = [start, end)                         heights, start < end
Capacity   = (resourceId, unitIds, attrs, slots)
Unit       = (unitId, listingId, providerId, kind, attrs, schedule, status)
Slot       = (unitId, slotId, start, end)         state in {free, held(id,until,token), booked(id,token)}
Terms      = (schema, values, hash)               ordered, versioned, content-addressed
Obligation = (obligor, kind, trigger, deadline, evidenceSchema, claimClass, cureBlocks, severity)
Bond       = (bondId, bookingId, legs, state)     buyer and provider legs
Evidence   = (hash, kind, submittedBy, atHeight)  content off-chain
Remedy     = (terms, state, now, claimClass, splitBps?) -> buckets
Lifecycle  = state machine over capacity + obligations + deadlines
```

## 3. Party and identity

A party is a deployer-derived REV address plus a role. There is no KYC, legal identity or
reputation in the core. Owner arms derive the address from the deploy's own `deployerId`;
admin and arbiter decisions are 2-of-3 signatures verified by `AdminGuard`.

## 4. Time

All intervals are half-open block-height ranges. `a.end == b.start` means adjacency. The
chain never parses dates and never reads the proposer's wall clock. Deadlines are heights:
holds, cancellation cutoffs, no-show grace, check-in/out, deposit review, dispute windows.
There are no timers; expiry is lazy and permissionless.

## 5. Capacity, unit and slot

- **Capacity** is a right to use a set of units over a set of intervals. Multi-unit capacity
  covers a machine plus an operator plus a site.
- **Unit** is a resource instance: room, seat, bike, vehicle or machine, with `attrs` and a
  schedule.
- **Slot** is the atomic reservable interval and the concurrency cell. A slot is exactly
  `free`, `held` or `booked`. A disjoint slot never contends with another.

The current implementation keeps one private `SlotContract` per slot and a `UnitContract`
per unit. `UnitFactory` owns the stateless multi-unit hold/release/confirm orchestration.

## 6. Terms schema and encoding

The current `rshare:terms:v1` canonical string is

```
kind|assetId|pricePerSlot|deposit|minSlots|maxSlots|instantBook|contiguous|
holdBlocks|depositReviewBlocks|disputeWindowBlocks|noShowBlocks|
fullRefundBeforeBlocks|partialRefundBps|partialRefundBeforeBlocks|feeBps|
maxLeadBlocks|maxSlotBlocks|maxUnits|eventTransferCutoffBlocks
```

`Terms("fields")` is the only ordered field list; `Terms("hash")` hashes the values with
Codec/BLAKE2b-256. `Terms("booking")` and `Terms("remedy")` return projections so callers do
not repeat the field order.

Phase 2 proposes `rshare:terms:v2` with length-prefixed values (the r-cash v3 encoding) and
an `obligations` section: provider standard hash, buyer-use hash, cure blocks, provider bond,
late-return policy, evidence schema and arbitration policy. The v2 encoding must ship with
test vectors before it is used.

## 7. Obligations

### 7.1 Provider service delivery

The provider must deliver the booked capacity for the interval with the promised attrs,
perform the service, hand over access or operator by `checkInHeight`, and cure a failed
delivery within `cureBlocks`.

| On-chain | Off-chain |
|---|---|
| provider bond, check-in/out heights, claim class, evidence hash, bounded arbitration | whether the room was clean, the drone flew, the operator was certified |

### 7.2 Buyer responsible use

The buyer must fund rent plus deposit, use the capacity within the attrs/operating envelope,
return it by `checkOutHeight`, report damage or an incident within the claim window, and
transfer access only through an explicit ticket rule.

| On-chain | Off-chain |
|---|---|
| buyer bond, heights, claim class, evidence hash, bounded arbitration | actual damage, misuse, weather, certification |

An obligation is named data, not a new state machine:

```
{"obligor": "provider" | "buyer",
 "kind": "deliver" | "use" | "pay" | "return",
 "trigger": "checkInHeight" | "checkOutHeight" | "claimWindow",
 "deadlineHeight": Int,
 "evidenceSchema": Hash,
 "claimClass": String,
 "cureBlocks": Int,
 "severity": "minor" | "major"}
```

## 8. Bond

One `Bond` contract per booking, with two legs:

```
buyer:    {role:"buyerBond",    amount: rent + deposit, refundTo: buyer,    forfeitTo: provider}
provider: {role:"providerBond", amount: providerBond,   refundTo: provider, forfeitTo: buyer}
```

- The buyer leg is funded in the booking deploy; the provider leg is funded at `accept` in
  per-booking mode, or reserved from a per-provider prefunded pool for instant booking.
- Each leg returns to its funder when its obligation is discharged; remedy buckets move a leg
  to the counterparty or platform within the bounds in [remedy-table.md](remedy-table.md).
- `Σ legs == bondTotal == Σ payouts + remaining`; `pay` is terminal and single-use; `balance`
  reports state.
- Arbitration reuses the booking's captured 2-of-3 set and `AdminGuard`, domain `"arbiter"`,
  with a class-bounded `splitBps`. No new oracle or trusted party.

Phase 1 implements the buyer leg only (the current deposit + rent). Phase 2 adds
`Bond.fund(leg, payment, ret)` for the provider leg; Phase 4 adds provider funding and the
cure window.

## 9. Evidence and anchors

Evidence is an opaque `{hash, kind, submittedBy, atHeight}` record in the booking or claim.
The chain proves existence at a height, not truth. Content lives off-chain (inspection
reports, telemetry, video, repair quotes). The chain never adjudicates the content.

Phase 3 proposes per-booking evidence anchors and challenge windows. A record is a hash plus
metadata; no on-chain oracle or attestor is introduced.

## 10. Remedy algebra

`remedy(terms, state, now, claimClass, splitBps?) -> [{owner, amount}] | (false, reason)`.

Amounts are non-negative integers; the outputs sum to the bond total; only the arguments
affect the result; unknown classes and out-of-range splits refuse. The full table is in
[remedy-table.md](remedy-table.md).

## 11. Lifecycle

The lifecycle is the state machine over capacity, obligations and deadlines. It has one
terminal outcome per booking and a permissionless path to it. The full graph and race
semantics are in [lifecycle.md](lifecycle.md).

## 12. Arbitration and bounded override

The arbiter signs a class-bounded `splitBps`; it does not attest facts and cannot move money
outside the class bounds. `AdminGuard` verifies the signature; the nonce is stored on the
booking. If the arbiter never acts, a permissionless default resolves the claim after the
dispute window.

## 13. On-chain and off-chain boundary

| On-chain | Off-chain |
|---|---|
| capacity, slots, terms hash, bonds, deterministic remedy, arbitration split, evidence hashes | evidence content, quality, telemetry, identity, insurance, licensing, consumer law, safety |

An on-chain remedy may only be a pure function of inputs identical on every node: heights,
states and verified signatures. Anything else is evidence, not adjudication.

## 14. Invariants

1. One slot is exactly free, held or booked.
2. One booking has one terminal outcome.
3. `Σ legs == bondTotal == Σ payouts + remaining`; no double pay.
4. Remedy buckets are non-negative and sum to the bond total.
5. Provider-bond forfeiture never exceeds the provider leg; buyer-deposit forfeiture never
   exceeds the deposit.
6. A slot releases only for its holder's token.
7. A booking's `rent == pricePerSlot * slotCount`; the deposit is separate.
8. No two bookings hold one slot at once; a lapsed hold is reclaimable.
9. Every non-terminal state has a permissionless terminal path after its deadline.

## 15. Reference interfaces

The Phase 1 interfaces are in `docs/reference/arms.md`. Phase 2 adds:

- `Bond.fund(leg, payment, ret)`
- `BookingContract.claim(class, amount, evidenceHash, opId, ret)`
- `BookingContract.resolveClaim(opId, ret)`
- `Terms("obligations", terms, ret)` and a v2 hash.

These replace `claimDeposit`, `claimNoShow` and `resolveDeposit` without increasing the arm
count: three arms become two plus the existing `arbitrate`.

## 16. Failure modes and non-goals

- No on-chain oracle, fact adjudication, KYC, insurance or legal enforcement.
- No arbitrary per-listing policy code; Remedy is a closed, versioned table.
- No continuous metering in the core; metering is a bounded Phase 5 extension.
- No global availability cache, counter, live-name registry or slot-group cell.
- No cross-shard settlement; tickets remain an event specialization.
- Remedy-table explosion is bounded by versioning, a class cap and refusing unknown classes.
- Provider-bond illiquidity is bounded by an optional per-booking or prefunded mode, default
  zero.
- Evidence bloat is bounded to hashes, kinds, submitters and heights.

## 17. Glossary

See `docs/reference/glossary.md` for the shared vocabulary. Terms introduced here: claim
class, buyer leg, provider leg, cure window, bounded override, evidence anchor.
