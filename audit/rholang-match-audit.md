# Rholang `match` and effective-code audit

Date: 2026-11-07

Scope: `contracts/src/deploy/*.rho` (19 deploy terms) and `contracts/src/lib/*.rho`
(7 inlined fragments). Read-only audit first; refactors are listed by risk tier.

## 1. Why this audit exists

The contracts use 455 `match` expressions (442 deploy, 13 lib). Every current `match` ends
in a wildcard or bare variable, so none is silently non-exhaustive. The problems are
structural: a `match` used only as a `let`, a second `match` on the same scrutinee, a
duplicated refusal branch, or a flat string dispatch written as an `if/else` ladder.

The target node (`rchain-rust`, rholo) makes the preservation rules precise:

- `if`/`else` is sugar for `match` on booleans.
- An unmatched `match` is **silence**, not an error. Every match over a cell or reply value
  must stay total and must repost the cell before replying.
- First matching case wins in source order; refusal order is part of the behavior.
- A pattern binding the same variable twice never matches.
- `map.get(k)` returns `Nil` for an absent key.
- `list.nth(i)` out of bounds aborts the deploy; keep the index guard.
- `<-` consumes a cell, `<<-` peeks; do not move that boundary.
- The arm inventory compares `contract Name(@"arm"` names, not source lines.

## 2. Inventory

| area | matches | dominant shapes |
|---|---:|---|
| `lib/Guard.rho` | 6 | single-arm `let`, `(true, Nil)` unit |
| `lib/AdminSet.rho` | 1 | `let` |
| `lib/Auth.rho` | 1 | result tuple |
| `lib/Money.rho` | 2 | `let` |
| `lib/Resolve.rho` | 1 | result tuple |
| `lib/AdminGuard.rho` | 1 | result tuple |
| `lib/AdminsText.rho` | 1 | `let` |
| `00_root.rho` | 22 | map presence, result tuple, action string |
| `01_kernel.rho` | 4 | `let` |
| `02_admin.rho` | 11 | result tuple, action string, unit |
| `03_terms.rho` | 9 | map presence, unit, result tuple |
| `04_remedy.rho` | 6 | `let`, result tuple |
| `10_provider.rho` | 18 | map presence, result tuple, unit |
| `13_unit.rho` | 10 | `let`, map presence, list-state dispatch |
| `14_listing_contract.rho` | 79 | result tuple, map presence, list-state dispatch |
| `15_bond.rho` | 15 | result tuple, `let` |
| `20_booking.rho` | 6 | result tuple, map presence |
| `22_booking_contract.rho` | 1 | result tuple |
| `23_guest_ledger.rho` | 26 | result tuple, map presence, 3-arm reserve dispatch |
| `24_guest_ledger_cover.rho` | 14 | result tuple, unit |
| `30_cover_terms.rho` | 2 | `let`, result tuple |
| `31_cover_offer.rho` | 12 | result tuple, map presence |
| `32_cover_pool.rho` | 51 | map presence, result tuple, auth unit |
| `33_cover_policy.rho` | 25 | map presence, result tuple, clock |
| `34_booking_instance.rho` | 115 | double op match, result tuple, clock/auth, status string |

The largest files by built-term bytes are `34_booking_instance.rho` (35328), `14_listing_contract.rho`
(28693), `32_cover_pool.rho` (23116), `23_guest_ledger.rho` (13141), `33_cover_policy.rho`
(10645), and `10_provider.rho` (10182). Depth is not the binding constraint; bytes and the
36-contract count are.

## 3. High-value, low-risk refactors

### 3.1 Merge the double op lookup in `34_booking_instance.rho`

Twenty arms write:

```rho
match st.get("ops").get(opId) {
  prev => {
    match prev {
      Nil => { ... }
      _ => { bcReplay!(st, prev, *ret2) }
    }
  }
}
```

The outer match is a `let`; the inner match is the dispatch. Merge them:

