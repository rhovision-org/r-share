# Cover economics

> **Status:** Design. These are scenario formulas and explicit assumptions, not a forecast
> or a production price list.

## Dead money

A cash deposit `D` locked for `T` days at an annual opportunity cost `r` costs the guest:

```
deadMoney = D * r * T / 365
```

For a booking with rent `R`, a hybrid effective deposit `d` frees `D - d`. The guest also
pays a cover premium `p_g` only if the guest buys guest-side cover; provider-side cover is
paid by the provider or the platform.

Example, not a price:

```
R = 300, D = 50, d = 10, cap = 200
provider premium = 6 (provider/platform)
guest cash before = 350
guest cash after  = 310
cash freed        = 40 (80% of the deposit)
```

If the guest also buys a liability waiver for `p_g`, the guest pays `p_g` but reduces tail
liability and may justify a smaller `d`. The comparison is `p_g` against `deadMoney` plus the
value of the risk transfer; it is not automatically favourable.

## Premium components

For a facility-level risk:

```
expectedLoss E
insurer variance rate varianceBps (0..10000)
loadBps (0..10000)
premiumFloor = E + E * varianceBps * loadBps / 100,000,000
```

The facility quotes `premiumTotal >= premiumFloor`. A per-subject certificate must not quote
below its allocated floor. The r-share pool does not invent a price; it reuses the r-sure
measure.

The facility allocation rule is a Phase 0 decision. A defensible starting rule is:

```
subjectPremium = max(subjectFloor, coverageCap * facilityRateBps / 10000)
sum(subjectPremium) <= facility premiumTotal
```

where `subjectFloor` is the facility floor scaled by the subject's share of the facility
exposure. The rule must be conservative: certificates never undercut the floor and the pool
never promises more premium than the facility collects.

## Pool solvency

```
cash = capital + premiumIn - premiumRefunded - claimsPaid - feesOut
outstandingCover = sum(active policy caps)
cash >= solvencyBps * outstandingCover / 10000
```

`pay` refuses `"solvency"` before moving value. The pool is paused for new binds, the
platform tops it up, or an r-sure recovery restores it. Fail closed.

## Claim economics

For one loss with assessed amount `L`:

```
depositPart = min(L, effectiveDeposit)
deductiblePart = min(L - depositPart, deductible)
coverPart = min(L - depositPart - deductiblePart, coverageCap)
uncovered = L - depositPart - deductiblePart - coverPart
```

The uncovered amount remains the responsible party's liability or a later excess layer. A
second policy can pay only `uncovered` for the same `lossId`; total recovery cannot exceed
`L`.

## Simulation

The Phase 0 simulation must state, not hide, its inputs:

- portfolio size; rent `R`; `Terms.deposit D`; effective `d`; cap `C`; deductible;
- loss frequency `f` and a severity distribution (illustrative, not observed);
- claim filing rate, arbitration uphold rate, moral-hazard multiplier for covered mode;
- expense load, capital charge, r-sure cession/commission/levy and solvency ratio;
- premium rule above.

Outputs:

- guest cash locked, cash freed, provider expected net recovery;
- pool loss ratio, break-even premium, probability of insolvency over N subjects;
- conservation of integer minor units with the last-bucket remainder;
- the same formulas as `CoverPool` and `CoverPolicy`.

A mismatch between the off-chain simulation and the on-chain arithmetic is a correctness
bug, not a rounding detail.

## Non-claims

- No claim that a premium is "actuarially fair."
- No claim that covered bookings are risk-free.
- No claim that the platform can fund unlimited protection from its fee share.
- No claim that a covered deposit is free: the guest may still owe a deductible, and the
  provider may still face uncovered loss above the cap.
