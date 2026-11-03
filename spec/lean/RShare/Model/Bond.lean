import RShare.Basic

/-!
# Bond

A Bond is a single-use escrow. `pay` consumes the state, checks the settle token and the
bucket sum, and closes. The bucket sum is the conservation obligation.
-/

namespace RShare
namespace Bond

structure Bucket where
  owner : Nat
  amount : Nat
  deriving Repr

structure BondState where
  paid : Bool
  expected : Nat
  remaining : Nat
  deriving Repr

def sumBuckets : List Bucket → Nat
  | [] => 0
  | b :: rest => b.amount + sumBuckets rest

def pay (b : BondState) (buckets : List Bucket) (expected : Nat)
    (settleToken suppliedToken : Nat) : Option BondState :=
  if b.paid then none
  else if suppliedToken ≠ settleToken then none
  else if expected ≠ b.expected then none
  else if sumBuckets buckets ≠ expected then none
  else some { paid := true, expected := b.expected, remaining := 0 }

theorem pay_once {b : BondState} {buckets : List Bucket} {expected token supplied}
    (h : b.paid = true) :
    pay b buckets expected token supplied = none := by
  simp [pay, h]

theorem pay_requires_token {b : BondState} {buckets : List Bucket} {expected token supplied}
    (h : supplied ≠ token) :
    pay b buckets expected token supplied = none := by
  simp [pay, h]

theorem pay_requires_expected {b : BondState} {buckets : List Bucket} {expected token supplied}
    (h : expected ≠ b.expected) :
    pay b buckets expected token supplied = none := by
  simp [pay, h]

theorem pay_requires_sum {b : BondState} {buckets : List Bucket} {expected token supplied}
    (h : sumBuckets buckets ≠ expected) :
    pay b buckets expected token supplied = none := by
  simp [pay, h]

theorem pay_closes {b : BondState} {buckets : List Bucket} {expected token supplied}
    {b' : BondState}
    (h : pay b buckets expected token supplied = some b') :
    b'.paid = true ∧ b'.remaining = 0 := by
  by_cases h1 : b.paid = true
  · simp [pay, h1] at h
  · by_cases h2 : supplied ≠ token
    · simp [pay, h1, h2] at h
    · by_cases h3 : expected ≠ b.expected
      · simp [pay, h1, h2, h3] at h
      · by_cases h4 : sumBuckets buckets ≠ expected
        · simp [pay, h1, h2, h3, h4] at h
        · simp [pay, h1, h2, h3, h4] at h
          rcases h with rfl
          simp

theorem pay_conserves {b : BondState} {buckets : List Bucket} {expected token supplied}
    {b' : BondState}
    (h : pay b buckets expected token supplied = some b') :
    sumBuckets buckets = expected := by
  by_cases h1 : b.paid = true
  · simp [pay, h1] at h
  · by_cases h2 : supplied ≠ token
    · simp [pay, h1, h2] at h
    · by_cases h3 : expected ≠ b.expected
      · simp [pay, h1, h2, h3] at h
      · by_cases h4 : sumBuckets buckets ≠ expected
        · simp [pay, h1, h2, h3, h4] at h
        · by_cases hsum : sumBuckets buckets = expected
          · exact hsum
          · exact False.elim (h4 hsum)

theorem bucket_nonneg (buckets : List Bucket) : ∀ b ∈ buckets, 0 ≤ b.amount := by
  intro b _
  exact Nat.zero_le b.amount

end Bond
end RShare
