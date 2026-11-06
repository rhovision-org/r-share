# Cover pricing

> **Status:** Design. Pins r-sure `c450072` (2026-11-06) and r-gen `c4f8a91`
> (`kasset.conventions/v1`). The variance rate is a convention, not a calibrated actuarial
> forecast.

## Source of truth

The cover layer does not invent a premium. It reuses r-sure's facility-level Risk and the
`SureMath` variance model:

- `SureMath("varianceStats", values, ret)` — variance rate from equally likely values;
- `SureMath("varianceRate", qualitySum, verdict, ret)` — engineering rate;
- `SureMath("varianceGap", engineeringBps, insurerBps, toleranceBps, ret)`;
- `SureMath("loadedPremium", expectedLoss, varianceBps, loadBps, ret)`;
- `Risk("assess", ...)` — expected loss, insurer variance, load, premium floor;
- `Risk("variance", ...)` — engineering and insurer estimates, signed gap, flag.

The engineering rate is:

```
Q = completeness + correctness + traceability + fitness   (4..20)
D = 20 - Q
verdictPenalty = 0 for valid, 1000 for partial
engineeringBps = D * 500 + verdictPenalty                  (0..9000)
```

`approval.liability.shareBps` is validated and recorded but is **not** part of
`engineeringBps`. It is an indemnity/co-insurance input, retained separately. Pricing it as
engineering variance is a category error.

## K-Asset input

The facility Risk reads an approved r-gen K-Asset through `KAsset("get")` and validates:

- quantity: non-empty unit, value >= 1;
- quality: completeness, correctness, traceability and fitness, each 1..5;
- verdict: `valid` or `partial` (`invalid` is not insurable);
- status `approved` with a non-nil `approval.liability`.

An insured may bind a variance attestation to a `rgen:kasset` evidence entry. The
attestation overrides the derivation, records its source, and is an off-chain engineering
claim, not a proof. If no engineering source exists, the risk is `insurer-only` and the gap
flag is set.

Per rung:

| Rung | K-Asset quality input |
|---|---|
| Lodging | property inspection, inventory, condition |
| Car rental | inspection, maintenance, mileage/condition |
| Events/ticketing | venue, organiser and production QA |
| Bike share | fleet maintenance, station/telemetry quality |
| RaaS | equipment, operator, sensor/telemetry and cyber QA |

## Facility and certificate

One r-sure Risk per facility (provider/asset class/epoch) carries expected loss, insurer
variance, load, premium floor and the engineering/insurer gap. A per-subject certificate
inherits the facility rate:

```
subjectPremium = max(subjectFloor, coverageCap * facilityRateBps / 10000)
subjectFloor   = facilityFloor * subjectExposure / facilityExposure
sum(subjectPremium) <= facility premiumTotal
```

The allocation is recomputed when a policy binds, releases or expires; rounding uses integer
minor units and the last certificate absorbs the remainder. A certificate below its floor is
refused. The pool reserves the cap and checks solvency before binding.

## Gap and disclosure

```
gapBps = insurerBps - engineeringBps
flagged = |gapBps| > toleranceBps   (default 2000)
```

The offer exposes `engineeringBps`, `insurerVarianceBps`, `gapBps`, `flagged`,
`premiumFloor` and `premiumTotal`. A provider/guest sees the disagreement before accepting.
An `insurer-only` risk is labeled as such. Do not hide the gap in a single "market price."

## r-sure fit

Phases 1–2 do not modify r-sure. The facility master policy's insured is the pool operator
(the platform or a licensed wrapper); r-sure pays the pool after its normal human-gated claim
process, and the pool pays certificates. No beneficiary or premium-payer change is required
for that topology.

Phase 3+ may add optional `beneficiary` and `premiumPayer` to r-sure Slip/Policy/Claim only
if direct certificate payout is required. Do not add a certificate primitive to r-sure; that
would duplicate r-share's `CoverPolicy`.

## Verification vectors

- quality 3/3/3/3, valid verdict -> `engineeringBps = 4000`;
- partial verdict adds 1000;
- liability share changes do not change `engineeringBps`;
- `loadedPremium(800, 5000, 2000) = 880`;
- a quote below the floor is refused;
- gap flag at tolerance 2000;
- insurer-only sets the flag;
- facility certificate sum equals the facility premium and no certificate is below its
  allocated floor.

These vectors must run against both the r-sure `SureMath` contract and the r-share pricing
adapter, with the same integer rounding.
