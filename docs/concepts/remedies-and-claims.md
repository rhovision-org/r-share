# Remedies and claims

> **Status:** Design proposal. Phase 1 implements fee, cancellation, no-show and deposit
> split; the generalized claim classes are Phase 2 targets.

A remedy is the deterministic money answer to a claim. A claim is an assertion by one party
that an obligation was breached, with a class and an evidence hash. The chain never decides
whether the assertion is true; it applies a default and a bounded override.

## The claim classes

| Class | Claimant / window | Default | Arbitration bound |
|---|---|---|---|
| `buyerUseBreach` | provider after checkout | claimed deposit to provider; remainder to buyer | ≤ deposit |
| `providerDeliveryBreach` | buyer after cure, or provider cancel | full rent + deposit + provider bond to buyer | ≤ rent + provider bond |
| `noFaultCancellation` | either party before cutoff | standard/event refund; both legs released | no fee outside Terms |
| `noShow` | provider after grace | rent net of fee; deposit released; provider bond released | rent ≥ cancellation floor |
| `damage` | provider after checkout | claimed amount from deposit; remainder to buyer | split within `[0, deposit]` |
| `partialPerformance` | either party | pro-rata rent by `performedBps`; fee pro-rata; remainder refunded | `performedBps` within the band |

The full table, pseudocode and worked examples are in
[the remedy table](https://github.com/PatrickMockridge/r-share/blob/main/spec/remedy-table.md).

## One claim arm

Phase 2 replaces three arms with two:

```
claim(class, amount, evidenceHash, opId)
resolveClaim(opId)
```

`arbitrate` remains. The class selects the claimant, the window and the remedy row. The
default is deterministic; the arbiter's `splitBps` is bounded by the row. This keeps the arm
count flat while adding the provider-delivery obligation.

## Evidence

Evidence is `{hash, kind, submittedBy, atHeight}`. The chain proves existence at a height,
not truth. The content lives off-chain; the arbiter and the parties read it. No on-chain
attestation or oracle is introduced.

## Why not an oracle

An oracle that settles quality or metering would become a trusted party in every booking.
The Pareto analysis rejects that trade: it adds trust without adding a primitive that the
deterministic table cannot express. Quality remains evidence; the remedy remains bounded.

## What the table does not cover

- subjective quality and emotional harm;
- insurance and liability beyond the bond;
- regulatory penalties;
- continuous per-block billing;
- a court order.

Those belong to the off-chain product or a fork with a different trust model.

> **Where to go deeper.** [Claim classes](../reference/claim-classes.md) is the compact
> reference. [Obligations and bonds](obligations-and-bonds.md) is the duty model. [Pareto
> analysis](https://github.com/PatrickMockridge/r-share/blob/main/spec/pareto.md) explains
> the trade.
