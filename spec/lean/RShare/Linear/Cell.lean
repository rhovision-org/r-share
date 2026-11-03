import RShare.Basic

/-!
# Linear state cells

A cell holds at most one value. `peek` reads without changing it; `take` empties
it and records it as held; `put` requires the cell empty and held; `call` is a
nested deployment step and may not touch a held cell.

`commit` is the cornerstone primitive for the allocator: one operation reads the
single live value, applies a pure function, and writes the result back in place.
It is one comm event, not a `take` followed by a `put`, so there is no observable
state in which the cell is empty.

The defect classes are rejection theorems:

* D1 `peek_then_put_rejected` — a post without a take leaves two cells;
* D2 `take_without_put_not_wf` — a graceful stop after a take loses the cell;
* D3 `nested_call_rejected` — a nested call on a held cell blocks.
-/

namespace RShare

abbrev CellId := Nat

structure Store (α : Type) where
  cells : CellId → List α

namespace Store

variable {α : Type}

def empty : Store α := ⟨fun _ => []⟩

def set (s : Store α) (c : CellId) (v : List α) : Store α :=
  ⟨fun x => if x = c then v else s.cells x⟩

@[simp] theorem set_same (s : Store α) (c : CellId) (v : List α) :
    (s.set c v).cells c = v := by
  simp [set]

theorem set_other (s : Store α) (c d : CellId) (v : List α) (h : d ≠ c) :
    (s.set c v).cells d = s.cells d := by
  simp [set, h]

end Store

inductive Op (α : Type) where
  | peek (c : CellId)
  | take (c : CellId)
  | put (c : CellId) (v : α)
  | call (c : CellId)
  | commit (c : CellId) (f : α → Option α)

/-- Run a trace. A failure is `none`; a graceful stop returns the held set. -/
def run {α : Type} : Store α → List CellId → List (Op α) → Option (Store α × List CellId)
  | s, h, [] => some (s, h)
  | s, h, Op.peek c :: rest =>
      match s.cells c with
      | [] => none
      | _ => run s h rest
  | s, h, Op.take c :: rest =>
      match s.cells c with
      | [_] => run (s.set c []) (c :: h) rest
      | _ => none
  | s, h, Op.put c v :: rest =>
      match s.cells c with
      | [] => if c ∈ h then run (s.set c [v]) (h.erase c) rest else none
      | _ => none
  | s, h, Op.call c :: rest =>
      if c ∈ h then none else run s h rest
  | s, h, Op.commit c f :: rest =>
      match s.cells c with
      | [v] =>
          match f v with
          | some v' => run (s.set c [v']) h rest
          | none => none
      | _ => none

/-- A trace is well formed when it stops with nothing held. -/
def WF {α : Type} (s : Store α) (h : List CellId) (ops : List (Op α)) : Prop :=
  ∃ s' h', run s h ops = some (s', h') ∧ h' = []

/-! ## Rejection theorems -/

/-- D1: peeking then posting the same cell is not a committed update. -/
theorem peek_then_put_rejected {α : Type} {s : Store α} {h : List CellId}
    {c : CellId} {v : α} (hne : s.cells c ≠ []) :
    run s h [Op.peek c, Op.put c v] = none := by
  cases hc : s.cells c with
  | nil => exact absurd hc hne
  | cons x xs => simp [run, hc]

/-- D1: a post with no take at all is rejected. -/
theorem put_without_take_rejected {α : Type} {s : Store α} {h : List CellId}
    {c : CellId} {v : α} (hempty : s.cells c = []) (hnot : c ∉ h) :
    run s h [Op.put c v] = none := by
  simp [run, hempty, hnot]

/-- D2: a graceful stop after a take leaves the cell held. -/
theorem take_without_put_not_wf {α : Type} {s : Store α} {h : List CellId}
    {c : CellId} {v : α} (hcell : s.cells c = [v]) : ¬ WF s h [Op.take c] := by
  intro hwf
  rcases hwf with ⟨s', h', hrun, hstop⟩
  simp [run, hcell] at hrun
  rcases hrun with ⟨rfl, rfl⟩
  simp at hstop

