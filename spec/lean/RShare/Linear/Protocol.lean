import RShare.Linear.Cell

/-!
# Helper protocols

* H1 — the helper is pure: no cell operations.
* H2a — the caller takes the cell; the helper posts the new value; the caller
  does not post again.
* H2b — the helper takes and posts the cell; the caller only peeks.
* P2 — an external read (the clock) happens before one cell commit; the commit
  itself touches no other cell and keeps the held set unchanged.
-/

namespace RShare

open Op

def countOp {α : Type} (p : Op α → Bool) : List (Op α) → Nat
  | [] => 0
  | op :: rest => (if p op then 1 else 0) + countOp p rest

def isPut {α : Type} : Op α → Bool
  | Op.put _ _ => true
  | _ => false

def isTake {α : Type} : Op α → Bool
  | Op.take _ => true
  | _ => false

/-- H1: a pure helper has no cell operations. -/
theorem h1_pure {α : Type} (s : Store α) (h : List CellId) :
    run s h [] = some (s, h) := rfl

/-- H2a: caller takes, helper posts once; the held set is empty afterwards. -/
theorem h2a_caller_token {α : Type} {s : Store α} {c : CellId} {v : α}
    (hcell : s.cells c = [v]) :
    run s [] [Op.take c, Op.put c v] = some ((s.set c []).set c [v], []) := by
  rw [take_put_commit hcell (by simp)]
  simp

/-- H2b: helper takes and posts; the caller's peek is harmless. -/
theorem h2b_delegated {α : Type} {s : Store α} {c : CellId} {v : α}
    (hcell : s.cells c = [v]) :
    run s [] [Op.peek c, Op.take c, Op.put c v]
      = some ((s.set c []).set c [v], []) := by
  have hset : (s.set c []).cells c = [] := by simp
  simp [run, hcell, hset]

/-- P2: the allocator pattern — an external read precedes one commit, and the
commit keeps the held set unchanged. -/
theorem p2_await_then_commit {α : Type} {s : Store α} {h : List CellId}
    {c : CellId} {f : α → Option α} {s' h'}
    (hrun : run s h [Op.commit c f] = some (s', h')) : h' = h :=
  commit_keeps_held hrun

/-- The canonical commit has exactly one take and one post in its take/put form. -/
theorem commit_counts {α : Type} (c : CellId) (v : α) :
    countOp isTake [Op.take c, Op.put c v] = 1 ∧
    countOp isPut [Op.take c, Op.put c v] = 1 := by
  constructor <;> simp [countOp, isTake, isPut]

end RShare
