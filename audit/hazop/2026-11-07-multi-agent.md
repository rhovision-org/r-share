# Multi-agent HAZOP and red-team review — 2026-11-07

Scope: the 19 Rholang deploy terms and the 7 inlined lib fragments, the booking/slot/Bond
workflow, the cover engine, the admin and registry boundaries, and the Lean model.

Method: three role-separated panels (workflow races, red team, Lean/HAZOP model), followed
by adversarial verification of every critical claim against the exact Rholang. Findings are
recorded in `audit/registers/hazards.yaml` as HZ-055…HZ-063. The match-specific code audit
is separate: `audit/rholang-match-audit.md`.

## 1. What a race can be here

- A deploy is atomic and blocks are sequential; two signed deploys do not interleave inside
  one arm.
- The residual race surface is cross-deploy TOCTOU, lazy-expiry reclaim, and multi-cell
  non-atomicity in the cover flow.
- Replay/idempotency (`opId`, admin nonce, Bond `paid`, claim/loss maps) is not a race; it
  arbitrates a repeated intent, not two different intents.

The controlling rules are: a peek must not be carried across an external await into a write;
a mutating arm must repost the cell on every refusal; every reply match must be total;
`<-` consumes and `<<-` peeks.

## 2. Workflow race verdicts

| id | pair | verdict | control |
|---|---|---|---|
| R1 | accept vs cancel/reject/expire | eliminated | one booking cell, status gate |
| R2 | settle vs settle/claimDeposit/resolveDeposit | eliminated | claim field and status guards |
| R3 | dispute/arbitrate/no-show pairs | eliminated | mutually exclusive status + height windows |
| R4 | two books on one slot | eliminated | single allocator cell, live hold |
| R5 | multi-slot partial hold | eliminated | all-or-none check, then apply |
| R6 | foreign/stale release token | eliminated | token-gated confirm/release |
| R7 | expiry reclaim then holder terminal | mitigated | `"not holder"` treated as released; refund independent |
| R8 | reset vs concurrent acquire/reclaim | eliminated | reset consumes and re-reads the cell |
| R9 | pause/delist vs in-flight acquire | mitigated | deploy order decides; committed booking stands |
| R10 | quote then book after a price change | mitigated | acquire revalidates; Bond uses authoritative summary |
| R11 | Bond double pay | eliminated | `paid` flag + private settle token |
| R12 | mid-payout abort | eliminated | deploy atomicity; unexpected branch aborts |
| R13 | release slots vs Bond pay | eliminated | release before pay in every terminal path |
| R14 | duplicate admin nonce | eliminated | per-object nonce under cell consume |
| R15 | key rotation vs old signature | mitigated | action re-verifies against the current admin set |
| R16 | registry register/rotate/freeze | eliminated | frozen/owner/version checks under one consume |
| R17 | duplicate guest/provider book creation | eliminated | peek-consume double check |
| R18 | refund after slot reclaim | mitigated | refund does not depend on release success |
| R19 | duplicate cover policy/subject | eliminated | pool policy/subject maps under one cell |
| R20 | concurrent bind vs capacity/solvency | eliminated | checked under the pool cell |
| R21 | cover pay vs settle/expire/attach | eliminated | policy status and cap under one cell |
| R22 | double claim/loss recovery | eliminated | claim map + shared loss ledger |
| R23 | attach vs settle | eliminated | one policy cell; attach is one-time |
| R24 | resolve before authorize/attachClaim | mitigated | `bcCoverPay` refuses `"claim token"`; test added |
| R25 | operator close/release vs booking settle | mitigated | attached close refused; missing entry treated as settled |
| R26 | permissionless expire vs pending claim | mitigated | expire refuses attached policies |

Accepted residual behavior: R7/R18 rely on the lazy-expiry doctrine; R9/R10 are serialized
deploy order; R24–R26 are mitigated by the cover guards and tests.

## 3. Red-team findings

| id | finding | verdict | status |
|---|---|---|---|
| EX-1 | `bindReserved` drains a provider reserve without a real hold | confirmed | fixed: `Listing("verifyHold")` required; no-hold test added |
| EX-2 | guest dispute after `claimDeposit`/`attachClaim` nullifies cover | confirmed | fixed: dispute refuses a pending/attached claim |
| EX-3 | public `Listing.acquire` is free and can hold inventory | confirmed | **open design item**; see section 5 |
| EX-4 | second `claimDeposit` overwrites the fixed claim | confirmed | fixed: claim frozen after first deposit and after attach |
| EX-5 | permissionless `CoverPool("premium")` inflates `premiumIn` | refuted | donation, not theft; accounting hygiene only |
| EX-6 | operator `release`/`close` accepts arbitrary `paidAmount` and strands the purse | confirmed | fixed: attached refused, measured amount and `entry.paid` reconciled |
| EX-7 | zero-premium offers exhaust capacity | confirmed | fixed: `premiumTotal >= 1` |
| EX-8 | solvency floor integer rounding; permissive config | confirmed/negligible | deferred; owner-only parameter risk |
| EX-9 | `attachCover` idempotency trusts only `policyId` | low | accepted; guest auth + private attach key |
| EX-10 | governance nonce greater-than semantics | low | accepted; admin-controlled |
| EX-11 | blocked paths rechecked | — | reset, guest-book, ledger failure, event settle, asset allowlist, factory auth, Bond pay |

## 4. Contract and proof changes

- `32_cover_pool.rho`: `bindReserved` now takes `unitSlots` and `releaseToken` and calls
  `Listing("verifyHold")` before any reserve debit. `expire`/settle paths, release/close
  accounting, reserve cap, and missing-pool-entry settle were hardened.
- `33_cover_policy.rho`: `authorize` is attached-only and refuses a repeated claim; `expire`
  refuses attached policies.
- `34_booking_instance.rho`: `claimDeposit` is one-shot; `dispute` refuses a pending or
  attached claim; `bcCoverPay` refuses an empty claim token instead of blocking; dead
  `bcCoverPaid` removed.
- `30_cover_terms.rho`: positive premium required.
- Lean: `Linear/Concurrency.lean`, `Linear/Deploy.lean`, `Model/Cover.lean` add 22
  kernel-checked theorems (94 registered). They prove disjoint-commit commutation, deploy
  rollback, fixed/one-time claim tokens, attached-only payout, attached expiry, idempotent
  terminal settle, claim freeze and dispute refusal.

## 5. Open items

1. **EX-3 / HZ-061 — free `Listing.acquire`.** Recommended design: a ledger-issued hold
   proof (`GuestLedger` mints a private key; `Listing` verifies it through a shared
   `HoldBook`, consumed by confirm/release). Fallback: accept and monitor with
   `maxUnits`/`holdBlocks`/pause; do not claim it eliminated.
2. **Runtime verification.** The isolated r-cash install failed and the default devnet is
   missing `component:asset-resolver`; no behavior suite has run against the latest
   contracts. A preflight now fails an RShare install early when r-cash is incomplete.
3. **EX-8 rounding/config.** Deferred; operator-only and sub-atomic-unit in practice.
4. **Helper extraction and `1/0` cleanup.** Remaining code-quality items, not safety
   regressions; converting `1/0` to refusals is a behavior change requiring per-site tests.

## 6. Assumptions not proved

Rholang refinement; unforgeable names/claim token; ERTP purse atomicity; node deploy
atomicity; operator settle trust; block-height clock; the `Listing.verifyHold` deployment
tie (`bindReservedRequiresHold` in `System.Boundary`). The Lean results are `proved-model`,
not a refinement of the deployed node.
