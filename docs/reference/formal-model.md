# Formal model

> **Status:** Implemented. The model builds with Lean 4.12 and no Mathlib. Its results are
> `proved-model`: theorems about the Lean state machine, not a refinement of the deployed
> Rholang. The flow manifest is the traceability link; it is not a correctness proof of the
> node or of Rholang reduction.

## What is proved

`spec/lean/` is a self-contained Lean project:

| Module | Content |
|---|---|
| `RShare/Basic.lean` | amounts, heights, division helpers |
| `RShare/Linear/Cell.lean` | one-value cells, `peek`/`take`/`put`/`commit`, well-formedness, D1–D3 rejection theorems |
| `RShare/Linear/Protocol.lean` | pure helper, caller-token, delegated and await-then-commit protocols |
| `RShare/Linear/Concurrency.lean` | atomic commit blocks, disjoint-cell commutation, permutation invariance, `disjoint_cells_linearizable` |
| `RShare/Linear/Deploy.lean` | all-or-none deploy wrapper, rollback, commit well-formedness, the hold-across-external-call counterexample |
| `RShare/Model/Allocator.lean` | per-listing allocation: one reservation append, single-owner slots, bounded holds, token-gated confirm/release, token-free projections |
| `RShare/Model/Booking.lean` | status graph, terminal stickiness, every status has a path to a terminal |
| `RShare/Model/Bond.lean` | single-use pay, bucket conservation |
| `RShare/Model/Remedy.lean` | fee, full refund, no-show and deposit-split conservation |
| `RShare/Model/Ledger.lean` | reserve/finish/drop replay shape |
| `RShare/Model/Cover.lean` | cover policy/binding state machine, fixed claim token, one-time pay, attached expiry, idempotent terminal settle, claim freeze, dispute refusal |
| `RShare/Model/System.lean` | composition aliases and the explicit boundary |
| `RShare/Invariants.lean` | stable S-names for the flow manifest |
| `RShare/Emit.lean` | deterministic vectors consumed by the suites |
| `RShare/Audit.lean` | `#print axioms` for every core theorem |

The central result is `RShare.Allocator.acquire_no_double_owner`: after a successful
acquire, every active reservation containing a requested key belongs to the acquiring
booking. `acquire_none_or_reservation` and `acquire_is_one_reservation` state the structural
shape: an acquire either refuses and leaves the state unchanged, or appends exactly one
reservation. There is no reachable partial state.

## Why races are not representable

The allocator's `confirm` and `release` are single-cell commits. `acquire` is a
take/await/put: it takes `listingState`, checks the pause/provider gates, awaits
`UnitFactory.slotScan`, and reposts. The booking, Bond and ledger cells are single-writer.
Two operations on the same cell are ordered by the cell; two operations on different cells
do not interact. The old multi-slot hold loop is deleted: the contract no longer has a state
in which unit A is held and unit B is not.

The model does **not** claim the node's Rholang reduction or scheduler. `Linear/Concurrency`
proves that commit-only traces on disjoint cells commute; `disjoint_cells_linearizable`
proves that permuting disjoint commit blocks preserves the result. It does not prove the
deployed `acquire` is a single commit; `held_across_external_call_unsafe` is the explicit
counterexample for the take/await/put window, and the acquire tie remains an assumption.

The deployed `acquire` now checks ShareAdmin pause and ProviderContract status; those checks
are not proved as a refinement of the model. A caller can still hold free slots without a
booking; that grief is bounded by `maxHoldBlocks`. `bindReserved` now requires a live
`Listing("verifyHold")`; the deployment tie is recorded as `bindReservedRequiresHold` in the
boundary, not proved.

The cover model proves claim-token and terminal-settle properties. ERTP purse atomicity,
claim-token unforgeability, operator trust, node deploy atomicity and Rholang refinement are
boundary assumptions, not theorems.

Two further boundaries: the allocator model abstracts the schedule/lead/duration/window
checks that `UnitFactory.slotScan` performs on the deployed term; those are covered by the
allocator and booking suites, not by the Lean `acquire`. The booking model is the status
graph; owner, claim, arbitration and deadline guards are listed as `owed` in the flow
manifest rather than proved.

## Status vocabulary

| Label | Meaning |
|---|---|
| `proved-model` | Machine-checked about the Lean model; no node tie. |
| `proved-model-bounded` | Machine-checked over a stated finite bound. |
| `modeled` | A definition or transition exists; no safety theorem. |
| `assumption` | An explicit boundary assumption; not proved. |
| `proved-tied` | Proved and tied to the implementation by a mechanical corpus. None claimed. |
| `owed` | An arm outside the current cornerstone model. |

`spec/flows/core.yaml` names the Lean declaration for every cornerstone arm and lists the
core arms that must be `proved-model`. `tools/check_flows.py` rejects a missing arm, a
missing Lean declaration, an unknown status, or a `proved-tied` without a corpus. It warns
when a `proved-model` target is a definition or constructor; `make check-flows-strict`
turns those warnings into failures.

## Audit baseline

The theorem registry now has 94 theorems, all named in the axiom audit. The flow manifest
has 43 rules over 39 arms, every `proved-model` target is a theorem, and
`make check-flows-strict` reports zero warnings. `System.disjoint_cells_commute` and
`provider_pause_not_structural` were removed; the acquire and cover deployment ties are
explicit assumptions. No rule claims `proved-tied`. The theorem registry and proof ladder are
in `audit/registers/theorems.yaml` and `audit/lean/rigour-ladder.md`.

## Run it

```sh
make lean-check      # lake build + axiom audit; rejects sorry/admit/axiom/native_decide
make vectors         # re-emit and compare spec/vectors/core.json
make check-flows     # source arms, flow rules and Lean names agree
make check-flows-strict # fail on def/constructor proved-model targets
make formal-check    # build, audit, vectors and flows
```

`unit_allocator` exercises the allocator transition table on-chain; `unit_claims` compares
the remedy vectors directly against the deployed `Remedy` contract.

> **Where to go deeper.** [Verification record](verification.md) is the behavioural run.
> [Invariants](invariants.md) is the normative list. [Concurrency
> doctrine](../concepts/concurrency-doctrine.md) is the allocation-cell rule.
