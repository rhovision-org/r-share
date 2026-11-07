import RShare.Basic
import RShare.Model.Booking

/-!
# Two-sided cover

Minimal cover state machine aligned with the deployed `cover-demo` fields: one
policy, one booking binding, the booking's embedded claim, and the pool entry.
The model keeps the guards that make the claim token single-use and the payout
ordering visible; it does not formalise purses, pricing or Rholang syntax.

The safety properties are stated over the transition relation `step` and its
reflexive-transitive closure `Reach`, with an invariant `Inv` that records the
paid/attached linkage.
-/

namespace RShare
namespace Cover

abbrev ClaimId := Nat
abbrev LossId := Nat
abbrev PolicyId := Nat

/-- Policy lifecycle: attached is the exercisable state in the deployed
`cover-demo` profile; settled and expired are terminal. -/
inductive PStatus where
  | active | attached | settled | expired
  deriving DecidableEq, Repr

/-- Binding lifecycle carried by the booking's cover cell. -/
inductive CStatus where
  | none | pending | attached | closed
  deriving DecidableEq, Repr

/-- The operator-authorized fixed claim token. -/
structure Token where
  claimId : ClaimId
  lossId : LossId
  amount : Amount
  assessed : Amount
  nonCoverRecovery : Amount
  evidenceHash : Nat
  deriving DecidableEq, Repr

/-- The booking's embedded deposit claim. -/
structure Claim where
  amount : Amount
  evidenceHash : Nat
  atHeight : Height
  deriving DecidableEq, Repr

/-- Policy accounting. `claims` is the paid-claim ledger used for idempotence. -/
structure Policy where
  status : PStatus
  cap : Amount
  paid : Amount
  expiresAtHeight : Height
  token : Option Token
  claims : List (ClaimId × Amount)
  deriving Repr

/-- The per-booking cover binding. -/
structure Binding where
  status : CStatus
  policyId : PolicyId
  claimId : Option ClaimId
  depositPart : Amount
  coverPart : Amount
  effectiveDeposit : Amount
  deductible : Amount
  cap : Amount
  deriving Repr

/-- Booking fields the cover flow reads and writes. -/
structure BookingState where
  status : Booking.BStatus
  claim : Option Claim
  deriving Repr

/-- Whole cover/booking slice. `pool` is the facility entry for this policy;
`none` means it has been removed (for example by `closePool`). -/
structure State where
  policy : Policy
  binding : Binding
  booking : BookingState
  pool : Option (PolicyId × Amount)
  deriving Repr

def tokenMatches (s : State) (claimId : ClaimId) : Bool :=
  match s.policy.token with
  | some t => t.claimId == claimId
  | none => false

def poolHas (s : State) : Bool :=
  match s.pool with
  | some p => p.1 == s.binding.policyId
  | none => false

def hasClaim (s : State) (claimId : ClaimId) : Bool :=
  s.policy.claims.any (fun p => p.1 == claimId)

/-- Authorize only on an attached policy with no token already fixed and
within the remaining cap. -/
abbrev authorizeOK (s : State) (t : Token) : Prop :=
  s.policy.status = .attached ∧
  s.policy.token = none ∧
  s.policy.paid + t.amount ≤ s.policy.cap

/-- Attach only with the stored token, the matching claim/loss/amount, an
attached binding with no prior claim, and a claim already recorded on the
booking. -/
abbrev attachOK (s : State) (t : Token) (claimId : ClaimId) (lossId : LossId)
    (depositPart coverPart : Amount) : Prop :=
  s.policy.status = .attached ∧
  s.policy.token = some t ∧
  t.claimId = claimId ∧
  t.lossId = lossId ∧
  t.amount = coverPart ∧
  depositPart ≤ s.binding.effectiveDeposit ∧
  s.binding.status = .attached ∧
  s.binding.claimId = none ∧
  s.booking.claim.isSome = true

/-- Pay only an attached binding whose claim matches the authorized token,
while the policy is attached, the pool entry exists and the policy is not
past its expiry height. -/
abbrev payOK (s : State) (claimId : ClaimId) (now : Height) : Prop :=
  s.binding.status = .attached ∧
  s.binding.claimId = some claimId ∧
  s.policy.status = .attached ∧
  tokenMatches s claimId = true ∧
  poolHas s = true ∧
  now ≤ s.policy.expiresAtHeight

