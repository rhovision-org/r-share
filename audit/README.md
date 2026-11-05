# r-share audit and HAZOP

> **Status:** Baseline desk audit at commit `dee7cff`, with five dynamic reproductions
> (F-001, F-004, F-005, F-006, F-007) on an isolated devnet. The remaining findings are
> static or suspected. This is not a production audit and does not certify the contracts,
> rholo, r-cash or any deployment.

This directory is the working audit record for the r-share smart-contract primitives, the
Lean model and the Rholang improvement plan. It is deliberately separate from `docs/`: the
registers are the editable source of truth, and the book chapters summarise them.

## Start here

| Artifact | Purpose |
|---|---|
| [charter.md](charter.md) | Scope, roles, evidence standard and exit criteria |
| [method.md](method.md) | Audit and HAZOP method, risk scale and evidence levels |
| [registers/findings.yaml](registers/findings.yaml) | Verified or suspected findings |
| [registers/hazards.yaml](registers/hazards.yaml) | HAZOP deviations, causes and dispositions |
| [registers/theorems.yaml](registers/theorems.yaml) | Lean theorem/property registry |
| [registers/improvements.yaml](registers/improvements.yaml) | Prioritised Rholang changes |
| [registers/tests.yaml](registers/tests.yaml) | Planned and implemented test scenarios |
| [hazop/nodes.yaml](hazop/nodes.yaml) | The N1–N20 HAZOP node list |
| [hazop/worksheet.md](hazop/worksheet.md) | Readable HAZOP summary |
| [lean/rigour-ladder.md](lean/rigour-ladder.md) | L0–L4 proof ladder and status rules |
| [improvement/rholang-plan.yaml](improvement/rholang-plan.yaml) | Canonical improvement backlog |
| [improvement/migration.md](improvement/migration.md) | Versioning, coexistence and rollback |
| [traceability/matrix.md](traceability/matrix.md) | Generated arm/hazard/finding/theorem/change matrix |
| [report.md](report.md) | Current verdict, counts and residual risk |

## Current snapshot

At the remediation commit the inventory is 13 deploy terms, 21 components, 10 state-cell
types and 119 public arm declarations. The Lean project has 72 theorems, all named in the
axiom audit; the flow manifest covers 31 arms with 35 rules and the strict gate reports zero
warnings. The P0/P1 findings and the addressed P2 fixes are implemented in v2 and exercised
by the 24 recorded suites; the three added P2 suites await acceptance, and the remaining
P2/P3 items stay open.

The original seed findings were:

- `reset` frees booked or held slots with owner authority only (F-001);
- `GuestBook.getOrCreate` can consume `guestBookState` and fail to repost it (F-002);
- `GuestLedgerPay` leaves a permanent pending op when the Bond factory is unavailable
  (F-003);
- a provider can capture rent by never checking in and claiming no-show (F-004);
- `settle` from `redeemed` does not require the event to be over (F-005);
- duplicate admin keys can make a nominal 2-of-3 into 1-of-3 (F-006);
- the flow manifest marks definitions and constructors as `proved-model` (F-019).

The original reproductions are in `evidence/2026-11-04-dee7cff/`. The remediation campaign
is a separate record under `evidence/`; the same isolated v2 devnet passed all 24 suites,
including cell liveness and factory auth. v1 instances keep v1 semantics; only v2 is
claimed remediated. P2/P3 remain a backlog.

## Publication

The repository is public and AGPL-3.0-or-later. The full findings, HAZOP rows and
reproduction steps are published. Only credentials are removed from evidence: private
keys, passwords, keystore ciphertext and node tokens. If a production or third-party
deployment appears, revisit this policy before publishing new findings.

## Reproduce

```sh
make check          # static, parser, docs and formal gates
make audit-check    # register schemas, arm inventory and traceability
make unit-all       # behavioural suites; needs a fresh devnet
make audit-dynamic  # fault, boundary, adversarial and cell-liveness suites
```

Every recorded run is stored under `audit/evidence/<date>-<commit>/` with a manifest that
names the commit, toolchain, node pins and artifact hashes.
