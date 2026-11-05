# HAZOP worksheet summary

The canonical rows are in [`../registers/hazards.yaml`](../registers/hazards.yaml); the
node list is in [`nodes.yaml`](nodes.yaml). This page is the readable index.

## Nodes

| Node | Area | Seed hazards |
|---|---|---|
| N01 | Install / ShareRoot | HZ-021, HZ-022, HZ-023 |
| N02 | AdminGuard / ShareAdmin | HZ-006, HZ-024, HZ-025 |
| N03 | Terms | HZ-013, HZ-015 |
| N04 | Remedy | HZ-017, HZ-030 |
| N05 | Payments | HZ-008, HZ-026, HZ-027, HZ-028 |
| N06 | ProviderBook / ProviderContract | HZ-007, HZ-012 |
| N07 | Listing creation | HZ-009, HZ-037 |
| N08 | UnitFactory / schedules | HZ-038 |
| N09 | Allocator | HZ-001, HZ-011, HZ-034, HZ-035 |
| N10 | GuestBook / GuestLedger | HZ-002, HZ-003, HZ-033 |
| N11 | Bond | HZ-016, HZ-029 |
| N12 | BookingContractFactory | HZ-009 |
| N13 | Booking requested/confirmed | HZ-004 |
| N14 | Booking in-service | HZ-039 |
| N15 | Settlement and claims | HZ-005, HZ-016, HZ-030, HZ-040 |
| N16 | Event ticket | HZ-005, HZ-031, HZ-032 |
| N17 | Pause/delist/admin | HZ-007, HZ-014 |
| N18 | Reads and projections | HZ-010, HZ-018, HZ-019, HZ-035 |
| N19 | Platform boundary | HZ-020, HZ-026, HZ-033 |
| N20 | Upgrade/rotation/migration | HZ-022, HZ-023, HZ-036 |

## Highest-risk rows

| Hazard | Deviation | Severity | Finding | Disposition |
|---|---|---|---|---|
| HZ-001 | reset frees a booked/held slot | S4 | F-001 | mitigate |
| HZ-002 | guestBookState lost on factory failure | S4 | F-002 | mitigate |
| HZ-006 | duplicate admin keys satisfy threshold | S4 | F-006 | mitigate |
| HZ-003 | pending op after Bond factory failure | S3 | F-003 | mitigate |
| HZ-004 | provider captures rent without delivery | S3 | F-004 | mitigate |
| HZ-005 | event settle before event end | S3 | F-005 | mitigate |
| HZ-007 | global pause bypassed by direct acquire | S3 | F-007 | mitigate |
| HZ-009 | arbitrary booking data accepted | S3 | F-009 | mitigate |
| HZ-019 | flow manifest overclaims proof status | S1/S3 | F-019 | mitigate |
| HZ-026 | r-cash lookup can hang | S3 | none | mitigate |
| HZ-029 | Bond double pay | S4 | none | mitigate |
| HZ-035 | projection leaks a token | S4 | none | mitigate |

## Method

For each node the audit applied the guide words in `nodes.yaml` to the listed parameters
and recorded credible deviations. A row is not a finding by itself; the finding register
carries the evidence and status. A hazard with no finding is a control or an accepted
operational risk, not a claim that the code is correct.
