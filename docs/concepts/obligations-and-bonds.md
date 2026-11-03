# Obligations and bonds

> **Status:** Design proposal. The buyer leg is the current deposit; the provider leg and the
> generalized claim are Phase 2/4 targets.

A booking creates two duties at once. The provider must deliver the service for the interval;
the buyer must use the capacity responsibly and pay. A bond is how each duty answers for a
breach.

## Provider service delivery

The provider must:

- deliver the booked unit or capacity for the interval with the promised attrs;
- perform the service and hand over access or an operator by `checkInHeight`;
- cure a failed delivery within `cureBlocks`.

On-chain: the provider bond, the check-in/out heights, a claim class, an evidence hash and a
bounded arbitration split. Off-chain: whether the room was clean, the drone flew or the
operator was certified.

## Buyer responsible use

The buyer must:

- fund rent and deposit at booking;
- use the capacity within the attrs and the operating envelope;
- return it by `checkOutHeight`;
- report damage or an incident inside the claim window;
- transfer access only through an explicit ticket rule.

On-chain: the buyer bond, heights, claim class, evidence hash and a bounded split. Off-chain:
actual damage, misuse, weather and certification.

## One bond, two legs

```
buyer leg:    rent + deposit   refundTo buyer     forfeitTo provider
provider leg: providerBond     refundTo provider  forfeitTo buyer
```

- The buyer leg is funded in the booking deploy.
- The provider leg is funded at `accept` in per-booking mode, or reserved from a per-provider
  prefunded pool for instant booking.
- Each leg returns to its funder when the obligation is discharged.
- A remedy moves a leg only within its class bound.
- `Σ legs == Σ payouts + remaining`; `pay` is terminal and single-use.

The provider bond is **collateral, not a new party**. It does not attest facts or vote. It
makes provider non-delivery answerable without an oracle.

## Deterministic remedies

The remedy is a pure function:

```
remedy(terms, state, now, claimClass, splitBps?) -> buckets
```

The chain applies a deterministic default. The 2-of-3 arbiter may override within the class
bound. It cannot move money outside the table, and it does not judge the facts. If the
arbiter never acts, a permissionless default resolves the claim after the dispute window.

## What is not on-chain

- whether the service was good;
- whether the damage was the buyer's fault;
- whether the pilot was certified today;
- insurance, liability, law or safety.

Those are evidence and off-chain judgment. The chain anchors the hash and enforces the
bounded money outcome.

## Phase status

- Phase 1: buyer deposit, escrow Bond, deterministic fee/cancel/no-show/deposit split.
- Phase 2: obligation objects, generalized claim, bounded arbitration.
- Phase 3: per-booking evidence anchors.
- Phase 4: provider bond and cure window.

> **Where to go deeper.** [Remedies and claims](remedies-and-claims.md) is the money table.
> [Bond arms](../reference/bond-arms.md) is the interface. [Obligation schema](../reference/obligation-schema.md)
> is the data shape.
