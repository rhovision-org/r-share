# Roadmap and acceptance milestones

> **Status:** Roadmap. Planned work is not implemented.

The roadmap is an acceptance map, not a promise. Each milestone names the suite that must
pass before the milestone is considered done. The machine-readable version is
`spec/acceptance.yaml`; the implemented suites are under `harness/`.

## Milestones

| Milestone | Scope | Acceptance |
|---|---|---|
| M0 | repo bootstrap, build inliner, static gates, book scaffold | `make spec-check`, `make docs-check` |
| M1 | kernel, Root, admin, Payments adapter | `unit_root`, `unit_payments` |
| M2 | provider, listing, unit and slot provisioning | `unit_provider`, `unit_listing` |
| M3 | booking creation, escrow, same-block races, multi-slot atomicity | `unit_booking`, `unit_concurrency` |
| M4 | lifecycle, cancellation, checkout, settlement, deposits, disputes | `unit_lifecycle` |
| M5 | events and tickets | `unit_events` |
| M6 | 2-of-3 pause, delist, arbitration, rotation | `unit_admin` |
| M7 | bike, car and robot fixtures on the unchanged path | `unit_generality` |
| M8 | property tests, soak, invariant checks, docs complete | `unit_all`, all invariants |

## Status

- M0: static and parser gates pass; the book builds.
- M1: contracts implemented; the two suites exist but have not run on a healthy devnet.
- M2–M7: contracts implemented; the suites are placeholders that exit 2.
- M8: not started.

"Implemented" means source plus static/parser gates. It does not mean a behavioural run.

## Universal-core roadmap

The [space-time proposal](https://github.com/PatrickMockridge/r-share/blob/main/spec/space-time.md)
adds a second, design-first track after the primitive milestones:

| Phase | Adds | Must not add |
|---|---|---|
| 1 | modular base layer, Terms/Remedy/Bond, pause/contiguous | claim classes, provider bond, oracle |
| 2 | Terms v2, obligation objects, remedy table, generalized claim | oracle, second writer, arbitrary policy |
| 3 | per-booking evidence anchors | fact adjudication, global append-only hot cell |
| 4 | provider bond and cure window | shared bond pool on the hot path, timers |
| 5 | bounded metering with buyer challenge | on-chain oracle, per-block billing |

Each phase must remain non-dominated on `spec/pareto.md`; a phase that adds a trust party or
a global bottleneck is rejected or moved to a fork.

## What would change the roadmap

- A new resource class that needs a new primitive.
- A cross-shard settlement requirement.
- A regulated product that needs an on-chain identity or insurance layer.
- A RaaS metering requirement that per-slot pricing cannot express.
- A security finding that changes the escrow or authority model.

Those are design discussions, not silent scope creep.

## How to propose a milestone

Open an issue with the use case, the primitive it stresses, the acceptance tests and the
off-chain boundary. If it needs a new booking arm, say why the existing arms cannot express
it. The community decides whether it belongs in r-share or in a fork.

> **Where to go deeper.** [Status, scope and honest limits](../orientation/status-and-scope.md)
> is the label vocabulary. [Test your fork](../guides/test-your-fork.md) is the gate. [The
> use-case ladder](../orientation/use-case-ladder.md) is the progression.
