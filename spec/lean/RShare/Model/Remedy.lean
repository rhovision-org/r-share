import RShare.Basic

/-!
# Remedy arithmetic

The Phase 1 remedy classes are pure functions from terms and the terminal event to a
settlement. The bucket identity is the theorem that every class conserves `rent + deposit`.
-/

namespace RShare
namespace Remedy

structure Settlement where
  providerRent : Nat
  platformFee : Nat
  guestRefund : Nat
  providerDamage : Nat
  deriving Repr

def total (s : Settlement) : Nat :=
  s.providerRent + s.platformFee + s.guestRefund + s.providerDamage

def fee (rent bps : Nat) : Nat := rent * bps / Bps

def fullRefund (rent deposit : Nat) : Settlement :=
  { providerRent := 0, platformFee := 0, guestRefund := rent + deposit,
    providerDamage := 0 }

def feeSettle (rent deposit bps : Nat) : Settlement :=
  let f := fee rent bps
  { providerRent := rent - f, platformFee := f, guestRefund := deposit,
    providerDamage := 0 }

def noShow (rent deposit bps : Nat) : Settlement :=
  feeSettle rent deposit bps

def depositSplit (deposit bps : Nat) : Settlement :=
  let damage := deposit * bps / Bps
  { providerRent := 0, platformFee := 0, guestRefund := deposit - damage,
    providerDamage := damage }

theorem fullRefund_conserves (rent deposit : Nat) :
    total (fullRefund rent deposit) = rent + deposit := by
  simp [total, fullRefund]

theorem fee_le_rent (rent bps : Nat) (h : bps ≤ Bps) : fee rent bps ≤ rent := by
  unfold fee
  apply Nat.div_le_of_le_mul
  calc rent * bps ≤ rent * Bps := Nat.mul_le_mul_left rent h
    _ = Bps * rent := Nat.mul_comm rent Bps

theorem feeSettle_conserves (rent deposit bps : Nat) (h : bps ≤ Bps) :
    total (feeSettle rent deposit bps) = rent + deposit := by
  have hf : fee rent bps ≤ rent := fee_le_rent rent bps h
  unfold total feeSettle
  simp only
  omega

theorem noShow_conserves (rent deposit bps : Nat) (h : bps ≤ Bps) :
    total (noShow rent deposit bps) = rent + deposit :=
  feeSettle_conserves rent deposit bps h

theorem depositSplit_conserves (deposit bps : Nat) (h : bps ≤ Bps) :
    total (depositSplit deposit bps) = deposit := by
  have hd : deposit * bps / Bps ≤ deposit := by
    apply Nat.div_le_of_le_mul
    calc deposit * bps ≤ deposit * Bps := Nat.mul_le_mul_left deposit h
      _ = Bps * deposit := Nat.mul_comm deposit Bps
  unfold total depositSplit
  simp only
  omega

end Remedy
end RShare
