import RShare.Linear.Cell

/-!
# Concurrency over linear cells

This file lifts the sequential `run` of `Cell.lean` to trace composition and
commutation. The development rests on two observations about `Op.commit`:

* a successful commit changes exactly one cell, to a single live value, and
  leaves the held set untouched (`commit_success`, `commit_keeps_held`);
* on two distinct cells the commits are independent, so their order is
  immaterial.

From those we get:

* `run_append` — a trace splits at a concatenation point;
* `commit_commute` — commits on distinct cells commute;
* `run_disjoint_commute` — commit-only traces on disjoint cell sets commute;
* `disjoint_cells_linearizable` — permuting pairwise-disjoint commit blocks
  preserves the result, which is the linearizability statement for the commit
  primitive.

The deployed `acquire` is not a single commit (HZ-011), so this linearizability
claim is about the commit primitive, not about the deployed arm.
-/

namespace RShare

/-- A trace consists only of commit operations. -/
def AllCommits {α : Type} : List (Op α) → Prop
  | [] => True
  | Op.commit _ _ :: rest => AllCommits rest
  | _ :: _ => False

/-- The cells named by the commits in a trace. Non-commit operations are
ignored; `AllCommits` rules them out where it matters. -/
def commitCells {α : Type} : List (Op α) → List CellId
  | [] => []
  | Op.commit c _ :: rest => c :: commitCells rest
  | _ :: rest => commitCells rest

/-! ## Trace composition -/

/-- A trace splits at a concatenation point: the suffix runs from whatever
state and held set the prefix produced. -/
theorem run_append {α : Type} (s : Store α) (h : List CellId) (xs ys : List (Op α)) :
    run s h (xs ++ ys)
      = (match run s h xs with
         | some (s', h') => run s' h' ys
         | none => none) := by
  induction xs generalizing s h with
  | nil => rfl
  | cons op rest ih =>
    cases op with
    | peek c =>
      cases hc : s.cells c with
      | nil => simp [run, hc]
      | cons x xs => simp [run, hc, ih]
    | take c =>
      cases hc : s.cells c with
      | nil => simp [run, hc]
      | cons x xs =>
        cases xs with
        | nil => simp [run, hc, ih]
        | cons y ys => simp [run, hc]
    | put c v =>
      cases hc : s.cells c with
      | nil =>
        by_cases hm : c ∈ h
        · simp [run, hc, hm, ih]
        · simp [run, hc, hm]
      | cons x xs => simp [run, hc]
    | call c =>
      by_cases hm : c ∈ h
      · simp [run, hm]
      · simp [run, hm, ih]
    | commit c f =>
      cases hc : s.cells c with
      | nil => simp [run, hc]
      | cons x xs =>
        cases xs with
        | nil =>
          cases hf : f x with
          | none => simp [run, hc, hf]
          | some v' => simp [run, hc, hf, ih]
        | cons y ys => simp [run, hc]

/-- Appending the same suffix preserves an equality of runs. -/
private theorem run_append_congr {α : Type} {s : Store α} {h : List CellId}
    {A B C : List (Op α)} (hAB : run s h A = run s h B) :
    run s h (A ++ C) = run s h (B ++ C) := by
  rw [run_append, run_append, hAB]

/-! ## Commuting commits -/

/-- Setting two distinct cells is independent of the setting order. -/
private theorem store_set_comm {α : Type} (s : Store α) {c d : CellId}
    (v w : List α) (h : c ≠ d) :
    (s.set c v).set d w = (s.set d w).set c v := by
  cases s with
  | mk cells =>
    simp only [Store.set]
    congr 1
    funext x
    by_cases hx : x = c
    · subst hx
      simp [h]
    · by_cases hy : x = d
      · subst hy
        simp [hx, h]
      · simp [hx, hy]

