# Two-sided cover

> **Status:** Partial implementation. `CoverTerms`, `CoverBook`, `CoverPool`, the
> `CoverPolicy`/`CoverSettle` capabilities and the atomic `bookWithCover`/`attachCover`
> booking flow are implemented and tested on the isolated devnet. Claim adjudication is
> operator-authorized; the fixed claim-token primitive is implemented but the booking path
> currently pays through the operator. Client types and the platform-funded/guest-paid
> variants are not implemented.

## Purpose

Add protection for both sides of a rental, booking, ticket, session or service contract,
and use that protection to reduce locked deposits and other dead money. The layer is
subject-kind agnostic: `booking`, `ticket`, `session` and `service_contract` differ in
configuration, not in the policy engine.

The design follows one rule:

> **Premium payer is the insured or the platform, never the counterparty.**

A guest may buy guest cover. A provider may buy provider cover. The platform may fund a
baseline protection programme for either side and carry a master policy. A guest never pays
the provider's premium, and a provider never pays the guest's premium.

## Roles and products

| Product | Insured | Beneficiary | Payer | Trigger |
|---|---|---|---|---|
| `asset_protection` (provider) | Provider | Provider | Provider or platform | Guest damage, theft, overstay, unpaid charges |
| `liability` (guest or provider) | Responsible party | Injured party or insured by indemnity | Insured or platform | Third-party injury/property damage |
| `booking_guarantee` (guest) | Guest | Guest | Guest or platform | Provider cancel, no-show, misrepresentation |
| `income_cover` (provider) | Provider | Provider | Provider or platform | Late guest cancellation or lost use |
| `performance` (buyer) | Buyer | Buyer | Buyer or platform | SLA/downtime/non-performance |
| `purchase_default` (seller) | Seller | Seller | Seller or platform | Buyer misses a purchase instalment after cure |
| `purchase_nondelivery` (buyer) | Buyer | Buyer | Buyer or platform | Seller fails delivery, title or condition |
| `fraud` (buyer or platform) | Buyer/platform | Buyer/platform | Buyer/platform | Ticket authenticity, identity or chargeback |

The product catalogue is configuration. New rungs add products; they do not add a new
booking arm or a new booking lifecycle.

## Deposit modes

`Terms.deposit` remains the reference ceiling. A cover offer may set an effective cash
deposit `d` with `0 <= d <= Terms.deposit`. `Terms` stays exactly 20 fields and the
`rshare:terms:v1` hash stays unchanged.

| Mode | Bond holds | First loss | Cover |
|---|---|---|---|
| `cash` | `rent + D` | Guest deposit `D` | Optional add-on only |
| `hybrid` | `rent + d`, `0 < d < D` | Guest cash `d`, then provider deductible | Provider cover above `d + deductible`, up to cap |
| `covered` | `rent + 0` | Provider deductible | Provider cover above deductible, up to cap |
| `guest_waiver` add-on | unchanged | Guest retains `d_g` | Guest liability cover above `d_g` |

A cash deposit is collateral, not a premium. It is not a payment for cover. Cover premiums
never enter the booking Bond; the Bond remains `rent + d` and its conservation invariant is
unchanged.

## Objects and states

### CoverOffer

Immutable, published by a provider (owner-authenticated) or the platform (2-of-3 admin
authenticated), bound to a listing or subject and its terms/offer hash.

```
offerId, subjectKind, useCase, product, trigger,
insuredRole, beneficiaryRole, payerRole,
assetClass, assetId, payer, coverageCap, deductible,
claimWindowBlocks, evidenceSchema, depositEffect,
kassetUri?, engineeringBps?, insurerVarianceBps?, loadBps?,
expectedLoss, premiumFloor, premiumTotal, gapBps?, active
```

State: `draft -> active <-> paused -> retired`. `paused` blocks new binds; existing policies
continue. `retired` is terminal for new binds.

Validation: payer is the insured or platform; payer is never the counterparty; `d <=
Terms.deposit`; cap > 0; deductible >= 0; premiumTotal >= premiumFloor; K-Asset/insurer
pricing fields are present or explicitly `insurer-only`.

### CoverPolicy

One policy per subject per side.

```
policyId, subjectKind, subjectId, offerId, useCase, product, trigger,
insured, beneficiary, payer, premiumPaid, coverageCap, deductible,
status, claim, paid, ops, coverToken, rSurePolicyUri?, openedAtHeight
```

State: `bound -> active -> claimPending -> paid | denied | expired`; `active -> disputed ->
arbitrated -> paid | denied`; `active -> cancelled` on the full-refund path. `paid`,
`denied` and `expired` are terminal.

