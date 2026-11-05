# HAZOP

> **Status:** Baseline HAZOP with a P0/P1 remediation pass. The canonical rows are in
> `audit/registers/hazards.yaml`; P0/P1 deviations are fixed in v2, P2/P3 remain open.

## Nodes

| ID | Node |
|---|---|
| N01 | Install / ShareRoot |
| N02 | AdminGuard / ShareAdmin |
| N03 | Terms |
| N04 | Remedy |
| N05 | Payments |
| N06 | ProviderBook / ProviderContract |
| N07 | Listing creation |
| N08 | UnitFactory / schedules |
| N09 | Allocator |
| N10 | GuestBook / GuestLedger |
| N11 | Bond |
| N12 | BookingContractFactory |
| N13 | Booking requested/confirmed |
| N14 | Booking in-service |
| N15 | Settlement and claims |
| N16 | Event ticket |
| N17 | Pause/delist/admin |
| N18 | Reads and projections |
| N19 | Platform boundary |
| N20 | Upgrade/rotation/migration |

The full node definitions are in `audit/hazop/nodes.yaml`.

## Guide words

`NO/NOT`, `MORE`, `LESS`, `AS WELL AS`, `PART OF`, `REVERSE`, `OTHER THAN`, `EARLY`,
`LATE`, `BEFORE`, `AFTER`, `REPEAT`, `CONCURRENT`, `UNAUTHORIZED`, `STALE`, `LOST`,
`DUPLICATE`, `UNKNOWN`, `UNAVAILABLE`.

Parameters: amount, authority, signature count, nonce/opId, state/status, unit, slot,
booking, asset, deadline/height, cell, release token, admin key set and provider index.

## Risk scale

| Severity | Meaning |
|---|---|
| S4 Critical | irrecoverable loss/lock, unauthorized transfer, double sale, governance takeover, safety-relevant off-chain harm |
| S3 High | temporary loss/lock, bounded misallocation, privacy leak, platform-wide outage |
| S2 Medium | griefing, degraded liveness, wrong status |
| S1 Low | docs/UX/refusal mismatch |

Unresolved S4/L3+ rows block an implementation wave unless the disposition is an accepted
risk with a named owner.

## Highest-risk rows

| Hazard | Deviation | Finding | Status |
|---|---|---|---|
| HZ-001 | reset frees a booked/held slot | F-001 | fixed in v2 |
| HZ-002 | guestBookState lost on factory failure | F-002 | fixed in v2 |
| HZ-003 | pending op after Bond factory failure | F-003 | fixed in v2 |
| HZ-004 | provider captures rent without delivery | F-004 | fixed in v2 |
| HZ-005 | event settle before event end | F-005 | fixed in v2 |
| HZ-006 | duplicate admin keys satisfy threshold | F-006 | fixed in v2 |
| HZ-007 | global pause bypassed by direct acquire | F-007 | fixed in v2 |
| HZ-009 | arbitrary booking data accepted | F-009 | fixed in v2 |
| HZ-019 | flow manifest overclaims proof status | F-019 | fixed in v2 |
| HZ-026 | r-cash lookup can hang | none | accepted/control |
| HZ-029 | Bond double pay | none | defended |
| HZ-035 | projection leaks a token | none | defended |
| HZ-040 | terminal settlement leaves slots booked | F-021 | fixed in v2 |

## How to read a row

Each row records the node, arm, guide word, parameter, deviation, causes, on-chain and
operational consequences, optional safety consequence, controls, severity, likelihood, risk,
finding, test, Lean property, disposition and owner. A row is not a finding by itself; the
finding register carries the evidence and status. A hazard with no finding is a control or
an accepted risk, not proof that the code is correct.