/-- D3: a nested call on a held cell is blocked. -/
theorem nested_call_rejected {α : Type} {s : Store α} {h : List CellId}
    {c : CellId} (hheld : c ∈ h) : run s h [Op.call c] = none := by
  simp [run, hheld]

/-- A take then a put is the committed update. -/
theorem take_put_commit {α : Type} {s : Store α} {h : List CellId}
    {c : CellId} {v : α} (hcell : s.cells c = [v]) (hnot : c ∉ h) :
    run s h [Op.take c, Op.put c v]
      = some ((s.set c []).set c [v], h.erase c) := by
  have hset : (s.set c []).cells c = [] := by simp
  simp [run, hcell, hnot, hset]
  rw [List.erase_of_not_mem hnot]

/-- I1: an empty trace is quiescent. -/
theorem wf_quiescent {α : Type} (s : Store α) : WF s [] [] :=
  ⟨s, [], rfl, rfl⟩

/-! ## Commit lemmas -/

/-- A successful commit keeps the held set unchanged: it never exposes an empty cell. -/
theorem commit_keeps_held {α : Type} {s : Store α} {h : List CellId}
    {c : CellId} {f : α → Option α} {s' h'}
    (hrun : run s h [Op.commit c f] = some (s', h')) : h' = h := by
  cases hc : s.cells c with
  | nil => simp [run, hc] at hrun
  | cons v rest =>
      cases rest with
      | nil =>
          cases hf : f v with
          | none => simp [run, hc, hf] at hrun
          | some v' =>
              simp [run, hc, hf] at hrun
              exact hrun.2.symm
      | cons w ws => simp [run, hc] at hrun

/-- A successful commit replaces exactly one live value. -/
theorem commit_success {α : Type} {s : Store α} {h : List CellId}
    {c : CellId} {f : α → Option α} {s' h'}
    (hrun : run s h [Op.commit c f] = some (s', h')) :
    ∃ v v', s.cells c = [v] ∧ f v = some v' ∧ s' = s.set c [v'] ∧ h' = h := by
  cases hc : s.cells c with
  | nil => simp [run, hc] at hrun
  | cons v rest =>
      cases rest with
      | nil =>
          cases hf : f v with
          | none => simp [run, hc, hf] at hrun
          | some v' =>
              simp [run, hc, hf] at hrun
              rcases hrun with ⟨hs, hh⟩
              exact ⟨v, v', rfl, hf, hs.symm, hh.symm⟩
      | cons w ws => simp [run, hc] at hrun

/-- A commit either produces exactly one new value or fails; there is no empty
intermediate value. -/
theorem commit_all_or_none {α : Type} {s : Store α} {h : List CellId}
    {c : CellId} {f : α → Option α} :
    run s h [Op.commit c f] = none ∨
      ∃ v v', s.cells c = [v] ∧ f v = some v' ∧
        run s h [Op.commit c f] = some (s.set c [v'], h) := by
  cases hc : s.cells c with
  | nil => exact Or.inl (by simp [run, hc])
  | cons v rest =>
      cases rest with
      | nil =>
          cases hf : f v with
          | none => exact Or.inl (by simp [run, hc, hf])
          | some v' => exact Or.inr ⟨v, v', rfl, hf, by simp [run, hc, hf]⟩
      | cons w ws => exact Or.inl (by simp [run, hc])

/-- A commit requires a single live value. -/
theorem commit_requires_single {α : Type} {s : Store α} {h : List CellId}
    {c : CellId} {f : α → Option α} {s' h'}
    (hrun : run s h [Op.commit c f] = some (s', h')) : s.cells c ≠ [] := by
  rcases commit_success hrun with ⟨v, v', hcell, _⟩
  rw [hcell]
  simp

end RShare