abbrev claimDepositOK (s : State) : Prop :=
  s.booking.status = .checkedOut ∧
  s.booking.claim = none ∧
  s.binding.claimId = none

abbrev disputeOK (s : State) : Prop :=
  s.booking.claim = none ∧
  s.binding.claimId = none ∧
  (s.booking.status = .checkedIn ∨ s.booking.status = .checkedOut)

def authorize (s : State) (t : Token) : Option State :=
  if authorizeOK s t then
    some { s with policy := { s.policy with token := some t } }
  else none

def attach (s : State) (t : Token) (claimId : ClaimId) (lossId : LossId)
    (depositPart coverPart : Amount) : Option State :=
  if attachOK s t claimId lossId depositPart coverPart then
    some { s with binding := { s.binding with
      claimId := some claimId
      depositPart := depositPart
      coverPart := coverPart } }
  else none

/-- Pay is idempotent per claim id: a claim already in the ledger returns the
state unchanged and therefore never increments `paid` twice. -/
def pay (s : State) (claimId : ClaimId) (now : Height) : Option State :=
  if payOK s claimId now then
    if hasClaim s claimId then
      some s
    else if s.policy.paid + s.binding.coverPart ≤ s.policy.cap then
      some { s with policy := { s.policy with
        paid := s.policy.paid + s.binding.coverPart
        claims := (claimId, s.binding.coverPart) :: s.policy.claims } }
    else none
  else none

/-- Expiry refuses an attached policy; terminal states are a no-op; otherwise
the policy may expire once the height is reached. -/
def expire (s : State) (now : Height) : Option State :=
  if s.policy.status = .settled ∨ s.policy.status = .expired then
    some s
  else if s.policy.status = .attached then
    none
  else if s.policy.expiresAtHeight ≤ now then
    some { s with policy := { s.policy with status := .expired } }
  else none

def closePool (s : State) : State :=
  { s with pool := none }

def claimDeposit (s : State) (c : Claim) (now : Height) : Option State :=
  if claimDepositOK s then
    some { s with booking :=
      { s.booking with claim := some { c with atHeight := now } } }
  else none

def dispute (s : State) : Option State :=
  if disputeOK s then
    some { s with booking := { s.booking with status := .disputed } }
  else none

/-- Terminal settle clears the pool, settles the booking, closes the binding
and settles the policy. It deliberately ignores whether a pool entry existed. -/
def terminalSettle (s : State) : State :=
  { s with
    policy := { s.policy with status := .settled }
    binding := { s.binding with status := .closed }
    booking := { s.booking with status := .settled }
    pool := none }

inductive Action where
  | authorize (t : Token)
  | attach (t : Token) (claimId lossId : ClaimId) (depositPart coverPart : Amount)
  | pay (claimId : ClaimId) (now : Height)
  | expire (now : Height)
  | closePool
  | claimDeposit (c : Claim) (now : Height)
  | dispute
  | settle
  deriving Repr

def step (s : State) : Action → Option State
  | .authorize t => authorize s t
  | .attach t c l d cp => attach s t c l d cp
  | .pay c now => pay s c now
  | .expire now => expire s now
  | .closePool => some (closePool s)
  | .claimDeposit c now => claimDeposit s c now
  | .dispute => dispute s
  | .settle => some (terminalSettle s)

/-- A reachable cover state. The recursive tail is on the left so induction
yields the invariant at the predecessor. -/
inductive Reach : State → State → Prop where
  | refl (s : State) : Reach s s
  | step {a b c : State} {act : Action} :
      Reach a b → step b act = some c → Reach a c

/-- A cover claim that has been selected but not yet attached. -/
def PendingClaim (s : State) : Prop :=
  s.binding.status = .pending

def authorized (s : State) : Prop :=
  ∃ t, s.policy.token = some t

def attached (s : State) : Prop :=
  ∃ c, s.binding.claimId = some c

/-- The reachable-state invariant that ties a payout to a fixed token and an
attached claim. -/
structure Inv (s : State) : Prop where
  pending_policy : s.binding.status = .pending → s.policy.status = .attached
  paid_cap : s.policy.paid ≤ s.policy.cap
  paid_token : 0 < s.policy.paid → authorized s
  paid_claim : 0 < s.policy.paid → attached s

