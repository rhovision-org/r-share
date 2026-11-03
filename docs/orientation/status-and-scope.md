# Status, scope and honest limits

> **Status:** This chapter defines the labels used everywhere else in the book.

r-share has static, parser and one recorded behavioural run. On 2026-11-03, a fresh devnet
built from rholo `e2d073d` with r-cash `49b441d` passed `unit_root`, `unit_payments`,
`unit_lifecycle` and `unit_claims`. The lifecycle run verifies the dispute gate, the
arbitration gate and permissionless `resolveDispute`; the claims run verifies
`buyerUseBreach`, `noFaultCancellation`, `noShow` and `damage` across all five kinds
(20/30 coverage cells). `providerDeliveryBreach`, `partialPerformance`, concurrency,
conservation and pause remain design or not run; every claim is labelled accordingly.

## Status labels

| Label | Meaning |
|---|---|
| **Implemented** | Source exists; `make spec-check` and `make parse-check` pass. No behavioural claim. |
| **Implemented, not yet exercised** | Source exists and the gates pass; no healthy-devnet suite run is recorded. |
| **Planned (M2)–(M8)** | A placeholder suite or feature is scheduled; `harness/unit_pending.py` exits 2. |
| **Mapping only** | No dedicated suite; the use case rides the shared contract path and is unverified. |
| **Design intent** | Roadmap or north-star direction; not implemented. |
| **Out of scope** | Deliberately not in this repository. |

Never read "implemented" as "tested", "audited", "production-ready" or "battle-tested". Those
words are reserved for a recorded command and artifact.

## What is implemented

| Area | Evidence |
|---|---|
| 13 install terms | `make build`, `make measure`, `make parse-check` |
| Component catalogue and state cells | `make check-arch` |
| No `select`, no capability captures, no unknown placeholders | `make check-select`, `make check-captures`, `make check-placeholders` |
| Integer money only | `make check-money` |
| Renderer | `harness/install.py --dry-run` renders every term with real placeholders |
| `unit_root` | passing on the recorded devnet |
| `unit_payments` | passing on the recorded devnet |
| `unit_lifecycle` | passing; dispute gate, arbitrate gate and `resolveDispute` |
| `unit_claims` | passing; four claim classes across five kinds (20/30 cells) |
| `unit_pricing` | passing; per-unit price stored and validated |
| `unit_guestbook` | passing; stable guest ledger |
| `unit_booking` | passing; end-to-end rent, windows and Bond total |
| Devnet bootstrap | `make devnet-up`, `make rcash-bootstrap`, `make install` |

## What is a design proposal

The [space-time primitives](../concepts/space-time-primitives.md), bilateral bonds, provider
bonds, obligation objects, generalized claims and evidence anchors are design proposals in
`spec/space-time.md`, `spec/pareto.md`, `spec/remedy-table.md` and `spec/lifecycle.md`. They
are not implemented. The Pareto scorecard records the current core as measured and the
targets as estimates.

## What is planned

| Milestone | Scope | Suite |
|---|---|---|
| M2 | provider registration, listing creation, units and slots | `unit_provider`, `unit_listing` |
| M3 | booking, escrow, same-block races, multi-slot atomicity | `unit_booking`, `unit_concurrency` |
| M4 | accept/reject/cancel/checkout/settle/deposit/dispute | `unit_lifecycle` |
| M5 | event seats, redeem, transfer, close | `unit_events` |
| M6 | 2-of-3 pause/delist/arbitration/rotation | `unit_admin` |
| M7 | bike, car and robot fixtures on the unchanged path | `unit_generality` |
| M8 | property tests, soak, invariant checks | `unit_all` |

The acceptance map is in `spec/acceptance.yaml`. The suite placeholders print their milestone
and exit 2; they never report a pass.

## What is out of scope

- a UI, backend server, indexer or off-chain database;
- KYC, fiat on-ramps, tax, invoicing and insurance;
- privacy or confidential bookings;
- cross-shard settlement;
- telemetry, remote stop and machine safety;
- a full governor or timelock system;
- legal or financial advice.

These belong to the fork or to the regulated product around it.

## How to verify a claim yourself

```sh
make spec-check       # build drift, budgets, architecture, captures, select, money
make parse-check      # render install terms and parse with the target node parser
make docs-check       # book links, orphans, coverage and build
```

The behavioural suites need the devnet path:

```sh
make devnet rcash-install keys install
make unit-root unit-payments
```

If a suite cannot run, it must be reported as not run, not as passing.

## The honesty rules

1. A status label appears near the top of every concept and case-study chapter.
2. "Implemented" never implies verification.
3. Mapping chapters say "mapping only" and name the planned milestone.
4. No chapter claims a run that is not recorded in the repository.
5. Numbers about tests, contracts or keys are measured, not remembered.

> **Where to go deeper.** [Test your fork](../guides/test-your-fork.md) is the practical
> gate. [Roadmap and acceptance milestones](../appendix/roadmap.md) maps M0–M8. [Invariants](../reference/invariants.md)
> is the list every suite checks.
