# Test your fork

> **Status:** Implemented for the static gates; the behavioural campaign is partial.

Testing has three layers: static gates that need no chain, a parser gate that uses the
target node's own parser, and behavioural suites that need a healthy r-cash devnet. A fork
should keep all three.

## 1. Static gates

```sh
make venv
make spec-check
```

`make spec-check` runs:

- build drift: `contracts/build/` matches `contracts/src/`;
- sizes and depth against the 56 KiB hard budget and the 64 KiB node cap;
- the component catalogue: unique state cells, known dependencies, acyclic graph;
- capture lint: no bundle over a registry-resolved name;
- no `select`;
- no unknown placeholders;
- integer money only.

A fork that changes a contract must run `make build` and commit the regenerated
`contracts/build/` and `contracts/measure.json`.

## 2. Parser gate

```sh
make parse-check
```

This renders the install terms with placeholder admins, an r-cash root and a platform owner,
then parses every built and rendered term with `rchain_rholang::parser::parse` from
`RCHAIN_RUST_DIR` (default `../RNodeRust`). The node parser enforces a depth limit of 128
and rejects unary `!`, bare-variable sends (`state!s` must be `state!(s)`), and `select`-like
constructs. Render output is in `.devnet/rendered/`.

## 3. Book gate

```sh
make docs-check
```

`tools/check_docs.py` checks SUMMARY coverage, relative links, includes, README links,
non-empty chapters, contract/reference coverage and local-path hygiene, then builds the
book. No plugins.

## 4. Behavioural suites

```sh
make devnet-up rcash-keys rcash-bootstrap keys install
make unit-all
```

| Suite | Status | Covers |
|---|---|---|
| `unit_root` | passing | resolve, keys, unknown/stale |
| `unit_payments` | passing | r-cash health, platform owner, REV issuer |
| `unit_provider` | passing | registration, ownership refusals |
| `unit_listing` | passing | terms, units, schedule, pause/delist |
| `unit_booking` | passing | instant/request, escrow, opId, per-unit rent/windows |
| `unit_allocator` | passing | atomic multi-slot acquire, token gates, pause, expiry, privacy |
| `unit_lifecycle` | passing | accept/reject/cancel/checkout/settle/dispute |
| `unit_claims` | passing | four claim classes across five kinds |
| `unit_pricing` | passing | per-unit price stored and validated |
| `unit_pause` | passing | global, provider and unit pause |
| `unit_admin` | passing | 2-of-3 pause and `setAdmins` on three surfaces |
| `unit_events` | passing | redeem, transfer, close |
| `unit_generality` | passing | five-kind whitelist |
| `unit_behavior` | passing | golden lifecycle transcript |
| `unit_conservation` | passing | seven settlement scenarios, closed-set sums |
| `unit_guestbook` | passing | stable guest ledger |

`make unit-all` runs all 16. An unhealthy devnet reports `not run`, never a pass.

## 5. The race gate

Races are not tested here; they are removed by the representation. The listing allocator
writes every requested slot in one commit, and `spec/lean/RShare/Model/Allocator.lean`
proves single ownership and all-or-none acquisition. `unit_allocator` checks the deployed
transition table: one held slot refuses the whole set, a wrong token moves nothing, and an
expired hold can be reclaimed. Two same-slot books still exercise the refusal path, but the
safety claim rests on the proof, not on a same-block schedule.

## 6. Conservation

After every suite, check:

- for each asset, `Σ account balances + Σ escrow purses` is constant;
- every slot is exactly one of free, held, booked;
- every terminal booking's buckets sum to `rent + deposit`;
- no slot cell is empty at a quiescent point.

These are in [Invariants](../reference/invariants.md).

## 7. Adding a suite for a new class

- Add `harness/scenarios/<class>_book.rho` and a fixture builder.
- Add `harness/unit_generality.py` coverage for book → accept → check-out → settle.
- Add allocator conformance cases and the conservation assertions.
- Add a rule to `spec/flows/core.yaml` or mark the new arm `owed`; a core arm must name a
  Lean declaration.
- Add the suite to `run_suites.py`, the Makefile and `check_suites`, then update this table
  and [Status, scope and honest limits](../orientation/status-and-scope.md).

## 8. What not to claim

- "Tested" without a recorded command and artifact.
- "Audited" without an audit report.
- "Production-ready" while the repo is devnet-only and unaudited.
- "Passing" without a recorded artifact; a `not run` suite is never a pass.

> **Where to go deeper.** [Platform constraints](../reference/platform-constraints.md) is
> the limit list. [Invariants](../reference/invariants.md) is the property list. [Fork and
> run](fork-and-run.md) is the setup.
