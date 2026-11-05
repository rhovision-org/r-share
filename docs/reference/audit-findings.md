# Audit findings

> **Status:** Baseline desk audit at `dee7cff`, followed by a P0/P1 remediation campaign on
> an isolated v2 devnet. The P0/P1 findings are fixed in v2 and covered by green suites;
> P2/P3 remain open. Full records are in `audit/registers/findings.yaml`.

## Summary

| Severity | Count | Status |
|---|---|---|
| P0 | 5 | fixed in v2 |
| P1 | 6 | fixed in v2 (including F-021) |
| P2 | 8 | open follow-up |
| P3 | 2 | open follow-up |

## P0 — fixed in v2

| Finding | Original issue | Fix | Regression |
|---|---|---|---|
| F-001 | `reset` freed a booked/held slot | reset only free/expired-held; live/booked refused | `unit_reset_safety` |
| F-002 | GuestBook could consume `guestBookState` without reposting | both failure branches repost | `unit_cell_liveness` |
| F-003 | Bond-factory failure left a pending op and held slot | release then drop on every failure | `unit_cell_liveness` |
| F-004 | provider captured rent via `claimNoShow` without check-in | `noShowClaimed` contest window; guest dispute; default refund | `unit_no_show_contest` |
| F-006 | duplicate admin keys / threshold 1 | distinct-key validation and threshold >= 2 on all admin sets | `unit_admin_keys`, `unit_admin` |

## P1 — fixed in v2

| Finding | Original issue | Fix | Regression |
|---|---|---|---|
| F-005 | `settle` from `redeemed` before event end | require `now >= checkOutHeight` | `unit_event_timing` |
| F-007 | global/provider pause bypassed direct `acquire` | pause and provider-active gates in `acquire` | `unit_pause_scope` |
| F-009 | factory accepted arbitrary booking data | `verifyHold` plus listing info match; direct calls refused | `unit_factory_auth` |
| F-011 | acquire doctrine claimed one commit | doctrine and model now say take/await/put; no restructure | docs and `formal-check` |
| F-019 | flow manifest overclaimed `proved-model` | rules target theorems; strict gate 0 warnings | `make check-flows-strict` |
| F-021 | terminal settlement left slots booked | `bcSettleWith` releases before paying | `unit_settle_release` |
| F-008 | asset allowlist was inert | `Payments.issuer` checks the stored list; `setAssets` refuses empty | `unit_payments` |
| F-012 | `removeListing` left the listing active | it delists first and propagates failure | `unit_provider_index` |
| F-013 | refund/transfer cutoffs could precede the slot | `ensureUnit` validates windows against slot start and duration | `unit_cutoffs` |
| F-014 | `resume`/`pause` reactivated a delisted listing | `delisted` is terminal | `unit_provider_index` |
| F-016 | Bond refusals collapsed to a deploy error | `"closed"` maps to a booking refusal | `unit_bond_refusals` |
| F-017 | `Remedy.noShow` was dead and provider cancel bypassed `Remedy.cancel` | booking routes both through Remedy | `unit_no_show_contest`, `conservation` |
| F-018/F-010 | docs and inventory drift | docs corrected; counts regenerated | `make docs-check` |

The recorded remediation campaign ran 24/24 suites on an isolated v2 devnet, including the
16 base suites and eight new regression suites. The tree now defines 27; the three added P2
suites await the acceptance run. See `audit/evidence/` and
[Verification record](verification.md).

## P2/P3 — open follow-up

The remaining findings cover `Terms.pricePerSlot` vs unit price binding, governance
timelock, privacy review, migration and behavioural coverage (R-012, R-016–R-021).

## What is not claimed

The audit and remediation do not certify the contracts or the node. v1 listings, bookings,
ledgers and provider records keep v1 semantics and are not claimed fixed; only the v2
deployment is remediated. The remaining P2/P3 findings stay open. The Lean result is `proved-model`,
not a Rholang refinement, and no `proved-tied` rule is claimed.