`coverToken` is private to the policy closure. Only the booking subject contract may call
`pay`; a guest or provider cannot pay itself.

### CoverClaim

Embedded in `CoverPolicy` for deterministic damage/cancellation claims in Phase 1; a
separate cell only for liability/title claims with an independent evidence lifecycle.

```
claimId, policyId, subjectId, claimant, trigger, amountClaimed,
evidenceHash, atHeight, status, awarded, depositPart, coverPart
```

### CoverPool

Pooled custody for premiums, capital, reserves and payouts. One pool per facility/epoch/
class plus a platform master pool. Custody is an ERTP purse held in closure, not an r-cash
account; the pool resolves `Payments("issuer")`, keeps its purse private, and pays through
`Payments("depositFacet")`.

```
poolId, operatorId, assetId, capital, premiumIn, premiumRefunded,
claimsPaid, feesOut, outstandingCover, capacity, solvencyBps,
reserves, policies, paidFor, status, ops, nonce
```

## Claim and payout order

1. The subject contract records the claim and evidence.
2. Deposit/collateral is applied first, up to the effective deposit or deductible.
3. The responsible party's deductible is retained.
4. The pool pays the remaining covered amount, capped at `coverageCap`.
5. A second policy may pay only the unrecovered excess of the same loss; one `lossId` caps
   total recovery at the assessed loss.
6. A policy cannot reopen a terminal booking. The booking terminality and slot release
   happen before Bond pay and pool pay; any failure after value moves rolls the whole deploy
   back.

## Premium funding

- **Guest-paid guest cover:** collected inside the guest's atomic book deploy. Failure rolls
  back rent, deposit, premium, slots and policy creation.
- **Provider-paid provider cover:** the provider prefunds a reserve at listing activation or
  on a schedule. The guest deploy may debit the reserve only for a bounded, offer-derived
  premium with a booking id and replay guard. A guest deploy never withdraws from a provider
  account.
- **Platform-funded protection:** the platform prefunds a master pool from its fee share;
  per-subject certificates debit the pool. The platform is the master insured; the
  provider/guest is the certificate beneficiary.

## r-sure boundary

r-sure is used at the facility layer, not as a per-booking placement:

- Facility-level Risk, priced with `SureMath.varianceRate`, `varianceGap` and
  `loadedPremium`, backed by a master policy or pool;
- r-sure capital/reinsurance for excess-of-loss and aggregate stop-loss above the r-share
  pool retention;
- r-share certificates for the per-subject policy, claim and payout.

r-sure's single-insured, human-gated per-risk Policy is not minted per small booking. A full
r-sure policy per booking is rejected in this design.

## Invariants

- **C1 Payer:** `premiumPayer ∈ {insured, platform}`; never the counterparty.
- **C2 Premium conservation:** premium is paid once and split between pool, platform fee and
  any facility/broker leg; no money creation.
- **C3 Claim conservation:** `depositPart + coverPart + otherRecovery == award`; total
  recovery never exceeds the assessed loss.
- **C4 Cap and deductible:** `coverPart <= min(coverageCap, award - depositPart)`;
  `depositPart <= effectiveDeposit`.
- **C5 Pool solvency:** `cash >= solvencyBps * outstandingCover / 10000`; `pay` refuses
  `"solvency"` before moving value and the subject remains retryable.
- **C6 Single terminal outcome:** a cover claim cannot reopen a terminal booking.
- **C7 Token gate:** only the subject contract's `coverToken` can authorize a pool payout.
- **C8 Premium floor:** `premiumTotal >= loadedPremium(expectedLoss, varianceBps, loadBps)`.
- **C9 Liability share:** `approval.liability.shareBps` is an indemnity/co-insurance input,
  not an engineering-variance signal.
- **C10 Subject kinds:** the policy engine is identical for booking, ticket, session and
  service contract; use case is configuration.
- **C11 Evidence:** an evidence hash is an anchor, not proof; adjudication remains a trust
  point.
- **C12 Disclosure:** a covered deposit must show the deductible, cap, exclusions and claim
  path before the guest books.

## Non-goals

- Not a regulated insurer or broker. r-sure is explicitly not an insurer.
- No automatic truth from evidence or telemetry.
- No elimination of all deposits; a first-loss/deductible remains.
- No new Terms fields and no change to the 20-field terms hash.
- No per-booking r-sure Slip/Policy.
- No shared hot-path pool with unbounded liability; pools are sharded and capped.
