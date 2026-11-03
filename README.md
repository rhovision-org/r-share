# r-share

[Read the book](docs/index.md) · [Book contents](docs/SUMMARY.md)

On-chain booking and rental primitives for the rholo chain, settled through
[r-cash](https://github.com/rhovision-org/r-cash). A community case study: fork it, copy the
patterns, and build your own letting, events, ticketing, bike, car, or robotics business.
The north star is robotics-as-a-service.

r-share models a rental as **capacity over an interval**: a provider publishes a listing, a
listing owns units, and each unit owns slots. Booking, escrow, cancellation, deposits,
disputes and tickets are shared. The chain is the backend; there is no server, UI, or
off-chain database in this repo.

## The ladder

| Stage | What it proves | Status |
|---|---|---|
| Short-term holiday letting | full stay lifecycle, deposit, cancellation, dispute | Lifecycle and claim suites pass; slot settlement not exercised |
| Events management | capacity across show times and sections | Claim path passes; event-only ticket arms not exercised |
| Ticketing | transferable, redeemable proof of admission | Claim path passes; event-only ticket arms not exercised |
| Municipal bike share | fleet, station-hour capacity, unlock credential | Claim path passes for the kind; fleet/station behavior not exercised |
| Car rental | vehicle windows, inspection, damage deposit | Claim path passes for the kind; inspection not exercised |
| Robotics-as-a-service | machine windows, operators, SLAs, session evidence | Claim path passes for the kind; per-unit pricing/SLAs not implemented |

## What is here

| Path | Contents |
|---|---|
| `contracts/src/deploy/` | 13 deployable Rholang terms (root, terms/remedy, capacity, bond, booking, payments) |
| `contracts/src/lib/` | build-time fragments (`Auth` inline; `Guard` only for the bootstrap Root; `Money` into Remedy) |
| `harness/` | installer, devnet driver, fixtures, behavioural suites |
| `tools/` | build inliner, size/concurrency lints, node-parser check, docs check |
| `spec/` | invariants and the milestone acceptance map |
| `docs/` | the mdBook source; `docs/book/` is generated and ignored |
| `book.toml` | mdBook configuration |

## Status

Devnet-only, unaudited, and pinned to r-cash v0.5 at commit `49b441d`. Do not put real value
on it.

The 13 install terms build, measure, and parse with the target node's own parser;
`make spec-check` passes. On 2026-11-03 a fresh devnet built from rholo `e2d073d` with
r-cash `49b441d` recorded passing runs for `unit_root`, `unit_payments` and
`unit_lifecycle`, `unit_claims`, `unit_pricing` and `unit_guestbook`. The lifecycle suite
verifies the dispute gate, the arbitration gate, permissionless `resolveDispute` and the
readable evidence anchors; the claims suite verifies
`buyerUseBreach`, `noFaultCancellation`, `noShow` and `damage` for lodging, events, bike,
car and robot. `unit_pricing` verifies that a unit stores and validates its own price.
Coverage is 20/30 `passing`; `providerDeliveryBreach` and `partialPerformance` remain
design. End-to-end rent aggregation across differently priced units is not yet tested. The remaining suites are placeholders that exit 2. See
[Status, scope and honest limits](docs/orientation/status-and-scope.md).

## Quickstart

```sh
make venv
make spec-check
make parse-check      # needs ../RNodeRust, the rholo node checkout
make docs
make docs-check
```

Optional devnet path (Docker plus sibling r-cash and node checkouts):

```sh
make devnet-up keys rcash-bootstrap install
make unit-all
```

See [Fork and run](docs/guides/fork-and-run.md) for the full setup.

## Read the book

- [Introduction](docs/index.md)
- [The north star](docs/orientation/north-star.md)
- [The use-case ladder](docs/orientation/use-case-ladder.md)
- [Holiday letting walkthrough](docs/case-studies/holiday-letting/index.md)
- [Events and ticketing](docs/case-studies/events-and-ticketing.md)
- [Fork and run](docs/guides/fork-and-run.md)
- [Contract map](docs/reference/contract-map.md)

## Fork it and build your own

Fork the repo, keep the AGPL notices, and copy `contracts/`, `harness/` and `tools/`. Change
the `kind`, the `attrs`, the slot IDs, the terms, and a pure `Remedy` branch. Do not add a
booking arm; if you think you need one, read
[Adding a new resource class](docs/case-studies/new-resource-class.md) first. Name your
product distinctly. You own the off-chain parts: UI, identity, calendar, telemetry, support,
insurance, and regulation.

## Documentation

`docs/` is the mdBook source; `docs/book/` is generated. Use `make docs`, `make docs-serve`,
and `make docs-check` (mdBook 0.5.4+). CI builds and checks the book; there is no GitHub
Pages deployment.

## License

AGPL-3.0-or-later. See `LICENSE` and `THIRD_PARTY_NOTICES.md`. Running a modified version as
a network service carries the AGPL §13 source-offer obligation. No real funds; not legal or
financial advice.
