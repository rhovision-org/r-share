# Cover use cases

> **Status:** Design. Each rung uses the same `CoverOffer`/`CoverPolicy`/`CoverClaim`/
> `CoverPool` engine; only the subject, roles, trigger, evidence and deposit effect change.

## Holiday letting (`subjectKind=booking`)

- Exposures: damage, theft, overstay, unpaid charges; provider cancellation/no-show,
  misrepresentation, deposit abuse; third-party injury.
- Provider cover: host protection, provider/platform payer, cap above deductible. Reduces
  the cash deposit to `hybrid` or `covered`.
- Guest cover: booking guarantee and liability waiver, guest/platform payer.
- Evidence: check-in/out inventory, photos, repair quotes, incident reports.
- Deposit effect: cash/hybrid/covered; `d <= Terms.deposit`.

## Events management (`subjectKind=ticket`/`event`)

- Exposures: organiser or venue cancels; event is not as described; ticket value prepaid;
  event liability.
- Buyer cover: cancellation/refund guarantee, event liability.
- Organiser cover: income cover for late cancellation or venue failure.
- Evidence: cancellation notice, venue notice, ticket record, event terms.
- Deposit effect: no security deposit; the prepaid ticket is the dead money.

## Ticketing (`subjectKind=ticket`)

- Exposures: counterfeit/resold ticket, organiser default, refund failure.
- Buyer cover: authenticity and transfer/resale guarantee, refund on cancellation.
- Organiser/platform cover: chargeback/fraud cover.
- Evidence: ticket credential, transfer record, entry scan, refund record.
- Deposit effect: prepaid ticket value; no cash deposit.

## Municipal bike share (`subjectKind=session`)

- Exposures: fleet damage/theft, station outage, rider injury/liability, unlock hold.
- Operator cover: fleet asset and availability cover.
- Rider cover: injury/liability, service-availability credit, unlock-hold waiver.
- Evidence: unlock/return telemetry, maintenance record, incident report.
- Deposit effect: unlock hold/usage deposit reduced or waived on covered sessions.

## Car rental (`subjectKind=booking`)

- Exposures: collision/damage, theft, overstay, cleaning; renter liability; third-party
  injury/property damage.
- Owner cover: asset protection above deductible.
- Renter cover: damage waiver (renter liability) and third-party liability, renter-paid.
- Evidence: inspection, photos, repair estimate, police/incident report.
- Deposit effect: large cash deposit can fall to the waiver deductible or zero.

## Robotics-as-a-service (`subjectKind=service_contract`, north star)

- Exposures: equipment damage, operator injury/liability, cyber/data, downtime and SLA
  shortfall, business interruption, provider non-performance.
- Provider cover: equipment, operator liability, cyber, business interruption; provider
  performance bond.
- Customer cover: delivery/SLA guarantee, downtime credit, non-performance refund,
  third-party injury.
- Evidence: sensor/telemetry logs, incident reports, SLA records, delivery/acceptance
  records, cyber incident reports.
- Deposit effect: retainer or performance bond reduced when covered; principal service
  prepayment protected by the customer-side guarantee.

## Cross-rung rules

- The premium payer is the insured or the platform; never the counterparty.
- A K-Asset quality estimate is required or the risk is labeled `insurer-only`.
- The trigger and evidence schema are configuration, not a new booking arm.
- The subject contract owns terminality and slot/session release; a cover policy cannot
  reopen a terminal subject.
- Claims use the r-share arbitration/evidence path for small deterministic claims; r-sure's
  human-adjudicated process is the facility/excess layer for large liability/title claims.
- The same pool accounting and solvency invariant apply across every rung.
