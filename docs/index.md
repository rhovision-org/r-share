# r-share

**Booking and rental primitives for rholo, settled through r-cash.**

r-share is a small, auditable layer for one idea: a rental is **capacity over an interval**.
A provider publishes a listing. The listing owns units. Each unit owns slots. A guest books
one or more slots, the payment is held in escrow, and settlement releases rent, fees and the
deposit according to a policy the provider chose in advance.

That model is deliberately boring, because boring is what lets it cover so much ground:

| Stage | What it proves |
|---|---|
| Short-term holiday letting | a full stay: nights, deposit, cancellation, damage, dispute |
| Events management | capacity across show times and sections |
| Ticketing | a transferable, redeemable proof of admission |
| Municipal bike share | a fleet of units at stations with hour windows |
| Car rental | vehicle classes, windows, inspection and damage deposit |
| Robotics-as-a-service | machines with operating windows, operators, SLAs and sessions |

The north star is robotics-as-a-service. The rungs below it are not distractions: each one
exercises the same listing → unit → slot → booking path under a different real-world
constraint. If the primitives survive bike share and car rental, they are ready for a
machine that can hurt someone.

## What this book is for

This is a **community case study**, not a product manual. Fork the repository, copy the
contracts and harness, change the `kind`, the `attrs`, and a pure policy branch, and build
your own letting, events, ticketing, fleet, or RaaS business on top. The book explains both
the parts you can copy and the parts you must own yourself: identity, calendars, telemetry,
support, insurance, and regulation.

## Status at a glance

r-share is a devnet-only, unaudited prototype pinned to r-cash v0.5 at commit `49b441d`.
Do not put real value on it.

| Area | Status |
|---|---|
| 13 install terms build, measure, and parse with the target node parser | Implemented |
| Static gates (`make spec-check`) | Passing |
| 16 behavioural suites | Passing on a fresh devnet; see [Verification](reference/verification.md) |
| Allocator cornerstone | `make formal-check`; Lean model, vectors and flow manifest (`proved-model`) |
| Full holiday-letting path | Contract path exercised by booking, lifecycle and conservation |
| Events/ticketing, bike, car, robotics | Kind-level suites pass; fleet/SLA behaviour is mapping only |

"Implemented" in this book means the source exists and passes the static and parser gates.
It does not mean the behaviour has been verified on a running chain. See
[Status, scope and honest limits](orientation/status-and-scope.md) for the labels used
throughout the book.

## Where the code lives

| Path | Contents |
|---|---|
| `contracts/src/deploy/` | the 13 deployable Rholang terms |
| `contracts/src/lib/` | fragments inlined at build time (`Auth`, `Guard`, `Terms`, `Money`) |
| `contracts/build/` | generated, committed, drift-checked deploy terms |
| `harness/` | installer, devnet driver, fixtures, behavioural suites |
| `tools/` | build inliner, size and concurrency lints, node-parser check, docs check |
| `spec/` | invariants and the milestone acceptance map |
| `docs/` | this book's source; `docs/book/` is generated |

## How to read this book

- **New to the idea?** Read [The north star](orientation/north-star.md) and
  [The use-case ladder](orientation/use-case-ladder.md).
- **Building a letting product?** Read the
  [holiday-letting walkthrough](case-studies/holiday-letting/index.md) end to end.
- **Exploring another use case?** Start with
  [Events and ticketing](case-studies/events-and-ticketing.md),
  [Bike share](case-studies/bike-share.md), [Car rental](case-studies/car-rental.md), or
  [Robotics-as-a-service](case-studies/robotics-raas.md).
- **Forking the repo?** Read [Fork and run](guides/fork-and-run.md) and
  [Adding a new resource class](case-studies/new-resource-class.md).
- **Designing the core primitives?** Read [Space-time primitives](concepts/space-time-primitives.md),
  [Obligations and bonds](concepts/obligations-and-bonds.md), and
  [Remedies and claims](concepts/remedies-and-claims.md).
- **Checking a fact?** Use the [reference](reference/contract-map.md) part; it is the
  canonical home for arms, schemas, constraints and install order.

## Quickstart

The book and the static gates need no chain:

```sh
make venv
make spec-check
make parse-check      # needs ../RNodeRust, the rholo node checkout
make docs
make docs-check
```

The optional devnet path needs Docker and sibling checkouts of r-cash and the node:

```sh
make devnet-up rcash-keys rcash-bootstrap keys install
make unit-all
```

See [Fork and run](guides/fork-and-run.md) for the full setup and the current limits.

## How this book is built

`docs/` is the mdBook source and `docs/book/` is generated. The book uses stock mdBook only:
no mermaid, no KaTeX, no per-page table-of-contents plugin. Diagrams are ASCII so they
survive every renderer and every fork. `make docs-check` runs a stdlib link, orphan and
reference-coverage checker before building.

> **Where to go deeper.** [Why primitives](orientation/why-primitives.md) explains the
> design stance. [Contract map](reference/contract-map.md) is the fastest route from a
> chapter to the source. [Provenance and license](appendix/provenance-and-license.md)
> covers the AGPL obligations of running a fork.
