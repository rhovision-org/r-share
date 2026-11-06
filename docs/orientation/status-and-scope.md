# Status, scope and honest limits

> **Status:** This chapter defines the labels used everywhere else in the book.

r-share has static and parser checks plus recorded behavioural campaigns. On 2026-11-03, a
fresh devnet built from rholo `e2d073d` with r-cash `49b441d` passed every then-defined suite
in `harness/run_suites.py`: root, payments, lifecycle, claims, pricing, guestbook, booking,
pause, allocator, admin, events, generality, provider, listing, the golden transcript and
conservation. The claims run verifies `buyerUseBreach`, `noFaultCancellation`, `noShow` and
`damage` across all five kinds (20/30 coverage cells); conservation verifies seven real
settlement scenarios over a closed balance set. `providerDeliveryBreach` and
`partialPerformance` remain design; the unrun breadth items are listed in
[Verification](../reference/verification.md).

## Status labels

| Label | Meaning |
|---|---|
| **Implemented** | Source exists; `make spec-check` and `make parse-check` pass. No behavioural claim. |
| **Implemented, not yet exercised** | Source exists and the gates pass; no healthy-devnet suite run is recorded. |
| **Planned** | A design-only feature or a future suite; no placeholder runner remains. |
| **Mapping only** | No dedicated suite; the use case rides the shared contract path and is unverified. |
| **Design intent** | Roadmap or north-star direction; not implemented. |
| **Experimental demo** | Implemented source exercised only under an explicit demo profile; not default, not insurance and not production-safe. |
| **Out of scope** | Deliberately not in this repository. |

Never read "implemented" as "tested", "audited", "production-ready" or "battle-tested". Those
words are reserved for a recorded command and artifact.

## What is implemented

| Area | Evidence |
|---|---|
| 14 safe install terms (19 with the experimental cover demo) | `make build`, `make measure`, `make parse-check` |
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
| `unit_booking` | passing; end-to-end rent, windows, Bond total and one-booking conservation |
| `unit_pause` | passing; global, provider and unit pause |
| `unit_allocator` | passing; atomic multi-slot acquire, token gates, pause, expiry, privacy |
| `unit_admin` | passing; booking, listing and provider `setAdmins`, threshold and replay |
| `unit_events` | passing; redeem, closeEvent and transferTicket |
| `unit_generality` | passing; five kinds validate |
| `unit_provider` | passing; register, profile, pause/resume, owner |
| `unit_listing` | passing; create, unit, activate, pause/resume, delist |
| `unit_behavior` | passing; golden lifecycle transcript |
| `unit_conservation` | passing; seven real scenarios, closed-set sum constant, Bond closure, replay, negative control |
| Rust SDK and CLI | `make client-test`, `make client-journey` on the recorded devnet |
| WASM wallet and keystore | `npm --prefix web run test:wasm` (offline) |
| Web client | `npm --prefix web run build`; guest, provider and admin Playwright journeys |
| Devnet bootstrap | `make devnet-up`, `make rcash-bootstrap`, `make install` |

## What is a design proposal

The [space-time primitives](../concepts/space-time-primitives.md), bilateral bonds, provider
bonds, obligation objects, generalized claims and evidence anchors are design proposals in
`spec/space-time.md`, `spec/pareto.md`, `spec/remedy-table.md` and `spec/lifecycle.md`. They
are not implemented. The Pareto scorecard records the current core as measured and the
targets as estimates.

## What is planned

The M2–M7 suite milestones have landed; the recorded campaigns are in
[Verification](../reference/verification.md). What remains:

| Item | Scope | Status |
|---|---|---|
| M8 | property tests, soak and broader invariant checks | planned |
| Formal model | `make formal-check`: Lean allocator/booking/bond/remedy model, vectors and flow manifest | implemented, proved-model |
| Audit baseline | `audit/` registers; 21 findings; P0/P1 and addressed P2 fixes implemented in v2; 24 suites recorded, 27-suite acceptance pending; 72/72 Lean theorems named in the axiom audit | Remaining P2/P3 open |
| Client | Rust SDK, CLI, WASM wallet, React app; curated catalogue and share links | implemented, exercised on devnet |
| Phase 2 | provider bond leg, obligation objects, generalized claims, evidence enforcement | design |

The client is deliberately not a marketplace. It has no indexer, provider inbox, search,
notifications or fiat ramp; see [Discovery and share
links](../reference/discovery-and-share-links.md).

The acceptance map is in `spec/acceptance.yaml`. No suite placeholder remains in the
runner: an unhealthy devnet reports `not run`, never a pass.

## What is out of scope

- a backend server, indexer or off-chain database;
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
make formal-check     # Lean build, axiom audit, vectors and flow manifest
make audit-check      # audit registers, arm inventory and traceability
make parse-check      # render install terms and parse with the target node parser
make docs-check       # book links, orphans, coverage and build
make client-check     # Rust SDK/CLI, WASM smoke, web unit tests and build
```

The behavioural suites need the devnet path:

```sh
make devnet-up rcash-keys rcash-bootstrap keys install
make unit-all
make audit-dynamic
make client-fixtures client-journey
make client-keystores
cd web && npm install && npm run test:e2e
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
