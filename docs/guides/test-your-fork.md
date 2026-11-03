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
make devnet rcash-install keys install
make unit-all
```

| Suite | Status | Covers |
|---|---|---|
| `unit_root` | implemented | resolve, keys, unknown/stale |
| `unit_payments` | implemented | r-cash health, platform owner, REV issuer |
| `unit_provider` | planned (M2) | registration, ownership refusals |
| `unit_listing` | planned (M2) | terms, units, slot schedule |
| `unit_booking` | planned (M3) | instant/request, escrow, opId |
| `unit_allocator` | planned (M3) | same-block races |
| `unit_lifecycle` | planned (M4) | accept/reject/cancel/checkout/settle |
| `unit_events` | planned (M5) | redeem, transfer, close |
| `unit_admin` | planned (M6) | 2-of-3 pause/delist/arbitrate |
| `unit_generality` | planned (M7) | bike/car/robot fixtures |

Planned suites exit 2 with the milestone they belong to. They are not reported as passing.
If a suite cannot run, report it as not run.

## 5. The race gate

Use two deploys submitted before a single propose, so both land in the same block:

1. **One slot, two guests** → exactly one `(true, ...)`, one `(false, "slot held", id)`,
   one debit.
2. **Disjoint slots** → both succeed, two debits.
3. **Overlapping multi-slot** → A books [1,2], B books [2,3]; exactly one wins; the loser
   holds none of its slots.
4. **Accept vs cancel** → exactly one terminal transition; the other returns `"closed"`;
   funds match the winner.
5. **Expiry race** → A's hold expires, B books the slot, A.expire refunds A and A.release
   returns `"not holder"`; B's slot is intact.

`harness/deploy.py` has `deploy_same_block` for this. A race test that only runs one deploy
at a time proves nothing.

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
- Add the race tests and the conservation assertions.
- Add the suite to the Makefile and `unit-all`.
- Update this table and [Status, scope and honest limits](../orientation/status-and-scope.md).

## 8. What not to claim

- "Tested" without a recorded command and artifact.
- "Audited" without an audit report.
- "Production-ready" while the repo is devnet-only and unaudited.
- "Passing" for a placeholder suite.

> **Where to go deeper.** [Platform constraints](../reference/platform-constraints.md) is
> the limit list. [Invariants](../reference/invariants.md) is the property list. [Fork and
> run](fork-and-run.md) is the setup.
