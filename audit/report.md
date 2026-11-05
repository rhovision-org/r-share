# r-share audit report

> **Status:** Baseline desk audit at `dee7cff`, followed by a P0/P1 and selected P2
> remediation campaign on an isolated v2 devnet. The addressed findings are implemented and
> covered by a recorded 24-suite run; the 27-suite acceptance run is pending. Remaining
> P2/P3 items stay open. v1 instances keep v1 semantics and
> are not claimed fixed.

## Verdict

The primitive set is not audited in the production sense. The current evidence supports a
narrow claim: selected happy and refusal paths work on one recorded devnet, and a small Lean
model proves local properties about a hand-written state machine. It does not support
"formally verified", "refinement proved", "no vulnerabilities" or "all arms covered".

The audit found a cluster of high-impact issues in the allocator, guest ledger, booking
fairness, event timing, admin threshold and formal-claim layers. The most serious are
straightforward to state and should be fixed before any wider use:

| Finding | Severity | Issue | Proposed change |
|---|---|---|---|
| F-001 | P0 | `reset` frees booked/held slots with owner authority only | R-001 |
| F-002 | P0 | `GuestBook.getOrCreate` can consume `guestBookState` without reposting | R-002 |
| F-003 | P0 | Bond-factory-unavailable leaves a pending op and held slots | R-003 |
| F-004 | P0 | Provider can capture rent by never checking in and claiming no-show | R-004 |
| F-006 | P0 | Duplicate admin keys can reduce a nominal 2-of-3 to 1-of-3 | R-006 |
| F-005 | P1 | `settle` from `redeemed` does not require the event to be over | R-005 |
| F-007 | P1 | Global pause does not stop direct `acquire` | R-009 |
| F-009 | P1 | `BookingContractFactory` accepts arbitrary booking data | R-008 |
| F-011 | P1 | `acquire` holds `listingState` across `UnitFactory.slotScan` | R-015 |
| F-019 | P1 | Flow manifest overclaims `proved-model` and is incomplete | R-015/R-020 |

The remaining findings in `registers/findings.yaml` cover terms/unit price binding and
behavioural coverage. The P2 asset allowlist, provider index, cutoff arithmetic, delist
terminality, Bond refusal mapping and dead-remedy findings were remediated in wave 2.

## Remediation wave 1

The P0/P1 fixes are implemented in the v2 source and exercised on an isolated devnet:

| Finding | Fix | Regression |
|---|---|---|
| F-001 | reset only free/expired-held; terminal settlement releases slots | `unit_reset_safety`, `unit_settle_release` |
| F-002/F-003 | GuestBook repost; ledger release/drop compensation | `unit_cell_liveness` |
| F-004 | no-show contest window; guest dispute; default refund | `unit_no_show_contest` |
| F-005 | event settle requires `checkOutHeight` | `unit_event_timing` |
| F-006 | distinct admin keys and threshold >= 2 | `unit_admin_keys`, `unit_admin` |
| F-007 | global/provider pause gates in acquire | `unit_pause_scope` |
| F-009 | listing `verifyHold` plus provider/listing/kind match | `unit_factory_auth` |
| F-019 | flow rules target theorems; strict gate 0 warnings | `make check-flows-strict` |
| F-021 | `bcSettleWith` releases slots before paying | `unit_settle_release` |

The recorded campaign passed 24/24 suites on the isolated v2 devnet. The tree now defines
27; the provider index, cutoff and Bond-refusal suites await the acceptance run. The remaining P2/P3 items stay open, and the v1
instances are not remediated.

## Remediation wave 2 (P2/P3)

| Finding | Fix | Regression |
|---|---|---|
| F-008 | `Payments.issuer` consults the stored asset allowlist; `setAssets` refuses empty | `unit_payments` |
| F-012 | `removeListing` delists the authoritative listing and propagates failure | `unit_provider_index` |
| F-013 | `ensureUnit` validates refund/transfer cutoffs against slot start and duration | `unit_cutoffs` |
| F-014 | `delisted` is terminal for `pause` and `resume` (owner and admin) | `unit_provider_index` |
| F-016 | Bond `"closed"` maps to `(false, "closed")`; `"not holder"` release is idempotent | `unit_bond_refusals` |
| F-017 | provider cancel routes through `Remedy.cancel`; no-show payout through `Remedy.noShow` | `unit_no_show_contest`, `conservation` |
| F-010/F-018 | docs, status vocabulary and inventory counts corrected | `make docs-check` |

