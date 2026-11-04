# r-share

[Read the book](docs/index.md) · [Book contents](docs/SUMMARY.md)

On-chain booking and rental primitives for the rholo chain, settled through
[r-cash](https://github.com/rhovision-org/r-cash). A community case study: fork it, copy the
patterns, and build your own letting, events, ticketing, bike, car, or robotics business.
The north star is robotics-as-a-service.

r-share models a rental as **capacity over an interval**: a provider publishes a listing, a
listing owns units, and each unit owns slots. Booking, escrow, cancellation, deposits,
disputes and tickets are shared. The chain is the backend. A Rust SDK, a CLI, a WASM wallet
and a React reference client live in this repo; there is no backend server, indexer or
off-chain database.

## The ladder

| Stage | What it proves | Status |
|---|---|---|
| Short-term holiday letting | full stay lifecycle, deposit, cancellation, dispute | Booking, lifecycle, claim and conservation suites pass; slot settlement exercised |
| Events management | capacity across show times and sections | Claim suite and unit_events ticket arms pass; section capacity is mapping only |
| Ticketing | transferable, redeemable proof of admission | unit_events redeem, transfer and close pass |
| Municipal bike share | fleet, station-hour capacity, unlock credential | unit_generality passes for the kind; fleet/station capacity is mapping only |
| Car rental | vehicle windows, inspection, damage deposit | unit_generality passes for the kind; inspection and insurance are mapping only |
| Robotics-as-a-service | machine windows, operators, SLAs, session evidence | Booking, allocator, per-unit pricing/windows and claims pass; SLAs and evidence enforcement are design |

## What is here

| Path | Contents |
|---|---|
| `contracts/src/deploy/` | 13 deployable Rholang terms (root, terms/remedy, capacity, bond, booking, payments) |
| `contracts/src/lib/` | build-time fragments (`Auth` inline; `Guard` only for the bootstrap Root; `Money` into Remedy) |
| `harness/` | installer, devnet driver, fixtures, behavioural suites |
| `crates/` | `rshare-core`, `rshare-sdk`, `rshare-keystore`, `rshare-cli`, `rshare-wasm` |
| `web/` | React + Vite + TypeScript reference client |
| `tools/` | build inliner, size/allocator/flow lints, node-parser check, docs check |
| `spec/` | invariants and the milestone acceptance map |
| `docs/` | the mdBook source; `docs/book/` is generated and ignored |
| `book.toml` | mdBook configuration |

## Status

Devnet-only, unaudited, and pinned to r-cash v0.5 at commit `49b441d`. Do not put real value
on it.

The 13 install terms build, measure, and parse with the target node's own parser;
`make spec-check` passes. On 2026-11-03 a fresh devnet built from rholo `e2d073d` with
r-cash `49b441d` recorded 16 passing suites: `unit_root`, `unit_payments`,
`unit_lifecycle`, `unit_claims`, `unit_pricing`, `unit_guestbook`, `unit_booking`,
`unit_pause`, `unit_allocator`, `unit_admin`, `unit_events`, `unit_generality`,
`unit_provider`, `unit_listing`, `unit_behavior` and `unit_conservation`. The lifecycle
suite verifies the dispute gate, the arbitration gate, permissionless `resolveDispute` and
the readable evidence anchors; the claims suite verifies
`buyerUseBreach`, `noFaultCancellation`, `noShow` and `damage` for lodging, events, bike,
car and robot. `unit_pricing` verifies that a unit stores and validates its own price, and
`unit_booking` proves an end-to-end booking of a machine and operator at different prices:
rent is the sum, the windows are the per-unit min/max, the Bond total is rent + deposit, and
a cancel refunds the escrow. `unit_conservation` runs seven real settlement scenarios
(cancel, reject, expire, settle with fee, no-show, deposit split and replay) over a closed
set of guest, provider, platform and Bond balances, with a negative control that must
detect untracked escrow. `unit_pause` verifies global, provider and unit pause;
`unit_allocator` verifies atomic multi-slot acquisition, token-only confirm/release, pause
gates, expiry takeover and the absence of a slot-capability leak. The admin, event and
generality suites cover signed refusals, the event-only arms and the five-kind whitelist;
provider and listing suites cover their full arms; a normalized lifecycle transcript is
compared against a committed golden file. Coverage is 20/30 `passing`;
`providerDeliveryBreach` and `partialPerformance` remain design.

A reference client now exercises the same surface from Rust, the CLI and a browser. On
2026-11-03/04 the CLI devnet journey passed: quote, book, held-slot refusal, replay,
wrong-terms refusal, manual accept/cancel and a one-signature refusal plus a two-signature
global pause/resume. The browser campaign passed the guest, provider and admin-multisig
journeys against the same devnet. The client uses a curated catalogue and share links; it
has no indexer, provider inbox, search or fiat ramp. See
[Client architecture](docs/reference/client-architecture.md).

The allocator cornerstone is also modelled in Lean: `make formal-check` proves the
single-owner, atomic-acquisition, bounded-hold, token-gate and conservation obligations over
the model, and `spec/flows/core.yaml` names the theorem for every core arm. The result is
`proved-model`; it is not a refinement of Rholang. See
[Status, scope and honest limits](docs/orientation/status-and-scope.md).

## Quickstart

```sh
make venv
make spec-check
make formal-check     # Lean model, vectors and flow manifest
make parse-check      # needs ../RNodeRust, the rholo node checkout
make docs
make docs-check
make client-check     # Rust SDK/CLI, WASM smoke, web unit tests and build
```

Optional devnet path (Docker plus sibling r-cash and node checkouts):

```sh
make devnet-up rcash-keys rcash-bootstrap keys install
make unit-all
make client-fixtures client-journey
cd web && npm install && npm run dev     # http://localhost:5173
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

Fork the repo, keep the AGPL notices, and copy `contracts/`, `harness/`, `tools/`, `crates/`
and `web/`. Change
the `kind`, the `attrs`, the slot IDs, the terms, and a pure `Remedy` branch. Do not add a
booking arm; if you think you need one, read
[Adding a new resource class](docs/case-studies/new-resource-class.md) first. Name your
product distinctly. You own the off-chain parts: identity, calendar, telemetry, support,
insurance, and regulation. The included client is a reference, not a hosted marketplace.

## Documentation

`docs/` is the mdBook source; `docs/book/` is generated. Use `make docs`, `make docs-serve`,
and `make docs-check` (mdBook 0.5.4+). CI builds and checks the book; there is no GitHub
Pages deployment.

## License

AGPL-3.0-or-later. See `LICENSE` and `THIRD_PARTY_NOTICES.md`. Running a modified version as
a network service carries the AGPL §13 source-offer obligation. No real funds; not legal or
financial advice.