```rho
match st.get("ops").get(opId) {
  Nil => { ... }
  prev => { bcReplay!(st, prev, *ret2) }
}
```

Sites: `accept`, `reject`, `cancel`, `cancelByProvider`, `checkIn`, `checkOut`,
`guestCheckOut`, `settle`, `claimDeposit`, `resolveDeposit`, `dispute`, `arbitrate`,
`resolveDispute`, `expire`, `claimNoShow`, `disputeNoShow`, `resolveNoShow`, `redeem`,
`closeEvent`, `transferTicket`. No new contracts, no refusal change.

### 3.2 Collapse the duplicated admin failure branch

Thirteen sites write:

```rho
match v {
  (true, ok) => {
    if (ok == false) { STATE!(s) | ret!((false, "below threshold")) }
    else { ... }
  }
  _ => { STATE!(s) | ret!((false, "below threshold")) }
}
```

The fixed reply shapes make both failures identical:

```rho
match v {
  (true, true) => { ... }
  _ => { STATE!(s) | ret!((false, "below threshold")) }
}
```

Sites: `00_root.rho` (4), `02_admin.rho` (1), `10_provider.rho` (2),
`14_listing_contract.rho` (2), `34_booking_instance.rho` (2), `40_payments.rho` (1),
plus the raw bool guard in `32_cover_pool.rho`. Preserve the repost and refusal string.

### 3.3 Collapse the re-match of `av` in `setAdmins`

`34_booking_instance.rho` matches `av`, then re-matches the same `av` inside the `_` arm.
Use one three-arm match:

```rho
match av {
  (true, Nil) => { ... }
  (false, reason) => { bcRefuse!(st, reason, *ret2) }
  _ => { bcRefuse!(st, "threshold", *ret2) }
}
```

### 3.4 String/status dispatch

- `32_cover_pool.rho` `poolDepositCheck`: `match effect { "none" => ...;
  "reduce_to_deductible" => ...; "remove" => ...; _ => (false, "deposit") }`.
- `24_guest_ledger_cover.rho` effective-deposit 3-way: `match
  offer.get("depositEffect") { "none" => ...; "reduce_to_deductible" => ...; _ => ... }`.
- `31_cover_offer.rho` pause/retire: keep the publisher pre-check, then
  `match entry.get("status") { "retired" => ...; "paused" => ...; _ => ... }`.
- `23_guest_ledger.rho` `GuestLedgerGate`: collapse both pause branches to
  `(true, false) => (true, Nil); _ => (false, "paused")`.

Each arm must keep its existing state repost and reply.

### 3.5 Remove duplication without adding contracts

- `32_cover_pool.rho` `CoverPool("reserve")`: normalize `Nil => 0`, `current => current`,
  then one cap check, one refund block, one accept block.
- `34_booking_instance.rho`: delete the uncalled `bcCoverPaid`.
- `32_cover_pool.rho`: use the existing `poolPolicyLookup` in `CoverPool("policy")` or
  delete the helper.
- `14_listing_contract.rho`: compute the shared `allocCheckUnits` accumulator once for
  `quote` and `acquire`; do not add a helper contract to this file.

### 3.6 Flatten nested `else { if ... }` ladders

Behavior-preserving bracketing only:

- `03_terms.rho` `termsSchedule` nine-condition ladder.
- `03_terms.rho` `Terms("validate")` prefix.
- `04_remedy.rho` `Remedy("cancel")` full/partial/late chain.
- `13_unit.rho` `unitSlotScan` start/end/past/future chain.
- Remaining status ladders in `34_booking_instance.rho`.

Preserve condition order and refusal strings.

## 4. Medium-risk refactors

### 4.1 `Remedy("cancel")` actor/kind dispatch

Replace the actor/kind `if/else` with a nested `match` on `actor`, then `kind`. The final
`_` maps to the existing `"actor"` refusal. Preserve the event and non-event refund splits.

### 4.2 Shared held-slot body in `13_unit.rho`

