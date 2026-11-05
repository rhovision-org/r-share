# Verification record

> **Status:** The authoritative remediation record is 24 suites on an isolated v2 devnet,
> plus the CLI journey and the browser journeys. The source tree now defines 27 suites; the
> three added P2 suites await the acceptance run and are marked pending below. It covers allocator acquisition, lifecycle, claims,
> booking, pause, admin on every `setAdmins` surface, events, provider and listing indexes,
> conservation, reset safety, terminal slot release, the no-show contest, event timing,
> admin key sets, pause scope, cell liveness, factory authentication, cutoff arithmetic and
> Bond refusal mapping. It does not cover every mutating arm, the two design-only claim
> classes or fleet-scale allocation. The formal model is recorded separately below.

## Remediation run (v2)

| Field | Value |
|---|---|
| Date | 2026-11-04 |
| r-share commit | `dee7cff` plus the uncommitted remediation tree (`dirty: true`) |
| rholo | `e2d073dcd02f1e560ccb84749937138a1e184b0b` |
| r-cash | `49b441d` |
| Node | isolated `rshare-audit` devnet, `localhost:41403` |
| Command | `harness/run_suites.py` (24 recorded; 27 defined), `cargo run -p rshare-cli -- devnet journey`, `npx playwright test` |
| Evidence | `audit/evidence/2026-11-04-remediation/` |

Results:

```
root           pass   keys, resolve, unknown key, stale version
payments       pass   health, platform owner, REV issuer, unknown asset refused "asset"
lifecycle      pass   dispute/arbitrate state gates; resolveDispute payout
claims         pass   claim classes across five kinds; no-show contest window
pricing        pass   unit prices stored; zero price refuses "price"
guestbook      pass   getOrCreate stable; ledger read-back matches
booking        pass   two priced units, per-unit windows, rent, bond, escrow
pause          pass   global/provider/unit pause and resume
allocator      pass   atomic multi-slot acquire; token-only confirm/release
admin          pass   booking/listing/provider setAdmins; threshold and replay
events         pass   redeem, closeEvent, transferTicket
generality     pass   five kinds validate; unknown kind refuses
provider       pass   register, profile update, pause/resume, owner check
listing        pass   create, unit, activate, pause/resume, delist, projections
behavior       pass   normalized lifecycle transcript matches the golden file
conservation   pass   eight scenarios; closed-set sum constant; provider cancel
reset_safety   pass   free/expired-held reset; live held and booked refused
settle_release pass   settle/resolveDeposit/closeEvent free the slot
no_show_contest pass  claim, dispute, resolve; default guest refund
event_timing   pass   settle/closeEvent before event end refuse
admin_keys     pass   duplicate keys, threshold 1 and >keys refused
pause_scope    pass   global and provider pause gate direct acquire
cell_liveness  pass   factory failure leaves cells live and no pending op
factory_auth   pass   direct BookingContractFactory call without a hold refused
provider_index pending  removeListing delists; delisted is terminal for pause/resume
cutoffs        pending  refund/transfer windows must fit the slot window
bond_refusals  pending  Bond "closed" maps to a booking refusal
```

`harness/run_suites.py` writes `deployments/suite-results.json` (gitignored) and exits 0 only
when every selected suite passes.

## Baseline run (v1)

| Field | Value |
|---|---|
| Date | 2026-11-03 |
| r-share commit | `41b7e06` (allocator and formal model); this chapter is the documentation commit |
| rholo | `e2d073dcd02f1e560ccb84749937138a1e184b0b` |
| r-cash | `49b441d` |
| Node | fresh local devnet, `localhost:40403` |
| Command | `make devnet-up rcash-keys rcash-bootstrap keys install && make unit-all` |

Results:

```
unit-root      pass   keys, resolve, unknown key, stale version
unit-payments  pass   health, platform owner, REV issuer, unknown asset
unit-lifecycle pass   dispute from requested refuses;
                      arbitrate outside dispute refuses;
                      checked-out booking resolves through resolveDispute
unit-claims    pass   four claim classes x five kinds (20/30 coverage cells)
unit-pricing   pass   units store 10 and 20; a zero price refuses "price"
unit-guestbook pass   getOrCreate is stable; ledger read-back matches
unit-booking   pass   two priced units, per-unit windows, rent 30, bond 80;
                      escrow debit/refund and bond closure
unit-pause     pass   global pause/resume, provider pause/resume, owner unit pause/resume
unit-allocator pass atomic multi-slot acquire; one held slot refuses the set;
                      token-only confirm/release; pause, expiry and privacy checks
unit-admin     pass   booking, listing and provider setAdmins; threshold and replay
unit-events    pass   redeem, closeEvent and transferTicket
unit-generality pass  five kinds validate; an unknown kind refuses
unit-provider  pass   register stable, profile update, pause/resume, owner check
unit-listing   pass   create, unit, activate, pause/resume, delist, projections
unit-behavior   pass   normalized lifecycle transcript matches the golden file
unit-conservation pass seven real scenarios; closed-set sum constant; bond closed;
                       negative control detects an untracked Bond
```