/-- A state in which authorize has a token to fix, attach has a binding to
attach, claimDeposit and pay have a booking and pool to act on. -/
def init : State :=
  { policy :=
      { status := .attached
        cap := 1000000
        paid := 0
        expiresAtHeight := 1000000
        token := none
        claims := [] }
    binding :=
      { status := .attached
        policyId := 0
        claimId := none
        depositPart := 0
        coverPart := 0
        effectiveDeposit := 0
        deductible := 0
        cap := 0 }
    booking := { status := .checkedOut, claim := none }
    pool := some (0, 0) }

/-! ## Guard extraction -/

private theorem authorize_success {s s' : State} {t : Token}
    (h : step s (.authorize t) = some s') :
    authorizeOK s t ∧
      s' = { s with policy := { s.policy with token := some t } } := by
  simp only [step, authorize] at h
  split at h
  · rename_i hg
    simp only [Option.some.injEq] at h
    exact ⟨hg, h.symm⟩
  · simp at h

private theorem attach_success {s s' : State} {t : Token}
    {claimId : ClaimId} {lossId : LossId} {depositPart coverPart : Amount}
    (h : step s (.attach t claimId lossId depositPart coverPart) = some s') :
    attachOK s t claimId lossId depositPart coverPart ∧
      s' = { s with binding := { s.binding with
        claimId := some claimId
        depositPart := depositPart
        coverPart := coverPart } } := by
  simp only [step, attach] at h
  split at h
  · rename_i hg
    simp only [Option.some.injEq] at h
    exact ⟨hg, h.symm⟩
  · simp at h

private theorem tokenMatches_spec {s : State} {claimId : ClaimId}
    (h : tokenMatches s claimId = true) :
    ∃ t, s.policy.token = some t ∧ t.claimId = claimId := by
  cases htok : s.policy.token with
  | none => simp [tokenMatches, htok] at h
  | some t => exact ⟨t, rfl, by simpa [tokenMatches, htok] using h⟩

private theorem poolHas_spec {s : State} (h : poolHas s = true) :
    ∃ p : PolicyId × Amount, s.pool = some p ∧ p.1 = s.binding.policyId := by
  cases hp : s.pool with
  | none => simp [poolHas, hp] at h
  | some p => exact ⟨p, rfl, by simpa [poolHas, hp] using h⟩

private theorem payOK_of_step {s s' : State} {claimId : ClaimId} {now : Height}
    (h : step s (.pay claimId now) = some s') : payOK s claimId now := by
  simp only [step, pay] at h
  split at h
  · assumption
  · simp at h

private theorem paidClaim_of_step {s s' : State} {claimId : ClaimId} {now : Height}
    (h : step s (.pay claimId now) = some s') :
    payOK s claimId now ∧ hasClaim s' claimId = true := by
  simp only [step, pay] at h
  split at h
  · rename_i hok
    split at h
    · rename_i hcl
      simp only [Option.some.injEq] at h
      subst s'
      exact ⟨hok, hcl⟩
    · split at h
      · rename_i hcap
        simp only [Option.some.injEq] at h
        subst s'
        refine ⟨?_, ?_⟩
        · refine ⟨?_, ?_, ?_, ?_, ?_, ?_⟩
          · simpa using hok.1
          · simpa using hok.2.1
          · simpa using hok.2.2.1
          · simpa [tokenMatches] using hok.2.2.2.1
          · simpa [poolHas] using hok.2.2.2.2.1
          · simpa using hok.2.2.2.2.2
        · simp [hasClaim]
      · simp at h
  · simp at h

private theorem pay_success {s s' : State} {claimId : ClaimId} {now : Height}
    (h : step s (.pay claimId now) = some s') :
    payOK s claimId now ∧
      (s' = s ∨
        (s.policy.paid + s.binding.coverPart ≤ s.policy.cap ∧
          s' = { s with policy := { s.policy with
            paid := s.policy.paid + s.binding.coverPart
            claims := (claimId, s.binding.coverPart) :: s.policy.claims } })) := by
  simp only [step, pay] at h
  split at h
  · rename_i hok
    split at h
    · rename_i hcl
      simp only [Option.some.injEq] at h
      exact ⟨hok, Or.inl h.symm⟩
    · split at h
      · rename_i hcap
        simp only [Option.some.injEq] at h
        exact ⟨hok, Or.inr ⟨hcap, h.symm⟩⟩
      · simp at h
  · simp at h

private theorem payOK_next {s s' : State} {claimId : ClaimId} {now : Height}
    (h : step s (.pay claimId now) = some s') : payOK s' claimId now := by
  have ⟨hok, hcases⟩ := pay_success h
  rcases hcases with hEq | ⟨_, hEq⟩
  · subst s'
    exact hok
  · subst s'
    refine ⟨?_, ?_, ?_, ?_, ?_, ?_⟩
    · simpa using hok.1
    · simpa using hok.2.1
    · simpa using hok.2.2.1
    · simpa [tokenMatches] using hok.2.2.2.1
    · simpa [poolHas] using hok.2.2.2.2.1
    · simpa using hok.2.2.2.2.2

private theorem pay_replay {s : State} {claimId : ClaimId} {now : Height}
    (hok : payOK s claimId now) (hcl : hasClaim s claimId = true) :
    pay s claimId now = some s := by
  simp [pay, hok, hcl]

/-! ## Invariant preservation -/

private theorem init_inv : Inv init := by
  refine ⟨?_, ?_, ?_, ?_⟩
  · intro hp
    simp [init] at hp
  · simp [init]
  · intro hp
    simp [init] at hp
  · intro hp
    simp [init] at hp

private theorem step_inv {s s' : State} {a : Action}
    (hinv : Inv s) (h : step s a = some s') : Inv s' := by
  cases a with
  | authorize t =>
      have ⟨hg, hs'⟩ := authorize_success h
      subst s'
      refine ⟨?_, ?_, ?_, ?_⟩
      · intro hpend
        have hpend' : s.binding.status = .pending := by simpa using hpend
        have hpol := hinv.pending_policy hpend'
        simpa using hpol
      · simpa using hinv.paid_cap
      · intro hpaid
        have hpaid' : 0 < s.policy.paid := by simpa using hpaid
        obtain ⟨tk, htk⟩ := hinv.paid_token hpaid'
        rw [hg.2.1] at htk
        exact Option.noConfusion htk
      · intro hpaid
        have hpaid' : 0 < s.policy.paid := by simpa using hpaid
        simpa using hinv.paid_claim hpaid'
  | attach t c l d cp =>
      have ⟨_, hs'⟩ := attach_success h
      subst s'
      refine ⟨?_, ?_, ?_, ?_⟩
      · intro hpend
        have hpend' : s.binding.status = .pending := by simpa using hpend
        have hpol := hinv.pending_policy hpend'
        simpa using hpol
      · simpa using hinv.paid_cap
      · intro hpaid
        have hpaid' : 0 < s.policy.paid := by simpa using hpaid
        simpa using hinv.paid_token hpaid'
      · intro _
        exact ⟨c, rfl⟩
  | pay c now =>
      have ⟨hok, hcases⟩ := pay_success h
      rcases hcases with hEq | ⟨hcap, hEq⟩
      · subst s'
        exact hinv
      · subst s'
        obtain ⟨tk, htk, _⟩ := tokenMatches_spec hok.2.2.2.1
        refine ⟨?_, ?_, ?_, ?_⟩
        · intro hpend
          have hpend' : s.binding.status = .pending := by simpa using hpend
          have hpol := hinv.pending_policy hpend'
          simpa using hpol
        · simpa using hcap
        · intro _
          exact ⟨tk, by simpa using htk⟩
        · intro _
          exact ⟨c, by simpa using hok.2.1⟩
  | expire now =>
      simp only [step, expire] at h
      split at h
      · rename_i hterm
        simp only [Option.some.injEq] at h
        subst s'
        exact hinv
      · split at h
        · simp at h
        · split at h
          · rename_i hnotatt htime
            simp only [Option.some.injEq] at h
            subst s'
            refine ⟨?_, ?_, ?_, ?_⟩
            · intro hpend
              have hpend' : s.binding.status = .pending := by simpa using hpend
              exact False.elim (hnotatt (hinv.pending_policy hpend'))
            · simpa using hinv.paid_cap
            · intro hpaid
              have hpaid' : 0 < s.policy.paid := by simpa using hpaid
              simpa using hinv.paid_token hpaid'
            · intro hpaid
              have hpaid' : 0 < s.policy.paid := by simpa using hpaid
              simpa using hinv.paid_claim hpaid'
          · simp at h
  | closePool =>
      simp only [step] at h
      simp only [Option.some.injEq] at h
      subst s'
      refine ⟨?_, ?_, ?_, ?_⟩
      · intro hp
        exact hinv.pending_policy (by simpa using hp)
      · simpa using hinv.paid_cap
      · intro hp
        exact hinv.paid_token (by simpa using hp)
      · intro hp
        exact hinv.paid_claim (by simpa using hp)
  | claimDeposit c now =>
      simp only [step, claimDeposit] at h
      split at h
      · simp only [Option.some.injEq] at h
        subst s'
        refine ⟨?_, ?_, ?_, ?_⟩
        · intro hpend
          have hpend' : s.binding.status = .pending := by simpa using hpend
          have hpol := hinv.pending_policy hpend'
          simpa using hpol
        · simpa using hinv.paid_cap
        · intro hpaid
          have hpaid' : 0 < s.policy.paid := by simpa using hpaid
          simpa using hinv.paid_token hpaid'
        · intro hpaid
          have hpaid' : 0 < s.policy.paid := by simpa using hpaid
          simpa using hinv.paid_claim hpaid'
      · simp at h
  | dispute =>
      simp only [step, dispute] at h
      split at h
      · simp only [Option.some.injEq] at h
        subst s'
        refine ⟨?_, ?_, ?_, ?_⟩
        · intro hpend
          have hpend' : s.binding.status = .pending := by simpa using hpend
          have hpol := hinv.pending_policy hpend'
          simpa using hpol
        · simpa using hinv.paid_cap
        · intro hpaid
          have hpaid' : 0 < s.policy.paid := by simpa using hpaid
          simpa using hinv.paid_token hpaid'
        · intro hpaid
          have hpaid' : 0 < s.policy.paid := by simpa using hpaid
          simpa using hinv.paid_claim hpaid'
      · simp at h
  | settle =>
      simp only [step] at h
      simp only [Option.some.injEq] at h
      subst s'
      refine ⟨?_, ?_, ?_, ?_⟩
      · intro hpend
        exfalso
        simp [terminalSettle] at hpend
      · simpa [terminalSettle] using hinv.paid_cap
      · intro hpaid
        have hpaid' : 0 < s.policy.paid := by simpa [terminalSettle] using hpaid
        obtain ⟨tk, htk⟩ := hinv.paid_token hpaid'
        exact ⟨tk, by simpa [terminalSettle] using htk⟩
      · intro hpaid
        have hpaid' : 0 < s.policy.paid := by simpa [terminalSettle] using hpaid
        obtain ⟨c, hc⟩ := hinv.paid_claim hpaid'
        exact ⟨c, by simpa [terminalSettle] using hc⟩

private theorem reach_inv {s : State} (h : Reach init s) : Inv s := by
  induction h with
  | refl => exact init_inv
  | step _ hstep ih => exact step_inv ih hstep

/-! ## Public theorems -/

theorem authorize_fixes_token {s s' : State} {t : Token}
    (h : step s (.authorize t) = some s') :
    s.policy.token = none ∧ s'.policy.token = some t := by
  have ⟨hg, hp⟩ := authorize_success h
  exact ⟨hg.2.1, by rw [hp]⟩

theorem authorize_refuses_second {s s' : State} {t t' : Token}
    (h : step s (.authorize t) = some s') :
    step s' (.authorize t') = none := by
  have ⟨_, hp⟩ := authorize_success h
  have hsome : s'.policy.token = some t := by rw [hp]
  have hg' : ¬ authorizeOK s' t' := by
    rintro ⟨_, htok, _⟩
    rw [hsome] at htok
    exact Option.noConfusion htok
  simp [step, authorize, hg']

theorem attach_requires_authorized_token {s s' : State} {t : Token}
    {claimId : ClaimId} {lossId : LossId} {depositPart coverPart : Amount}
    (h : step s (.attach t claimId lossId depositPart coverPart) = some s') :
    s.policy.token = some t ∧ s.policy.status = .attached := by
  have ⟨hg, _⟩ := attach_success h
  exact ⟨hg.2.1, hg.1⟩

theorem claim_token_one_time {s s' : State} {t : Token}
    {claimId : ClaimId} {lossId : LossId} {depositPart coverPart : Amount}
    (h : step s (.attach t claimId lossId depositPart coverPart) = some s') :
    ∀ (t' : Token) (c' l' d' cp' : Nat),
      step s' (.attach t' c' l' d' cp') = none := by
  have ⟨_, hb⟩ := attach_success h
  have hclaimId : s'.binding.claimId = some claimId := by
    rw [hb]
  intro t' c' l' d' cp'
  have hg' : ¬ attachOK s' t' c' l' d' cp' := by
    rintro ⟨_, _, _, _, _, _, _, hnone, _⟩
    rw [hclaimId] at hnone
    exact Option.noConfusion hnone
  simp [step, attach, hg']

theorem pay_requires_attach {s s' : State} {claimId : ClaimId} {now : Height}
    (h : step s (.pay claimId now) = some s') :
    s.binding.status = .attached ∧ s.binding.claimId = some claimId := by
  have hok := payOK_of_step h
  exact ⟨hok.1, hok.2.1⟩

theorem pay_once {s s₁ s₂ : State} {claimId : ClaimId} {now : Height}
    (h₁ : step s (.pay claimId now) = some s₁)
    (h₂ : step s₁ (.pay claimId now) = some s₂) :
    s₂.policy.paid = s₁.policy.paid := by
  have hok' := payOK_next h₁
  have hcl := (paidClaim_of_step h₁).2
  have hp : pay s₁ claimId now = some s₁ := pay_replay hok' hcl
  have hs₂ : s₂ = s₁ := by
    simp only [step] at h₂
    rw [hp] at h₂
    injection h₂ with hs
    exact hs.symm
  rw [hs₂]

theorem payout_implies_authorized_attached {s : State}
    (hr : Reach init s) (hp : 0 < s.policy.paid) :
    authorized s ∧ attached s := by
  have hinv := reach_inv hr
  exact ⟨hinv.paid_token hp, hinv.paid_claim hp⟩

theorem expire_cannot_precede_pending_claim {s : State} {now : Height}
    (hr : Reach init s) (hp : PendingClaim s) :
    step s (.expire now) = none := by
  have hinv := reach_inv hr
  have hatt : s.policy.status = .attached := hinv.pending_policy hp
  simp [step, expire, hatt]

theorem terminal_settle_idempotent (s : State) :
    terminalSettle (terminalSettle s) = terminalSettle s := by
  cases s with
  | mk policy binding booking pool =>
      simp [terminalSettle]

theorem settle_ignores_deleted_pool (s : State) :
    terminalSettle (closePool s) = terminalSettle s := by
  simp [terminalSettle, closePool]

theorem claimDeposit_frozen_after_attach {s s' : State} {t : Token}
    {claimId : ClaimId} {lossId : LossId} {depositPart coverPart : Amount}
    (h : step s (.attach t claimId lossId depositPart coverPart) = some s') :
    ∀ (c : Claim) (now : Height), step s' (.claimDeposit c now) = none := by
  have ⟨_, hb⟩ := attach_success h
  have hclaimId : s'.binding.claimId = some claimId := by
    rw [hb]
  intro c now
  have hg' : ¬ claimDepositOK s' := by
    rintro ⟨_, _, hnone⟩
    rw [hclaimId] at hnone
    exact Option.noConfusion hnone
  simp [step, claimDeposit, hg']

theorem claimDeposit_refuses_existing_claim {s : State} {c : Claim} {now : Height}
    (h : s.booking.claim ≠ none ∨ s.binding.claimId ≠ none) :
    step s (.claimDeposit c now) = none := by
  have hg' : ¬ claimDepositOK s := by
    rcases h with hb | hb
    · rintro ⟨_, hnone, _⟩
      exact hb hnone
    · rintro ⟨_, _, hnone⟩
      exact hb hnone
  simp [step, claimDeposit, hg']

theorem dispute_refused_pending_claim {s : State}
    (h : s.booking.claim ≠ none ∨ s.binding.claimId ≠ none) :
    step s .dispute = none := by
  have hg' : ¬ disputeOK s := by
    rcases h with hb | hb
    · rintro ⟨hnone, _, _⟩
      exact hb hnone
    · rintro ⟨_, hnone, _⟩
      exact hb hnone
  simp [step, dispute, hg']

end Cover
end RShare