The four-tuple and three-tuple `["held", ...]` list patterns cannot merge, but their body can
move to a local `unitSlotAdvance` helper. This adds a local contract, so check the contract
budget.

### 4.3 `bcNow`/`bcAuth` helpers in `34_booking_instance.rho`

The repeated clock/auth rejection blocks can use two local helpers that own the repost. This
adds two contracts to the file at 32/36; do it only after the free reductions in §3.

### 4.4 `CoverSettle` and `release`/`close` shared tails

The four settle arms and the release/close retire tail repeat state-cell and purse handling.
Extract shared local helpers only after §3, and re-run `make measure` after each. These are
the highest-risk state-cell edits in the audit.

## 5. Do not force `match`

- **No guards.** Range, arithmetic, and membership checks stay `if/else` or flat `else if`.
  `03_terms.rho` `Terms("validate")` is an ordered 20-clause table; keep it flat.
- **No or-patterns.** `status == "settled" || status == "expired"`, `Nil || ""`, and negated
  membership cannot become one match pattern.
- **Linear-name identity.** `if (*key != attachKey)`, `if (tok == *token)` stay `if`; a
  match can bind the name but cannot compare identity in a pattern.
- **`nth` needs a guard.** Keep `i < length` before `match list.nth(i) { x => ... }`.
- **Single-arm `match` as `let` is idiomatic.** Do not rewrite it; there is no `let` in
  Rholang.
- **Silent destructuring receives.** `for (@(true, ...) <- ch)` swallows failure tuples. If a
  site needs an explicit refusal, convert it to `for (@r <- ch) { match r { (true, ...) =>
  ...; _ => ret!(r) } }`. This is a behavior change and is Tier 3, not Tier 1.

## 6. Higher-risk findings, deferred

- **37 `_ => { 1/0 }` arms** discard the failure reason and abort the deploy. Some are
  intentional invariant crashes. Converting them to refusals is a behavior change and needs
  per-site review.
- **Silent init receives** in `32_cover_pool.rho` and several other files: `for (@(true, ...)
  <- ch)` can stop a deploy without a refusal. Convert only with tests that cover resolution
  failure.
- **Deep nesting in `14_listing_contract.rho`, `15_bond.rho`, `33_cover_policy.rho`, and
  `32_cover_pool.rho`**: helper extraction is correct but costs contract names; defer until
  the free reductions land and `make measure` shows headroom.

## 7. Refactor protocol

For every file:

1. Diff the set of `contract X(@"arm"` names before and after; it must be unchanged.
2. Diff refusal strings and their order; they must be unchanged unless the change is
   explicitly a behavior change.
3. Check every `<-` vs `<<-`; do not move a receive boundary.
4. For every mutating arm, every branch that consumes a cell must repost it before replying.
5. Every new `match` must end in `_`, `false`, or `Nil` that reposts and replies.
6. Do not repeat a pattern variable.
7. Keep the `nth` index guard.
8. Do not add a contract if the file is at its byte/contract cap; do not raise a cap.

Static gates after each batch:

```sh
make build-check
make measure
make parse-check
make spec-check
make audit-check
make check
```

Behavior suites only after all refactors are written:

- `unit_lifecycle`, `unit_allocator`, `unit_bond_refusals`, `unit_admin`,
  `unit_admin_keys`, `unit_reset_safety`, `unit_cutoffs`, `unit_pause_scope`.
- `unit_cell_liveness`, `unit_settle_release`, `unit_conservation`, `unit_behavior`.
- `unit_cover`, `unit_cover_policy`, `unit_cover_booking`, `unit_cover_disabled`.
- One final `make unit-all`; not per edit.

## 8. Priority

1. §3.1 double op match in `34` (largest, zero risk).
2. §3.2 admin failure collapse (13 sites, zero risk).
3. §3.5 duplication removal (`reserve`, dead helpers).
4. §3.4 string dispatch and §3.6 ladder flattening.
5. §4 helper extraction only if `make measure` shows headroom.
6. Defer §6 behavior changes to a separate reviewed batch.