The runner writes `deployments/suite-results.json` (gitignored) and exits 0 only when every
selected suite passes.

## What the lifecycle suite proves

- `dispute` from a `requested` booking refuses `"closed"`; it cannot freeze escrow.
- `arbitrate` on a non-disputed booking refuses `"closed"`; the 2-of-3 set cannot bypass the
  state machine.
- A `confirmed` booking can `checkIn`, `checkOut`, `dispute`, and then any caller can
  `resolveDispute` once the dispute window has elapsed. The default pays the deposit to the
  guest and rent net of fee to the provider.

The suite uses a stub Bond and an empty unit set. It is a state-machine test, not a
marketplace scenario; it does not prove slot settlement or ERTP conservation.

## What the claims suite proves

- `cancel` from a requested booking returns the full rent and deposit.
- `claimNoShow` after the no-show window pays the provider rent net of fee and releases the
  deposit.
- `claimDeposit` followed by `resolveDeposit` splits the deposit between provider and guest.

The same three scenarios run for lodging, event, bike, car and robot. They exercise the
remedy arithmetic and the booking state machine; they do not exercise a real Bond, slots,
or ERTP conservation.

## What the pricing suite proves

Two units created with prices 10 and 20 report those prices from `ListingContract("unit")`,
and a price of zero refuses `"price"`. The booking suite proves the ledger sums them: a
booking of both units has `rent == 30`.

## What the booking suite proves

The end-to-end path registers a provider, creates and activates a robot listing, creates a
machine at 10 and an operator at 20 with different slot windows, funds the deployer's r-cash
account through the deposit facet, and books both units. The booking has `rent == 30`,
`checkInHeight == min(unit windows)`, `checkOutHeight == max(unit windows)`, status
`"confirmed"`, and its Bond reports `expected == 80` (rent + deposit).

The suite then measures the account balance: the booking debits exactly 80, a full-refund
cancel restores it, and the Bond ends with `paid: true, remaining: 0`. This is a one-booking
conservation check, not the full per-asset invariant harness.

## What the pause suite proves

`ShareAdmin.admin` accepts a 2-of-3 signed `pauseAll` and `resumeAll`; `isPaused` flips
accordingly. A provider can pause and resume, and `isActive` follows. The listing owner can
`pauseUnit` and `resumeUnit`, and `ListingContract("unit")` reports the status. A second
provider register returns the same provider URI, exercising the existing-provider fast
path.

## What the allocator and event suites prove

`unit_allocator` proves that a multi-slot request is all-or-none, that one held slot refuses
the whole set, that confirm and release require the booking token, that listing and unit
pause refuse acquisition, that an expired hold is reclaimed, and that `info`, `unit` and
`probe` never expose a slot capability. The interleaving invariants are proved in
`spec/lean/RShare/Model/Allocator.lean`. `unit_admin` proves a one-signature admin
action refuses `"below threshold"` and a reused nonce refuses `"replay"`. `unit_events`
proves `redeem` moves a booking to `"redeemed"`, `closeEvent` settles it, and
`transferTicket` moves the ticket holder. `unit_generality` validates terms for all five
kinds and refuses an unknown kind. `unit_provider` and `unit_listing` cover the provider
and listing arms end to end, including profile update, unit creation and delist.
`unit_behavior` records a normalized lifecycle transcript (refusals, booleans, evidence
hash) and compares it to `harness/scenarios/golden/lifecycle.json`; heights and URIs are
never part of the transcript.

## What the guestbook and evidence checks prove

`GuestBook("getOrCreate")` returns the same ledger on a second call and `GuestBook("ledger")`
reads it back; the existing-guest path no longer consumes and rewrites the index.
`BookingContract("info")` returns the dispute hash and height after a dispute, so the anchor
is readable rather than write-only.

## r-cash caveat

The pinned r-cash installer currently stops at its `51_pool` step because the node reports
`parse depth exceeded` for that term. `make rcash-bootstrap` tolerates that stop, runs the
r-cash `09_identity` step to install the account factory, and repairs
`deployments/devnet-v4.json` from the install state. r-share needs only the r-cash Root,
clock, asset resolver and account factory; the AMM pool is not on its path.

## Formal model record

`make formal-check` on the same checkpoint:

