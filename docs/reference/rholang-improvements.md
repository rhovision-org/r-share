# Rholang improvements

> **Status:** P0/P1 and the addressed P2 items are implemented in the v2 deployment and
> covered by the remediation suites. The remaining P2/P3 items are a backlog. Every item is
> linked to a finding and hazard in `audit/registers/improvements.yaml`.

The ordering is by risk reduction per unit of change. Breaking changes are allowed because
the repository is devnet-only, but each one must ship with versioning, a compatibility
window and a rollback.

## P0 — safety and funds

| ID | Change | Finding | Status |
|---|---|---|---|
| R-001 | Restrict `reset` to free/expired-held slots; release slots on terminal settlement | F-001/F-021 | implemented |
| R-002 | Repost every consumed state cell on every normal branch; add a cell-fullness suite | F-002 | implemented |
| R-003 | On every normal refusal after `GuestLedgerReserve`, release slots and drop the op | F-003 | implemented |
| R-004 | No-show contest window: claim, guest dispute, default refund | F-004 | implemented |
| R-005 | Require `now >= checkOutHeight` for `settle` from `redeemed` | F-005 | implemented |
| R-006 | Reject duplicate admin keys; enforce threshold >= 2 on all sets | F-006 | implemented |
| R-022 | Release slots in `bcSettleWith` before the Bond pays | F-021 | implemented |

## P1 — liveness, integrity and operator safety

| ID | Change | Finding | Status |
|---|---|---|---|
| R-007 | Make `Payments.issuer` consult the asset allowlist, or remove the inert list | F-008 | implemented |
| R-008 | Authenticate `BookingContractFactory.create` with listing `verifyHold` and info match | F-009 | implemented |
| R-009 | Gate direct acquire on global pause and provider status | F-007 | implemented |
| R-010 | Reject or define refund/transfer cutoffs outside the slot window | F-013 | implemented |
| R-011 | Decide whether `delisted` is terminal; `resume` must refuse it if so | F-014 | implemented |
| R-012 | Decide whether `Terms.pricePerSlot` binds unit prices | F-015 | open |
| R-013 | Map Bond refusals to booking refusals where possible; document `1/0` | F-016 | implemented |
| R-014 | Keep the provider index and listing status consistent | F-012 | implemented |
| R-015 | Correct the take/await/put doctrine and Lean tie | F-011/F-019 | implemented |

## P2 — hardening

| ID | Change | Finding | Status |
|---|---|---|---|
| R-016 | Timelock/two-step admin rotation; protect `component:root`; version monotonicity | F-006 | open |
| R-017 | Review read projections for token/capability leakage | HZ-035 | open |
| R-018 | Add `schemaVersion`/`abiVersion`, v1/v2 coexistence and migration | HZ-022 | open |
| R-019 | Align the Phase 2 provider bond and delivery breach with no-show | F-004 | open |
| R-020 | Correct terminality, reset, pause, invariant 7, dead code and inventory claims | F-010/F-017/F-018/F-019 | implemented |
| R-021 | Expand behavioural and fault coverage | F-020 | open |

## Migration rule

Freeze `audit-baseline-<commit>` as v1. Add `schemaVersion` to every stateful cell and
`abiVersion` to every `info`. Deploy v2 under new component versions/keys, run v1 and v2 in
parallel, run conformance and soak, switch the Root pointer, then deprecate v1 after a full
devnet cycle. Roll back by Root pointer before state mutation; after mutation use a reverse
migration or a documented snapshot restore.

The full plan is `audit/improvement/rholang-plan.yaml`; migration details are in
`audit/improvement/migration.md`.
