# Claim classes

> **Status:** Phase 1 verifies `buyerUseBreach`, `noFaultCancellation`, `noShow` and
> `damage` across all five kinds through `claimDeposit`, `claimNoShow`, `resolveDeposit` and
> `cancel`. `providerDeliveryBreach` and `partialPerformance` remain design-only; the class
> parameter is the Phase 2 generalization.

| Class | Claimant / window | Default remedy | Bound |
|---|---|---|---|
| `buyerUseBreach` | provider after checkout | claimed deposit to provider; remainder to buyer; rent/fee normal | ≤ deposit |
| `providerDeliveryBreach` | buyer after `checkInHeight + cureBlocks`, or provider cancel | full rent + deposit + provider bond to buyer; fee/rent 0 | ≤ rent + provider bond |
| `noFaultCancellation` | either party before the rule cutoff | standard/event refund; both legs released | no fee outside Terms |
| `noShow` | provider after `checkInHeight + noShowBlocks` | rent net of fee; deposit released; provider bond released | rent ≥ cancellation floor |
| `damage` | provider after checkout | claimed amount from deposit; remainder to buyer; rent/fee normal | split within `[0, deposit]` |
| `partialPerformance` | either party | pro-rata rent by `performedBps`; fee pro-rata; remainder refunded | `performedBps` within the band |

## Arms

```
BookingContract("claim", class, amount, evidenceHash, opId, ret)
  -> (true, Nil) | (false, "class" | "window" | "amount" | "evidence" | "replay")
BookingContract("resolveClaim", opId, ret)
  -> (true, Nil) | (false, "closed" | "not claimed" | "too early")
BookingContract("arbitrate", nonce, splitBps, sigs, opId, ret)
  -> (true, Nil) | (false, "replay" | "below threshold" | "bps" | "closed")
```

`claim` replaces `claimDeposit` and `claimNoShow`; `resolveClaim` replaces
`resolveDeposit`; `arbitrate` remains. The class selects the remedy row; the arbiter's
`splitBps` is bounded by that row.

## Evidence

`{hash, kind, submittedBy, atHeight}`. The chain never reads the content. The evidence hash is
an anchor, not a truth claim.

## Refusals

`"class"` (unknown class), `"window"` (claimant or height window), `"amount"` (outside the
bond or negative), `"evidence"` (missing or malformed hash), `"bps"` (split outside the
class bound), `"closed"` (terminal), `"replay"` (opId used).

> **Where to go deeper.** [Remedies and claims](../concepts/remedies-and-claims.md) is the
> narrative. [Remedy table](https://github.com/PatrickMockridge/r-share/blob/main/spec/remedy-table.md)
> is the normative table. [Bond arms](bond-arms.md) is the money side.
