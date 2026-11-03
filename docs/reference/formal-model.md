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
| `RShare/Model/Allocator.lean` | per-listing allocation: one reservation append, single-owner slots, bounded holds, token-gated confirm/release, token-free projections |
| `RShare/Model/Booking.lean` | status graph, terminal stickiness, every status has a path to a terminal |
| `RShare/Model/Bond.lean` | single-use pay, bucket conservation |
| `RShare/Model/Remedy.lean` | fee, full refund, no-show and deposit-split conservation |
| `RShare/Model/Ledger.lean` | reserve/finish/drop replay shape |
| `RShare/Model/System.lean` | composition aliases and the explicit boundary |
| `RShare/Invariants.lean` | stable S-names for the flow manifest |
| `RShare/Emit.lean` | deterministic vectors consumed by the suites |
| `RShare/Audit.lean` | `#print axioms` for every core theorem |

The central result is `RShare.Model.Allocator.acquire_no_double_owner`: after a successful
acquire, every active reservation containing a requested key belongs to the acquiring
booking. `acquire_none_or_reservation` and `acquire_is_one_reservation` state the structural
shape: an acquire either refuses and leaves the state unchanged, or appends exactly one
reservation. There is no reachable partial state.

## Why races are not representable

The allocator's `acquire`, `confirm` and `release` are single-cell commits. The booking,
Bond and ledger cells are also single-writer. Two operations on the same cell are ordered by
the cell; two operations on different cells do not interact. The old multi-slot hold loop is
deleted: the contract no longer has a state in which unit A is held and unit B is not.

The model does **not** claim the node's Rholang reduction or scheduler. Provider pause is a
ledger gate before `acquire`, not an allocator field; a direct caller can acquire while the
provider is paused, bounded by `maxHoldBlocks`. A caller can hold free slots without a
booking; that grief is bounded by the same limit. These are stated as boundary theorems.

## Status vocabulary

| Label | Meaning |
|---|---|
| `proved-model` | Machine-checked about the Lean model; no node tie. |
| `proved-tied` | Proved and tied to the implementation by a mechanical corpus. None claimed. |
| `owed` | An arm outside the current cornerstone model. |

`spec/flows/core.yaml` names the Lean declaration for every cornerstone arm and lists the
core arms that must be `proved-model`. `tools/check_flows.py` rejects a missing arm, a
missing Lean declaration, an unknown status, or a `proved-tied` without a corpus.

## Run it

```sh
make lean-check      # lake build + axiom audit; rejects sorry/admit/axiom/native_decide
make vectors         # re-emit and compare spec/vectors/core.json
make check-flows     # source arms, flow rules and Lean names agree
make formal-check    # all three
```

The suites consume the vectors: `unit_allocator` exercises the allocator transition table
and `unit_claims` compares the remedy vectors against the deployed `Remedy` contract.

> **Where to go deeper.** [Verification record](verification.md) is the behavioural run.
> [Invariants](invariants.md) is the normative list. [Concurrency
> doctrine](../concepts/concurrency-doctrine.md) is the allocation-cell rule.