```
lean: built, 76 axiom audits, 72 source theorems, no sorry/admit/axiom/native_decide
vectors: 21 vectors current
check-flows: 35 rules, 31 arms, 13 core arms proved-model, 0 proof-quality warnings
```

The model proves allocator single-owner and all-or-none acquisition, bounded holds,
token-gated confirm/release, booking terminal progress, Bond single use and remedy
conservation. It does not prove a refinement of Rholang; `spec/flows/core.yaml` is the
traceability link. Every `proved-model` target is now a theorem and
`make check-flows-strict` reports zero warnings. See [Formal model](formal-model.md) and
the [proof roadmap](proof-roadmap.md).

## Audit baseline

The 2026-11-04 audit is a separate record from the behavioural campaign.

| Field | Value |
|---|---|
| Baseline commit | `dee7cff` |
| Node | isolated `rshare-audit` devnet, rholo `e2d073d` |
| Registers | `audit/registers/findings.yaml`, `hazards.yaml`, `theorems.yaml`, `improvements.yaml` |
| Reproduced | F-001, F-004, F-005, F-006, F-007 |
| Static only | F-002, F-003 and the remaining P2/P3 findings |
| Evidence | `audit/evidence/2026-11-04-dee7cff/` |
| Lean | 72 source theorems, all named in the axiom audit; 0 flow proof-quality warnings |

The audit baseline does not certify the contracts. The P0/P1 findings were subsequently
remediated in v2; P2/P3 remain open. See [Audit findings](audit-findings.md) and
[Rholang improvements](rholang-improvements.md).

## Remediation campaign

The P0/P1 remediation was exercised on a separate isolated devnet.

| Field | Value |
|---|---|
| Date | 2026-11-04 |
| Node | isolated `rshare-audit` devnet at `localhost:41403` |
| Suites | 24/24 recorded: the base suites plus reset safety, terminal release, no-show contest, event timing, admin keys, pause scope, cell liveness and factory auth. Provider index, cutoffs and Bond refusals are defined but pending acceptance |
| Contract fixes | reset guard, GuestBook repost, guest-ledger compensation, no-show contest, admin distinct/threshold, event settle guard, pause gates, factory origin check, terminal slot release, asset allowlist, provider delist, cutoff validation, delist terminality, Bond refusal mapping, Remedy routing |
| Formal | 72 source theorems, 76 axiom audits, strict flow gate 0 warnings |
| Client | CLI devnet journey 6/6 checks; web guest, provider and admin e2e 3/3 passed |
| Evidence | `audit/evidence/2026-11-04-remediation/` manifest and logs |

The 24/24 recorded run is a devnet remediation campaign, not a production audit. v1 instances keep
their old semantics; only the v2 deployment is claimed remediated.

## What is still not run

- `providerDeliveryBreach` and `partialPerformance`, which have no code;
- fleet-scale allocation and station capacity;
- provider-bond legs, obligation objects, generalized claims and evidence enforcement
  (Phase 2 design).

Do not describe those as passing. The coverage manifest records the real status.

## Client campaign

The reference client has its own recorded campaign on the same node.

| Field | Value |
|---|---|
| Date | 2026-11-04 |
| r-share client commit | `e84af13` |
| rholo | `e2d073dcd02f1e560ccb84749937138a1e184b0b` |
| r-cash | `49b441d` |
| Rust | 1.95.0 |
| Node | fresh local devnet, `localhost:40403` |
| Commands | `make client-fixtures`, `make client-journey`, `make client-keystores`, `cd web && npx playwright test` |

The CLI journey (`deployments/client-results.json`, gitignored) passed every check:

```
quote                         ok   rent matches the listing price
book                          ok   status confirmed
held slot refuses            ok   "slot held"
replay returns same booking  ok   same booking URI
wrong terms hash refuses     ok   "terms"
manual listing               ok   requested -> accepted -> cancelled
one admin signature          ok   "below threshold"
two admin signatures         ok   pauseAll and resumeAll finalized
```

The browser campaign passed three Playwright journeys:

```
guest-journey     create wallet -> faucet -> fund account -> catalogue -> quote -> book
provider-journey  register -> create non-instant listing -> add unit -> activate
                  -> guest books -> provider accepts
admin-multisig    import admin1 and admin2 -> build pauseAll payload -> sign both
                  -> submit -> paused; build resumeAll -> sign both -> submit
```

The admin journey uses the real global admin set; one signature is insufficient. The
journeys run with one worker because the admin test changes global pause state. They need a
fresh devnet, fixtures and a browser; the offline gates (`make client-check`) do not.

The WASM smoke test is offline: create/unlock/lock/export, wrong password, tampered MAC,
r-cash keystore cross-decrypt, a signature parity fixture against `tools/sign_payload.py`,
op-id format, evidence hashing and a preview size check.
