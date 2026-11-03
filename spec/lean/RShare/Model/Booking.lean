import RShare.Basic

/-!
# Booking state machine

The model records the Phase 1 status graph. The Rholang arms add guards (owner, height,
claim) that are enumerated in the flow manifest; this module proves the graph properties
the guards must preserve: terminal states have no outgoing step, every state has a path to a
terminal state, and dispute/arbitration cannot bypass the graph.
-/

namespace RShare
namespace Booking

inductive BStatus where
  | requested | confirmed | checkedIn | checkedOut | redeemed
  | settled | rejected | cancelled | expired | noShow | disputed
  deriving DecidableEq, Repr

def terminal : BStatus → Bool
  | .settled | .rejected | .cancelled | .expired | .noShow => true
  | _ => false

inductive Step : BStatus → BStatus → Prop where
  | accept : Step .requested .confirmed
  | reject : Step .requested .rejected
  | cancelRequested : Step .requested .cancelled
  | cancelConfirmed : Step .confirmed .cancelled
  | expire : Step .requested .expired
  | checkIn : Step .confirmed .checkedIn
  | noShow : Step .confirmed .noShow
  | checkOut : Step .checkedIn .checkedOut
  | checkOutRedeemed : Step .redeemed .settled
  | disputeIn : Step .checkedIn .disputed
  | disputeOut : Step .checkedOut .disputed
  | settle : Step .checkedOut .settled
  | resolve : Step .disputed .settled

/-- Reflexive-transitive closure of the booking step relation. -/
inductive Reach : BStatus → BStatus → Prop where
  | refl (s : BStatus) : Reach s s
  | step {a b c : BStatus} : Step a b → Reach b c → Reach a c

theorem terminal_no_step {s t : BStatus} (h : terminal s = true)
    (hstep : Step s t) : False := by
  cases hstep <;> simp [terminal] at h

theorem terminal_sticky {s t : BStatus} (h : terminal s = true) (hr : Reach s t) :
    t = s := by
  cases hr with
  | refl => rfl
  | step hstep _ => exact False.elim (terminal_no_step h hstep)

theorem settle_once {t : BStatus} : ¬ Step .settled t := by
  intro h
  exact terminal_no_step (by rfl) h

theorem dispute_only_from_checked {s : BStatus} (h : Step s .disputed) :
    s = .checkedIn ∨ s = .checkedOut := by
  cases h with
  | disputeIn => exact Or.inl rfl
  | disputeOut => exact Or.inr rfl

theorem cancel_only_from_open {s : BStatus} (h : Step s .cancelled) :
    s = .requested ∨ s = .confirmed := by
  cases h with
  | cancelRequested => exact Or.inl rfl
  | cancelConfirmed => exact Or.inr rfl

theorem accept_only_from_requested {s : BStatus} (h : Step s .confirmed) :
    s = .requested := by
  cases h with
  | accept => rfl

theorem expire_only_from_requested {s : BStatus} (h : Step s .expired) :
    s = .requested := by
  cases h with
  | expire => rfl

/-- Every booking state is terminal or has a path to a terminal state. -/
theorem progress (s : BStatus) :
    terminal s = true ∨ ∃ t, terminal t = true ∧ Reach s t := by
  cases s with
  | settled => exact Or.inl rfl
  | rejected => exact Or.inl rfl
  | cancelled => exact Or.inl rfl
  | expired => exact Or.inl rfl
  | noShow => exact Or.inl rfl
  | requested =>
      exact Or.inr ⟨.cancelled, by rfl,
        Reach.step Step.cancelRequested (Reach.refl .cancelled)⟩
  | confirmed =>
      exact Or.inr ⟨.cancelled, by rfl,
        Reach.step Step.cancelConfirmed (Reach.refl .cancelled)⟩
  | checkedIn =>
      exact Or.inr ⟨.settled, by rfl,
        Reach.step Step.disputeIn (Reach.step Step.resolve (Reach.refl .settled))⟩
  | checkedOut =>
      exact Or.inr ⟨.settled, by rfl,
        Reach.step Step.settle (Reach.refl .settled)⟩
  | redeemed =>
      exact Or.inr ⟨.settled, by rfl,
        Reach.step Step.checkOutRedeemed (Reach.refl .settled)⟩
  | disputed =>
      exact Or.inr ⟨.settled, by rfl,
        Reach.step Step.resolve (Reach.refl .settled)⟩

end Booking
end RShare
