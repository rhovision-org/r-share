import RShare.Basic

/-!
# Hold deposit and the confirm gate

A small state machine for the per-acquire hold deposit. `acquire` requires a paid
deposit; `confirm` requires a verified funded booking; `settle` refunds only when the
listing probe reports that the hold no longer stands, and forfeits only after the hold
window. This models the deployed gates, not the Rholang refinement.
-/

namespace RShare
namespace Hold

inductive HStatus where
  | free | held | booked | paidOut
  deriving DecidableEq, Repr

structure Gate where
  depositPaid : Bool
  bookingVerified : Bool
  notHeld : Bool
  pastHoldWindow : Bool
  deriving DecidableEq, Repr

inductive Step : HStatus → Gate → HStatus → Prop where
  | acquire {g : Gate} : g.depositPaid = true → Step .free g .held
  | confirm {g : Gate} :
      g.depositPaid = true → g.bookingVerified = true → Step .held g .booked
  | release {g : Gate} : g.notHeld = true → Step .held g .free
  | refund {g : Gate} :
      g.notHeld = true → g.pastHoldWindow = false → Step .held g .paidOut
  | forfeit {g : Gate} : g.pastHoldWindow = true → Step .held g .paidOut
  | settleBooked {g : Gate} :
      g.bookingVerified = true → Step .booked g .paidOut

/-- An acquire step cannot be taken unless the deposit is paid. -/
theorem acquire_requires_deposit {g : Gate} (h : Step .free g .held) :
    g.depositPaid = true := by
  cases h with
  | acquire hpaid => exact hpaid

/-- A confirm step requires the funded-booking check; the deposit is also paid. -/
theorem confirm_requires_booking {g : Gate} (h : Step .held g .booked) :
    g.depositPaid = true ∧ g.bookingVerified = true := by
  cases h with
  | confirm hpaid hverified => exact ⟨hpaid, hverified⟩

/-- A release step cannot be taken while the listing still reports the hold. -/
theorem release_requires_no_hold {g : Gate} (h : Step .held g .free) :
    g.notHeld = true := by
  cases h with
  | release hnot => exact hnot

/-- A paid-out deposit came from a refund or a forfeit, each with its own gate. -/
theorem refund_requires_no_hold {g : Gate} (h : Step .held g .paidOut) :
    g.notHeld = true ∨ g.pastHoldWindow = true := by
  cases h with
  | refund hnot _ => exact Or.inl hnot
  | forfeit hwindow => exact Or.inr hwindow

/-- A refund inside the window is possible only when the hold no longer stands. -/
theorem refund_before_window_requires_no_hold {g : Gate}
    (h : Step .held g .paidOut) (window : g.pastHoldWindow = false) :
    g.notHeld = true := by
  cases h with
  | refund hnot _ => exact hnot
  | forfeit hwindow => exact False.elim (by simp [window] at hwindow)

/-- A paid-out deposit has no outgoing step. -/
theorem paidOut_terminal {g : Gate} {s : HStatus} : ¬ Step .paidOut g s := by
  intro h
  cases h

/-- A booked hold reached a paid-out deposit only through the booking check. -/
theorem settleBooked_requires_booking {g : Gate} (h : Step .booked g .paidOut) :
    g.bookingVerified = true := by
  cases h with
  | settleBooked hverified => exact hverified

end Hold
end RShare
