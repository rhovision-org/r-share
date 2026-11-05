# Proof roadmap

> **Status:** Baseline audit of the Lean model. The current result is `proved-model` only.
> No rule claims `proved-tied`; no refinement is proved.

## What exists

`spec/lean/` is a self-contained Lean 4.12 project with no Mathlib. It models a one-value
cell, a per-listing allocator, a booking status graph, a Bond, remedy arithmetic and an
op-id ledger. The source has 72 theorems, all named in `RShare/Audit.lean`. The flow
manifest has 35 rules over 31 arms; the repository has 119 public arms. The no-show contest
adds `noShowClaimed`, `noShowDisputed` and `noShowRefunded` plus source-status theorems.
Several `proved-model` entries point at `def`s or `Step` constructors rather than theorems.
`System.disjoint_cells_commute` and `provider_pause_not_structural` were removed; the acquire boundary is now an explicit assumption.

## Status vocabulary

| Status | Meaning | Evidence |
|---|---|---|
| `proved-model` | kernel-checked theorem about the Lean model | `#print axioms`, allowed axioms only |
| `proved-model-bounded` | kernel-checked theorem over a stated finite bound | `decide` or equivalent |
| `modeled` | function/transition defined; no safety theorem | declaration |
| `assumption` | explicit boundary assumption | registry and docs |
| `owed` | no model yet | register row |
| `proved-tied` | proved plus mechanical conformance | corpus + harness + manifest |

A `def` or constructor is never `proved-model`.

## Ladder

### L0 — hygiene and completeness

- audit every theorem, not at least 20 lines;
- require `proved-model` targets to be theorems or aliases;
- validate `source_globs` and the invariants block;
- reject vacuous `True` proofs carrying status;
- reconcile the duplicate S-numbering with `INV-###` IDs;
- enumerate every public arm against the flow manifest.

### L1 — executable model and bounded checks

Prove rent = sum of unit price × slot count, reset safety, event-settle timing, no-show
reachability, ledger pipeline termination and failure compensation. Add bounded exhaustive
checks over small units/slots/heights with `decide`.

### L2 — concurrency and deploy atomicity

Model disjoint-cell commutation, single-cell linearizability, deploy rollback and the
take/await/put failure class. The deployed `acquire` is not a single commit, so its tie
stays `assumption`/`owed` until the contract is restructured.

### L3 — authority and value

Model admin keys, threshold, distinctness, domains and nonces; model per-asset account and
escrow conservation. r-cash purse internals and node ERTP remain assumptions.

### L4 — refinement research

Mechanise the used Rholang fragment and prove a simulation/refinement. This is a research
track, not a scheduled deliverable.

## `proved-tied`

A rule may claim `proved-tied` only with a proved/bounded theorem naming an executable model
function, a generated conformance corpus, coverage of success and every refusal branch plus
boundaries, a harness run against the deployed contract, and a pinned evidence manifest. It
means sampled observational conformance, not refinement.

## Honest claim

The current model proves local properties of a hand-written state machine. It does not prove
that the deployed Rholang implements that machine, that the node schedules it correctly, or
that r-cash conserves value. See `audit/lean/rigour-ladder.md` for the work packages.