/-- Commits on distinct cells commute: the immediate effects are independent
and neither inspects the other cell. -/
theorem commit_commute {α : Type} {s : Store α} {h : List CellId}
    {c d : CellId} {f g : α → Option α} (hne : c ≠ d) :
    run s h [Op.commit c f, Op.commit d g]
      = run s h [Op.commit d g, Op.commit c f] := by
  cases hc : s.cells c with
  | nil =>
    have hl : run s h [Op.commit c f, Op.commit d g] = none := by simp [run, hc]
    have hr : run s h [Op.commit d g, Op.commit c f] = none := by
      cases hd : s.cells d with
      | nil => simp [run, hd]
      | cons u us =>
        cases us with
        | cons v vs => simp [run, hd]
        | nil =>
          cases hg : g u with
          | none => simp [run, hd, hg]
          | some u' =>
            have hc' : (s.set d [u']).cells c = [] := by
              rw [Store.set_other s d c [u'] hne]
              exact hc
            simp [run, hd, hg, hc']
    rw [hl, hr]
  | cons x xs =>
    cases xs with
    | cons y ys =>
      have hl : run s h [Op.commit c f, Op.commit d g] = none := by simp [run, hc]
      have hr : run s h [Op.commit d g, Op.commit c f] = none := by
        cases hd : s.cells d with
        | nil => simp [run, hd]
        | cons u us =>
          cases us with
          | cons v vs => simp [run, hd]
          | nil =>
            cases hg : g u with
            | none => simp [run, hd, hg]
            | some u' =>
              have hc' : (s.set d [u']).cells c = x :: y :: ys := by
                rw [Store.set_other s d c [u'] hne]
                exact hc
              simp [run, hd, hg, hc']
      rw [hl, hr]
    | nil =>
      cases hd : s.cells d with
      | nil =>
        have hr : run s h [Op.commit d g, Op.commit c f] = none := by simp [run, hd]
        have hl : run s h [Op.commit c f, Op.commit d g] = none := by
          cases hf : f x with
          | none => simp [run, hc, hf]
          | some x' =>
            have hd' : (s.set c [x']).cells d = [] := by
              rw [Store.set_other s c d [x'] (Ne.symm hne)]
              exact hd
            simp [run, hc, hf, hd']
        rw [hl, hr]
      | cons u us =>
        cases us with
        | cons v vs =>
          have hr : run s h [Op.commit d g, Op.commit c f] = none := by simp [run, hd]
          have hl : run s h [Op.commit c f, Op.commit d g] = none := by
            cases hf : f x with
            | none => simp [run, hc, hf]
            | some x' =>
              have hd' : (s.set c [x']).cells d = u :: v :: vs := by
                rw [Store.set_other s c d [x'] (Ne.symm hne)]
                exact hd
              simp [run, hc, hf, hd']
          rw [hl, hr]
        | nil =>
          cases hf : f x with
          | none =>
            have hr : run s h [Op.commit d g, Op.commit c f] = none := by
              cases hg : g u with
              | none => simp [run, hd, hg]
              | some u' =>
                have hc' : (s.set d [u']).cells c = [x] := by
                  rw [Store.set_other s d c [u'] hne]
                  exact hc
                simp [run, hd, hg, hc', hf]
            rw [show run s h [Op.commit c f, Op.commit d g] = none by simp [run, hc, hf], hr]
          | some x' =>
            cases hg : g u with
            | none =>
              have hl : run s h [Op.commit c f, Op.commit d g] = none := by
                have hd' : (s.set c [x']).cells d = [u] := by
                  rw [Store.set_other s c d [x'] (Ne.symm hne)]
                  exact hd
                simp [run, hc, hf, hd', hg]
              rw [hl]
              simp [run, hd, hg]
            | some u' =>
              have h1 : (s.set c [x']).cells d = [u] := by
                rw [Store.set_other s c d [x'] (Ne.symm hne)]
                exact hd
              have h2 : (s.set d [u']).cells c = [x] := by
                rw [Store.set_other s d c [u'] hne]
                exact hc
              have hcomm : (s.set c [x']).set d [u'] = (s.set d [u']).set c [x'] :=
                store_set_comm s [x'] [u'] hne
              simp [run, hc, hd, hf, hg, h1, h2, hcomm]

/-- One commit on cell `c` moves unchanged past a whole commit-only trace that
avoids `c`. -/
private theorem commit_swap_past {α : Type} {c : CellId} {f : α → Option α} :
    ∀ (s : Store α) (h : List CellId) (ys : List (Op α)),
      AllCommits ys → c ∉ commitCells ys →
      run s h (ys ++ [Op.commit c f]) = run s h (Op.commit c f :: ys) := by
  intro s h ys
  induction ys generalizing s h with
  | nil => intro _ _; simp
  | cons op rest ih =>
    intro hy hdisj
    cases op with
    | commit d g =>
      have hrest : AllCommits rest := hy
      have hdne : c ≠ d := by
        intro hcd
        apply hdisj
        simp [commitCells, hcd]
      have hdrest : c ∉ commitCells rest := by
        intro hmem
        apply hdisj
        simp [commitCells, hmem]
      have hcc : run s h (Op.commit c f :: Op.commit d g :: rest)
          = run s h (Op.commit d g :: Op.commit c f :: rest) := by
        simpa using run_append_congr (C := rest)
          (commit_commute (s := s) (h := h) (c := c) (d := d) (f := f) (g := g) hdne)
      rw [hcc]
      cases hd : s.cells d with
      | nil => simp [run, hd]
      | cons u us =>
        cases us with
        | cons v vs => simp [run, hd]
        | nil =>
          cases hg : g u with
          | none => simp [run, hd, hg]
          | some u' =>
            have hih := ih (s.set d [u']) h hrest hdrest
            simp [run, hd, hg, hih]
    | peek a => simp [AllCommits] at hy
    | take a => simp [AllCommits] at hy
    | put a v => simp [AllCommits] at hy
    | call a => simp [AllCommits] at hy

/-- A trace step is congruent in its tail: if two traces have the same run from
every state, they have the same run behind a common prefix operation. -/
private theorem run_cons_congr {α : Type} {s : Store α} {h : List CellId}
    {op : Op α} {A B : List (Op α)}
    (hAB : ∀ (s' : Store α) (h' : List CellId), run s' h' A = run s' h' B) :
    run s h (op :: A) = run s h (op :: B) := by
  cases op with
  | peek c => cases hc : s.cells c <;> simp [run, hc, hAB]
  | take c =>
    cases hc : s.cells c with
    | nil => simp [run, hc]
    | cons x xs => cases xs <;> simp [run, hc, hAB]
  | put c v =>
    cases hc : s.cells c with
    | nil =>
      by_cases hm : c ∈ h
      · simp [run, hc, hm, hAB]
      · simp [run, hc, hm]
    | cons x xs => simp [run, hc]
  | call c => by_cases hm : c ∈ h <;> simp [run, hm, hAB]
  | commit c f =>
    cases hc : s.cells c with
    | nil => simp [run, hc]
    | cons x xs =>
      cases xs with
      | nil => cases hf : f x <;> simp [run, hc, hf, hAB]
      | cons y ys => simp [run, hc]

/-- A common prefix preserves pointwise equality of runs. -/
private theorem run_append_congr_right {α : Type} {A B : List (Op α)}
    (hAB : ∀ (s : Store α) (h : List CellId), run s h A = run s h B) :
    ∀ (C : List (Op α)) (s : Store α) (h : List CellId),
      run s h (C ++ A) = run s h (C ++ B) := by
  intro C
  induction C with
  | nil => intro s h; simpa using hAB s h
  | cons op C' ih =>
    intro s h
    have h : run s h (op :: (C' ++ A)) = run s h (op :: (C' ++ B)) :=
      run_cons_congr (fun s' h' => ih s' h')
    simpa [List.cons_append] using h

/-! ## Disjoint traces commute -/

/-- Commit-only traces on disjoint cell sets commute. -/
private theorem run_disjoint_commute_aux {α : Type} (xs : List (Op α))
    (hx : AllCommits xs) :
    ∀ (ys : List (Op α)) (s : Store α) (h : List CellId),
      AllCommits ys →
      (∀ c ∈ commitCells xs, c ∉ commitCells ys) →
      run s h (xs ++ ys) = run s h (ys ++ xs) := by
  intro ys
  induction ys with
  | nil => intro s h _ _; simp
  | cons op ys' ih =>
    intro s h hy hdisj
    cases op with
    | commit c f =>
      have hy' : AllCommits ys' := hy
      have hdisj' : ∀ c' ∈ commitCells xs, c' ∉ commitCells ys' := by
        intro c' hc' hmem
        exact hdisj c' hc' (by simp [commitCells, hmem])
      have hnotin : c ∉ commitCells xs := by
        intro hc
        exact hdisj c hc (by simp [commitCells])
      have step1 : run s h (xs ++ [Op.commit c f]) = run s h (Op.commit c f :: xs) :=
        commit_swap_past (c := c) (f := f) s h xs hx hnotin
      have hL : run s h (xs ++ (Op.commit c f :: ys'))
          = run s h (Op.commit c f :: (xs ++ ys')) := by
        have e1 : xs ++ (Op.commit c f :: ys') = (xs ++ [Op.commit c f]) ++ ys' := by
          simp
        rw [e1]
        have hcongr := run_append_congr (C := ys') step1
        simpa [List.cons_append] using hcongr
      have hIH : run s h (Op.commit c f :: (xs ++ ys'))
          = run s h (Op.commit c f :: (ys' ++ xs)) :=
        run_cons_congr (fun s' h' => ih s' h' hy' hdisj')
      calc
        run s h (xs ++ (Op.commit c f :: ys'))
            = run s h (Op.commit c f :: (xs ++ ys')) := hL
        _ = run s h (Op.commit c f :: (ys' ++ xs)) := hIH
        _ = run s h ((Op.commit c f :: ys') ++ xs) := by rw [List.cons_append]
    | peek a => simp [AllCommits] at hy
    | take a => simp [AllCommits] at hy
    | put a v => simp [AllCommits] at hy
    | call a => simp [AllCommits] at hy

/-- Commit-only traces on disjoint cell sets commute. -/
theorem run_disjoint_commute {α : Type} {s : Store α} {h : List CellId}
    {xs ys : List (Op α)} (hx : AllCommits xs) (hy : AllCommits ys)
    (hdisj : ∀ c ∈ commitCells xs, c ∉ commitCells ys) :
    run s h (xs ++ ys) = run s h (ys ++ xs) :=
  run_disjoint_commute_aux xs hx ys s h hy hdisj

/-! ## Linearizability of disjoint commit blocks -/

/-- Two commit blocks are disjoint when no cell committed by the first is
committed by the second. -/
def BlockDisjoint {α : Type} (a b : List (Op α)) : Prop :=
  ∀ c ∈ commitCells a, c ∉ commitCells b

/-- A block list is good when every block is commit-only and the blocks are
pairwise disjoint. -/
def GoodBlocks {α : Type} (bs : List (List (Op α))) : Prop :=
  (∀ b ∈ bs, AllCommits b) ∧ bs.Pairwise (BlockDisjoint (α := α))

private theorem blockDisjoint_symm {α : Type} {a b : List (Op α)} :
    BlockDisjoint a b → BlockDisjoint b a := by
  intro h c hc hca
  exact h c hca hc

/-- Goodness is invariant under permuting the block list. -/
private theorem goodBlocks_perm {α : Type} {bs bs' : List (List (Op α))}
    (hperm : bs.Perm bs') (hok : GoodBlocks (α := α) bs) :
    GoodBlocks (α := α) bs' := by
  rcases hok with ⟨hall, hpair⟩
  constructor
  · intro b hb
    exact hall b (hperm.mem_iff.mpr hb)
  · exact hpair.perm hperm (fun {x y} h => blockDisjoint_symm h)

/-- Two disjoint commit blocks commute as blocks. -/
private theorem block_swap {α : Type} {s : Store α} {h : List CellId}
    {a b : List (Op α)} (ha : AllCommits a) (hb : AllCommits b)
    (hd : BlockDisjoint a b) :
    run s h (a ++ b) = run s h (b ++ a) :=
  run_disjoint_commute ha hb hd

private theorem linearizable_aux {α : Type} (blocks blocks' : List (List (Op α)))
    (hperm : blocks.Perm blocks') :
    ∀ (s : Store α) (h : List CellId), GoodBlocks (α := α) blocks →
      run s h blocks.join = run s h blocks'.join := by
  induction hperm with
  | nil => intro s h _; rfl
  | cons b _h ih =>
    intro s h hok
    rcases hok with ⟨hall, hpair⟩
    have hok₁ : GoodBlocks (α := α) _ :=
      ⟨fun x hx => hall x (List.mem_cons_of_mem b hx), hpair.tail⟩
    have ih' := fun (s' : Store α) (h' : List CellId) => ih s' h' hok₁
    simpa [List.join_cons] using run_append_congr_right ih' b s h
  | swap x y l =>
    intro s h hok
    rcases hok with ⟨hall, hpair⟩
    have hy : AllCommits y := hall y (by simp)
    have hx : AllCommits x := hall x (by simp)
    have hdisj : BlockDisjoint y x := by
      have hpair' := hpair
      simp only [List.pairwise_cons] at hpair'
      exact hpair'.1 x (by simp)
    have hswap : run s h (y ++ x) = run s h (x ++ y) :=
      block_swap hy hx hdisj
    have hcongr := run_append_congr (C := l.join) hswap
    simpa [List.join_cons, List.append_assoc] using hcongr
  | trans h₁ _h₂ ih₁ ih₂ =>
    intro s h hok
    have hok₂ : GoodBlocks (α := α) _ := goodBlocks_perm h₁ hok
    exact (ih₁ s h hok).trans (ih₂ s h hok₂)

/-- Permuting pairwise-disjoint commit blocks preserves the result: the block
order is unobservable by the commit primitive. -/
theorem disjoint_cells_linearizable {α : Type} {s : Store α} {h : List CellId}
    {blocks blocks' : List (List (Op α))}
    (hperm : blocks.Perm blocks') (hok : GoodBlocks (α := α) blocks) :
    run s h blocks.join = run s h blocks'.join :=
  linearizable_aux blocks blocks' hperm s h hok

end RShare
