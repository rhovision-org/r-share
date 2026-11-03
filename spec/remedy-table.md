# Remedy table

**Status:** Phase 1 implements and verifies `fee`, `cancel`, `noShow` and `depositSplit` for
`buyerUseBreach`, `noFaultCancellation`, `noShow` and `damage` across all five kinds. The
generalized class parameter, `providerDeliveryBreach` and `partialPerformance` remain Phase 2
design.

## 1. Inputs and outputs

```
remedy(terms, state, now, claimClass, splitBps?) -> [{owner, amount}] | (false, reason)
```

- `terms` is the versioned terms map (or its remedy projection).
- `state` is the booking state at the decision point (status, heights, rent, deposit, bond legs).
- `now` is the current block height.
- `claimClass` is one of the classes below.
- `splitBps` is the arbiter's bounded override, 0–10000, optional.
- Amounts are non-negative integers; the result sums to the bond total.

## 2. Buckets

| Bucket | Recipient |
|---|---|
| `providerRent` | provider |
| `providerDamage` | provider, from the deposit or provider bond |
| `guestRefund` | buyer, from rent, deposit or provider bond |
| `platformFee` | platform |

No other bucket exists. A bucket that would be negative refuses `"amount"`.

## 3. Claim classes

| Class | Claimant / window | Default remedy | Arbitration bound |
|---|---|---|---|
| `buyerUseBreach` | provider after checkout | claimed deposit to provider; remainder to buyer; provider bond released; rent/fee normal | reduce forfeiture; never above deposit |
| `providerDeliveryBreach` | buyer after `checkInHeight + cureBlocks`, or provider cancel | full rent + deposit refund; provider bond to buyer; fee/rent 0 | preserve cure or partial payment; never above rent + provider bond |
| `noFaultCancellation` | either party before the rule cutoff | standard/event refund; deposit released; provider bond released | no fee outside Terms |
| `noShow` | provider after `checkInHeight + noShowBlocks` | rent net of fee to provider; deposit released; provider bond released | reduce provider rent to the cancellation floor |
| `damage` | provider after checkout | claimed amount from deposit; remainder to buyer; rent/fee normal | split within `[0, deposit]` |
| `partialPerformance` | either party | pro-rata rent by `performedBps`; fee pro-rata; remainder refund; deposit released; provider bond released below `deliveryFloorBps` | adjust `performedBps` within the admissible band |

`performedBps` without a meter:
`clamp(floor((min(now,end)-start)*10000/(end-start)), 0, 10000)`.

## 4. Pseudocode

```
remedy(terms, state, now, class, split):
  require class in CLASSES
  require 0 <= split <= 10000 when split is present
  rent, deposit, bond = state.rent, state.deposit, state.providerBond
  fee = floor(rent * terms.feeBps / 10000)
  switch class:
    buyerUseBreach:
      claim = min(state.claim, deposit)
      return [provider: rent - fee + claim,
              platform: fee,
              buyer: deposit - claim]
    providerDeliveryBreach:
      return [provider: 0,
              platform: 0,
              buyer: rent + deposit + bond]
    noFaultCancellation:
      refund = cancellationRefund(terms, now, state.start, rent)
      return [provider: rent - refund,
              platform: 0,
              buyer: refund + deposit]
    noShow:
      return [provider: rent - fee,
              platform: fee,
              buyer: deposit]
    damage:
      claim = min(state.claim, deposit)
      return [provider: rent - fee + claim,
              platform: fee,
              buyer: deposit - claim]
    partialPerformance:
      performed = state.performedBps or timeProRata(...)
      providerRent = floor(rent * performed / 10000)
      fee = floor(providerRent * terms.feeBps / 10000)
      return [provider: providerRent - fee,
              platform: fee,
              buyer: rent - providerRent + deposit]
  if split is present: replace the class-specific share with split within its bound
```

The actual implementation must be a pure, total Rholang function in `04_remedy.rho` (or a
successor), with the same arithmetic for every class.

## 5. Conservation and rounding

- All amounts are integer minor units.
- Division is floor; the remainder stays with the party that did not receive the rounded share.
- `Σ buckets == rent + deposit + providerBond` for a bilateral bond, or
  `rent + deposit` for the Phase 1 buyer-only bond.
- No bucket is negative.
- A class bound is enforced before any payout; a violation refuses `"bps"` or `"amount"`.

## 6. Worked examples

### 6.1 Buyer-use breach

Rent 300, deposit 50, fee 5%, claim 30. Provider gets 285 + 30 = 315; platform 15; buyer 20.
Total 350 = rent + deposit.

### 6.2 Provider-delivery breach with a provider bond

Rent 300, deposit 50, provider bond 200. Buyer gets 550; provider 0; platform 0. Total 550.

### 6.3 No-show

Rent 300, deposit 50, fee 5%. Provider 285; platform 15; buyer 50. Total 350.

### 6.4 Partial performance at 50%

Rent 300, deposit 50, fee 5%, performedBps 5000. Provider rent 150, fee 7 (floor 7.5),
provider 143; platform 7; buyer 150 + 50 = 200. Total 350.

### 6.5 Damage claim

Rent 300, deposit 50, fee 5%, claim 20. Provider 285 + 20 = 305; platform 15; buyer 30.
Total 350.

## 7. Test vectors

Every class must have vectors for:

- the lower and upper bound of `splitBps`;
- zero and maximum claim amounts;
- rounding at `feeBps` 1, 500, 9999 and 10000;
- a missing or malformed terms field (refusal);
- an unknown class (refusal);
- a bucket sum mismatch (refusal).

The vectors live in `spec/coverage.yaml` and are exercised by the Phase 2 remedy tests.

## 8. Relationship to the current contracts

Phase 1 keeps `Remedy("fee"/"cancel"/"noShow"/"depositSplit")` and the existing
`claimDeposit`/`claimNoShow`/`resolveDeposit` arms. Phase 2 generalizes the claim arms to
`claim(class, amount, evidenceHash, opId)` and `resolveClaim(opId)`; `arbitrate` remains. The
Phase 1 arithmetic must match the legacy `Policy` formulas exactly; the Phase 2 table must
be a strict extension for the existing classes.
