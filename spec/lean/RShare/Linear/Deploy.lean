import RShare.Linear.Concurrency

/-!
# Deploy atomicity

A deploy is all-or-none: it either exposes the store and held set that the
whole trace produced, or it leaves the pre-state untouched. Node deploy
atomicity is the model's explicit assumption (`deploy_atomic_rollback` is its
Lean statement, not a derivation from Rholang semantics).

The file also records the two cell-pattern facts the audit leans on:

* `commit_preserves_wf` — a commit never changes the held set, so appending a
  successful commit to a well-formed trace keeps it well formed;
* `run_failure_no_leak` / `deploy_atomic_rollback` — a failed run leaks no
  partial write;
* `held_across_external_call_unsafe` — HZ-011: a cell held across an external
  call (take, await, put) makes the cell unavailable and loses it if the deploy
  stops, which a concrete witness exhibits;
* `external_read_then_commit_safe` — a pure read before a commit is harmless:
  the commit revalidates the cell and the read can be dropped.
-/

namespace RShare

/-- All-or-none deploy. A successful trace commits its result; a failed trace
rolls the store and held set back to the pre-state. Node atomicity is assumed
at this boundary. -/
def deploy {α : Type} (s : Store α) (h : List CellId) (ops : List (Op α)) :
    Store α × List CellId :=
  match run s h ops with
  | some (s', h') => (s', h')
  | none => (s, h)

/-- A successful commit leaves the held set untouched, so appending it to a
well-formed trace preserves well-formedness. -/
theorem commit_preserves_wf {α : Type} {s : Store α} {h : List CellId}
    {ops : List (Op α)} {c : CellId} {f : α → Option α}
    {s' s'' : Store α} {h'' : List CellId}
    (hrun : run s h ops = some (s', []))
    (hcommit : run s' [] [Op.commit c f] = some (s'', h'')) :
    WF s h (ops ++ [Op.commit c f]) := by
  have hheld : h'' = [] := commit_keeps_held hcommit
  exact ⟨s'', h'', by rw [run_append, hrun]; simpa using hcommit, hheld⟩

/-- A failed trace leaks nothing: the deploy returns the pre-state exactly. -/
theorem run_failure_no_leak {α : Type} {s : Store α} {h : List CellId}
    {ops : List (Op α)} (hfail : run s h ops = none) :
    deploy s h ops = (s, h) := by
  simp [deploy, hfail]

/-- Deploy atomicity: the result is either the unchanged pre-state or exactly
the state of a successful run. No partial store escapes. -/
theorem deploy_atomic_rollback {α : Type} (s : Store α) (h : List CellId)
    (ops : List (Op α)) :
    deploy s h ops = (s, h) ∨
      ∃ s' h', run s h ops = some (s', h') ∧ deploy s h ops = (s', h') := by
  cases hres : run s h ops with
  | none =>
    left
    simp [deploy, hres]
  | some p =>
    cases p with
    | mk s' h' =>
      right
      exact ⟨s', h', rfl, by simp [deploy, hres]⟩

/-- HZ-011 counterexample. Cell `0` holds `5`. A trace that takes the cell and
then awaits an external call holds it open: the nested call on the held cell
blocks, a concurrent commit on the emptied cell fails, and stopping the deploy
leaves the cell held, so the trace is not well formed and the value is gone.
The pattern `take; await external; put` therefore cannot be linearized by the
commit primitive. -/
theorem held_across_external_call_unsafe :
    let s0 : Store Nat := (Store.empty : Store Nat).set 0 [5]
    run s0 [] [Op.take 0] = some (s0.set 0 [], [0]) ∧
    ¬ WF s0 [] [Op.take 0] ∧
    run (s0.set 0 []) [0] [Op.call 0] = none ∧
    run (s0.set 0 []) [0] [Op.commit 0 (fun x => some x)] = none ∧
    (s0.set 0 []).cells 0 = [] := by
  dsimp
  have hcell : ((Store.empty : Store Nat).set 0 [5]).cells 0 = [5] := by simp
  refine ⟨?_, ?_, ?_, ?_, ?_⟩
  · simp [run]
  · exact take_without_put_not_wf hcell
  · simp [run]
  · simp [run]
  · simp

/-- A pure read before a commit is safe: `peek` does not change the store and
fails only on an empty cell, where the commit fails too, so the read can be
dropped. The commit revalidates the cell that it writes. -/
theorem external_read_then_commit_safe {α : Type} {s : Store α} {h : List CellId}
    {c : CellId} {f : α → Option α} :
    run s h [Op.peek c, Op.commit c f] = run s h [Op.commit c f] := by
  cases hc : s.cells c with
  | nil => simp [run, hc]
  | cons x xs => simp [run, hc]

end RShare
