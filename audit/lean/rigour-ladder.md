# Lean rigour ladder

The current model is a hand-written sequential state machine. It is useful, but it is not
Rholang semantics and it does not prove a refinement. The ladder makes the claim class
explicit and raises it only when the artifact exists.

## Status vocabulary

| Status | Meaning | Required evidence |
|---|---|---|
| `proved-model` | kernel-checked theorem about the Lean model | `#print axioms` line; no forbidden axiom |
| `proved-model-bounded` | kernel-checked theorem over an explicitly bounded finite domain | bound stated; `decide` or equivalent |
| `modeled` | transition/function defined; no safety theorem | declaration only |
| `assumption` | explicit boundary assumption; not proved | registry entry and docs |
| `owed` | no model yet | register row |
| `proved-tied` | proved plus mechanical conformance corpus | checklist below |

A `def`, `inductive` or constructor is `modeled`, never `proved-model`. `True := trivial`
is not a proof.

## L0 — hygiene and completeness

- `Audit.lean` names every theorem and alias target; `check_lean.py` compares the audit to
  the theorem registry and rejects unaudited theorems.
- `check_flows.py` validates `source_globs` and `invariants`, requires `proved-model`
  targets to be theorems or aliases, adds `modeled`, `assumption` and
  `proved-model-bounded`, rejects vacuous `True` theorems carrying proof status, and
  validates `proved-tied` corpora.
- `check_arm_inventory.py` compares every public arm to the flow manifest; an omitted arm
  needs an `unmapped` row with a rationale.
- Resolve the duplicate S-numbering in favour of `INV-###` IDs that map to
  `spec/invariants.md`.

At baseline: 63 theorems in source, 38 audited. The L0/L1 and no-show pass now names all
72 source theorems. The flow manifest has 35 rules over 31 arms; 119 public arms remain in
the inventory, with the rest explicitly unmapped.

## L1 — executable model and bounded checks

Add local theorems over the existing model:

- `rentOf_eq_sum`, `summaryOf_rent`, `acquire_count_eq` for invariant 7;
- `confirm_preserves_keys`, `release_frees_token`, `acquire_refusal_unchanged`;
- `reset_free_only`: reset changes only free/expired-held slots;
- a guarded booking machine with `no_show_requires_confirmed_and_window`,
  `settle_redeemed_requires_event_end`, and a reachable-trace theorem showing that a
  provider can reach no-show without checking in;
- ledger `pipeline_terminal`, `factory_failure_drops`, `factory_unavailable_drops`,
  `release_on_failure`;
- bounded exhaustive checks over 2–3 units, 2–4 slots, heights 0–8 and 1–3 reservations,
  closed with `decide` and labelled `proved-model-bounded`.

## L2 — concurrency, cells, deploy atomicity, linearizability

New `Linear/Concurrency.lean` and `Linear/Deploy.lean`:

- `commit_commute`, `run_disjoint_commute`;
- `single_cell_linearizable`, `disjoint_cells_linearizable`;
- `commit_preserves_wf`, `run_failure_no_leak`, `deploy_atomic_rollback` (node atomicity is
  an explicit assumption);
- `held_across_external_call_unsafe`: a counterexample for HZ-011;
- `external_read_then_commit_safe`: revalidation after a pure read is sound.

The deployed `acquire` is not a single commit; its tie remains `assumption`/`owed` until the
contract is restructured or the model is changed to the take/await/put pattern.

## L3 — admin thresholds and ERTP conservation

New `Model/Admin.lean` and `Model/Asset.lean`:

- `AdminWf`: `Nodup keys`, `1 ≤ threshold`, `threshold ≤ keys.length`, plus a high-value
  policy flag requiring threshold at least 2;
- `duplicate_key_breaks_threshold`; `quorum_requires_distinct`;
- `setAdmins_wf`, `nonce_monotone`, `owner_authority`;
- `transfer_conserves`, `deposit_conserves`, `withdraw_conserves`,
  `settlement_conserves`, `no_cross_asset`, `bond_pay_moves_expected`.

r-cash Purse internals, issuer trust and node ERTP behaviour remain assumptions.

## L4 — refinement research

A later research track would need an operational semantics for the used Rholang subset, a
translation or abstraction from deployed terms, a simulation/bisimulation relation, and
treatment of names, unforgeability, comm scheduling and deploy atomicity. It is recorded as
research and is not claimed by this audit.

## `proved-tied`

`proved-tied` means observational conformance on a generated corpus, not refinement. It
requires a proved/bounded theorem naming an executable model function; a generated corpus
under `spec/vectors/conformance/`; coverage of success and every refusal branch plus
boundary and invalid inputs; a harness run against the deployed contract on a pinned devnet;
an evidence manifest with commits, node version, command and hashes; and a checker that
rejects stale or missing evidence. At baseline no rule claims `proved-tied`.