Terms/unit price binding (F-015/R-012), governance timelock, privacy, migration and arm
coverage remain open.

## Baseline

| Field | Value |
|---|---|
| Commit | `dee7cff448c30bd84890af408eb2fc4d6012494c` |
| Branch | `main` |
| Node | isolated `rshare-audit` devnet at `localhost:41403`/`41405`; rholo `e2d073dcd02f1e560ccb84749937138a1e184b0b` |
| Lean | 4.12.0, Lake 5.0.0 |
| Rust | 1.95.0 |
| Node.js | 22.22.2 |
| Inventory | 13 deploy terms, 21 components, 10 state-cell types, 119 arms |
| Lean | 72 theorems in source, all 72 named in the axiom audit; strict flow gate 0 warnings |
| Flow manifest | 35 rules, 31 arms, 13 core arms, 6 `owed` |

## What this audit supports

- A full arm inventory exists and is checked against the flow manifest.
- The HAZOP nodes N1–N20 are defined and the seed hazards are recorded with causes,
  consequences, risk and dispositions.
- The source-level findings are traceable to `file:line` and proposed changes.
- The Lean status vocabulary is corrected; the gaps are explicit rather than hidden.

## What it does not support

- No claim that every arm is behaviourally tested. The 27-suite set (24 recorded) is broader than the
  original 16 and covers the remediation faults, but it is still not a full arm × refusal
  matrix; several state-machine suites use a stub Bond. F-020/R-021 remain open.
- No claim of Rholang refinement. The model is a hand-written sequential state machine;
  HZ-011 blocks a linearizability claim for the deployed `acquire`.
- No claim of node-level ERTP conservation. Conservation is model-level and behavioural
  over a closed account set.
- No production or safety certification. The audit can find defects; it cannot certify the
  node, r-cash, cryptography or off-chain safety.
- P0/P1 and the addressed P2 findings are closed only for the v2 deployment with the listed regression suites. Terms/unit price binding, governance timelock, privacy review, migration and provider bond remain open, and v1 instances are not remediated.

## Residual risk

| Risk | Status |
|---|---|
| Reset double-sale | Fixed in v2; v1 remains |
| Guest onboarding deadlock | Fixed in v2; v1 remains |
| Pending-op lockup | Fixed in v2; v1 remains |
| No-show rent capture | Fixed in v2; v1 remains |
| Admin threshold bypass | Fixed in v2; v1 remains |
| Event early settlement | Fixed in v2; v1 remains |
| Global pause bypass | Fixed in v2; v1 remains |
| Unauthenticated booking factory | Fixed in v2; v1 remains |
| Formal manifest overclaim | Fixed in v2 |
| Asset allowlist | Fixed in v2; v1 remains |
| Provider index / delist terminality | Fixed in v2; v1 remains |
| Cutoff arithmetic | Fixed in v2; v1 remains |
| Bond refusal mapping | Fixed in v2; v1 remains |
| Remedy dead code | Fixed in v2 |
| Terms/unit price binding | Open; decision required |
| Test coverage gaps | Open; R-021 |

## Dynamic evidence

The historical audit reproductions used the old `audit_*` suites on an isolated devnet
(`rshare-audit`, ports 41403/41405; the unrelated `rgen-bootstrap` devnet was not touched):

| Suite | Finding | Observed |
|---|---|---|
| `audit_reset` | F-001 | reset freed a booked slot; a second booking acquired it; the first stayed confirmed |
| `audit_no_show` | F-004 | provider claimed no-show without check-in; guest dispute refused `"closed"` |
| `audit_event` | F-005 | redeem then settle succeeded before `checkOutHeight` |
| `audit_admin_keys` | F-006 | one signature satisfied a duplicated-key threshold of 2 |
| `audit_pause` | F-007 | `GuestLedger.book` refused `"paused"`; direct `acquire` succeeded |

The historical log is in `evidence/2026-11-04-dee7cff/`. The remediation campaign is a
separate v2 run: 24/24 suites are recorded; the P2 provider-index, cutoff and Bond-refusal
suites await acceptance; the CLI journey and three browser journeys pass. See
`docs/reference/verification.md` and the remediation evidence manifest.

## Next evidence

The P2/P3 programme is in `registers/tests.yaml`. A remaining finding becomes `fixed` only
when its scenario is reproduced and then passes on a fresh devnet with evidence stored under
`evidence/<date>-<commit>/`.

## Review

The baseline has not yet had an independent review. Until it does, this document is a
self-audit.
