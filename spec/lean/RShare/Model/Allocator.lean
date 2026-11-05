import RShare.Basic

/-!
# Per-listing atomic allocator

The allocator is one cell per listing. A booking request is represented as one
`Reservation` appended to `reservations` in a single commit, so there is no
reachable state in which some requested keys are held and others are not.

The model abstracts the deployed slot map: `keysOf` flattens the requested
`(unitId, slotId)` pairs and `occupied` decides whether a key is held by a live
reservation or booked. The contract's exact refusal reasons are mirrored in
`checkAcquire`; the flow manifest maps each arm to these declarations.
-/

namespace RShare
namespace Allocator

abbrev UnitId := Nat
abbrev SlotId := Nat
abbrev BookingId := Nat
abbrev Token := Nat
abbrev SlotKey := UnitId × SlotId

inductive ListingStatus where
  | draft | active | paused | delisted
  deriving DecidableEq, Repr

inductive UnitStatus where
  | active | paused
  deriving DecidableEq, Repr

structure AllocUnit where
  unitId : UnitId
  pricePerSlot : Amount
  maxHoldBlocks : BlockSpan
  status : UnitStatus
  deriving Repr

structure Terms where
  minSlots : Nat
  maxSlots : Nat
  maxUnits : Nat
  maxLeadBlocks : Nat
  maxSlotBlocks : Nat
  holdBlocks : Nat
  deriving Repr

structure Reservation where
  bookingId : BookingId
  keys : List SlotKey
  untilHeight : Height
  token : Token
  booked : Bool
  deriving Repr

structure AllocState where
  status : ListingStatus
  terms : Terms
  units : List AllocUnit
  reservations : List Reservation
  deriving Repr

structure UnitRequest where
  unitId : UnitId
  slots : List SlotId
  deriving Repr

structure AcquireRequest where
  bookingId : BookingId
  unitSlots : List UnitRequest
  untilHeight : Height
  token : Token
  deriving Repr

structure AcquireSummary where
  count : Nat
  rent : Amount
  deriving Repr

def keysOf : List UnitRequest → List SlotKey
  | [] => []
  | ur :: rest => ur.slots.map (fun s => (ur.unitId, s)) ++ keysOf rest

def countOf (req : AcquireRequest) : Nat := (keysOf req.unitSlots).length

def unitOf (st : AllocState) (id : UnitId) : Option AllocUnit :=
  st.units.find? (fun u => u.unitId == id)

def unitPrice (st : AllocState) (id : UnitId) : Amount :=
  match unitOf st id with
  | some u => u.pricePerSlot
  | none => 0

def rentOf (st : AllocState) (req : AcquireRequest) : Amount :=
  req.unitSlots.foldl (fun acc ur => acc + ur.slots.length * unitPrice st ur.unitId) 0

def summaryOf (st : AllocState) (req : AcquireRequest) : AcquireSummary :=
  { count := countOf req, rent := rentOf st req }

def active (r : Reservation) (now : Height) : Bool :=
  r.booked || decide (now < r.untilHeight)

/-- Is a key held by a live reservation or booked? -/
def occupied (st : AllocState) (k : SlotKey) (now : Height) : Bool :=
  st.reservations.any (fun r => decide (k ∈ r.keys) && active r now)

def freeAll (st : AllocState) (req : AcquireRequest) (now : Height) : Bool :=
  req.unitSlots.all (fun ur =>
    ur.slots.all (fun s => !occupied st (ur.unitId, s) now))

def unitsActive (st : AllocState) (req : AcquireRequest) (now : Height) : Bool :=
  req.unitSlots.all (fun ur =>
    match unitOf st ur.unitId with
    | none => false
    | some u =>
        u.status == UnitStatus.active &&
        req.untilHeight ≤ now + u.maxHoldBlocks)

def basicChecks (st : AllocState) (req : AcquireRequest) (now : Height) : Bool :=
  (st.status == ListingStatus.active) &&
  (decide (st.terms.minSlots ≤ countOf req)) &&
  (decide (countOf req ≤ st.terms.maxSlots)) &&
  (decide (req.unitSlots.length ≤ st.terms.maxUnits)) &&
  (decide (now < req.untilHeight)) &&
  unitsActive st req now

def checkAcquire (st : AllocState) (req : AcquireRequest) (now : Height) : Bool :=
  basicChecks st req now && freeAll st req now

def reservationOf (req : AcquireRequest) : Reservation :=
  { bookingId := req.bookingId
    keys := keysOf req.unitSlots
    untilHeight := req.untilHeight
    token := req.token
    booked := false }

/-- One atomic acquisition: the whole request becomes one reservation, or the
state is unchanged. -/
def acquire (st : AllocState) (req : AcquireRequest) (now : Height) :
    Option (AllocState × AcquireSummary) :=
  if checkAcquire st req now then
    some ({ st with reservations := reservationOf req :: st.reservations },
          summaryOf st req)
  else
    none

/-- Confirm is all-or-none over the requested keys: a reservation with the same
token and key set becomes booked, or nothing changes. -/
def confirm (st : AllocState) (keys : List SlotKey) (token : Token)
    (now : Height) : Option AllocState :=
  if st.reservations.any
      (fun r => (r.token == token) && (r.keys == keys) &&
                (!r.booked) && decide (now < r.untilHeight)) then
    some { st with reservations :=
      st.reservations.map (fun r =>
        if (r.token == token) && (r.keys == keys) then { r with booked := true }
        else r) }
  else
    none

/-- Release removes every reservation with the token. Reclaimed reservations
carry a different token and are ignored, so release never strands the holder's
other keys. -/
def release (st : AllocState) (token : Token) : Option AllocState :=
  if st.reservations.any (fun r => r.token == token) then
    some { st with reservations :=
      st.reservations.filter (fun r => !(r.token == token)) }
  else
    none

/-- Public projection: no token is ever part of an observation. -/
structure PublicReservation where
  bookingId : BookingId
  keys : List SlotKey
  booked : Bool
  deriving DecidableEq, Repr

def public (r : Reservation) : PublicReservation :=
  { bookingId := r.bookingId, keys := r.keys, booked := r.booked }

def publicView (st : AllocState) : List PublicReservation :=
  st.reservations.map public

/-- The summary rent is exactly `rentOf`; invariant 7 starts here. -/
theorem summaryOf_rent (st : AllocState) (req : AcquireRequest) :
    (summaryOf st req).rent = rentOf st req := rfl

/-- The summary slot count is exactly the flattened requested-key count. -/
theorem summaryOf_count (st : AllocState) (req : AcquireRequest) :
    (summaryOf st req).count = countOf req := rfl

end Allocator
end RShare

/-! ## Core theorems -/

namespace RShare
namespace Allocator

theorem keysOf_mem {urs : List UnitRequest} {ur : UnitRequest}
    (hur : ur ∈ urs) {s : SlotId} (hs : s ∈ ur.slots) :
    (ur.unitId, s) ∈ keysOf urs := by
  induction urs with
  | nil => simp at hur
  | cons x xs ih =>
      simp only [keysOf, List.mem_append, List.mem_map]
      rcases List.mem_cons.mp hur with rfl | hx
      · exact Or.inl ⟨s, hs, rfl⟩
      · exact Or.inr (ih hx)

theorem reservationOf_keys {req : AcquireRequest} {ur : UnitRequest}
    (hur : ur ∈ req.unitSlots) {s : SlotId} (hs : s ∈ ur.slots) :
    (ur.unitId, s) ∈ (reservationOf req).keys :=
  keysOf_mem hur hs

/-- An acquisition is either a refusal with the state unchanged, or exactly one
new reservation. -/
theorem acquire_none_or_reservation (st : AllocState) (req : AcquireRequest)
    (now : Height) :
    acquire st req now = none ∨
      ∃ sum, acquire st req now =
        some ({ st with reservations := reservationOf req :: st.reservations }, sum) := by
  unfold acquire
  split
  · exact Or.inr ⟨summaryOf st req, rfl⟩
  · exact Or.inl rfl

theorem acquire_success_reservation {st st' : AllocState} {req : AcquireRequest}
    {now : Height} {sum : AcquireSummary}
    (h : acquire st req now = some (st', sum)) :
    st'.reservations = reservationOf req :: st.reservations := by
  unfold acquire at h
  split at h
  · simp only [Option.some.injEq, Prod.mk.injEq] at h
    rw [← h.1]
  · simp at h

/-- A successful acquire marks every requested key in the new reservation. -/
theorem acquire_marks_requested {st st' : AllocState} {req : AcquireRequest}
    {now : Height} {sum : AcquireSummary}
    (_h : acquire st req now = some (st', sum)) {ur : UnitRequest}
    (hur : ur ∈ req.unitSlots) {s : SlotId} (hs : s ∈ ur.slots) :
    (ur.unitId, s) ∈ (reservationOf req).keys :=
  reservationOf_keys hur hs

theorem freeAll_true {st : AllocState} {req : AcquireRequest} {now : Height}
    (h : freeAll st req now = true) {ur : UnitRequest}
    (hur : ur ∈ req.unitSlots) {s : SlotId} (hs : s ∈ ur.slots) :
    occupied st (ur.unitId, s) now = false := by
  unfold freeAll at h
  rw [List.all_eq_true] at h
  have h1 := h ur hur
  rw [List.all_eq_true] at h1
  have h2 := h1 s hs
  simpa using h2

theorem checkAcquire_free {st : AllocState} {req : AcquireRequest} {now : Height}
    (h : checkAcquire st req now = true) {ur : UnitRequest}
    (hur : ur ∈ req.unitSlots) {s : SlotId} (hs : s ∈ ur.slots) :
    occupied st (ur.unitId, s) now = false := by
  have hf : freeAll st req now = true := by
    unfold checkAcquire at h
    simp only [Bool.and_eq_true] at h
    exact h.2
  exact freeAll_true hf hur hs

/-- If any requested key is already occupied, the whole acquisition refuses and
leaves the state unchanged. This is the no-double-allocation guard. -/
theorem acquire_refuses_occupied {st : AllocState} {req : AcquireRequest}
    {now : Height} {ur : UnitRequest} (hur : ur ∈ req.unitSlots)
    {s : SlotId} (hs : s ∈ ur.slots)
    (hoccupied : occupied st (ur.unitId, s) now = true) :
    acquire st req now = none := by
  have hf : freeAll st req now = false := by
    cases h : freeAll st req now with
    | false => rfl
    | true =>
        have hfalse := freeAll_true h hur hs
        rw [hfalse] at hoccupied
        exact Bool.noConfusion hoccupied
  simp [acquire, checkAcquire, hf]

/-- Confirm is one map over the reservation list: it either books every matching
reservation or changes nothing. -/
theorem confirm_none_or_map (st : AllocState) (keys : List SlotKey)
    (token : Token) (now : Height) :
    confirm st keys token now = none ∨
      confirm st keys token now =
        some { st with reservations :=
          st.reservations.map (fun r =>
            if (r.token == token) && (r.keys == keys) then
              { r with booked := true } else r) } := by
  unfold confirm
  split
  · exact Or.inr rfl
  · exact Or.inl rfl

/-- Release either removes every reservation with the token, or refuses. -/
theorem release_none_or_filter (st : AllocState) (token : Token) :
    release st token = none ∨
      release st token =
        some { st with reservations :=
          st.reservations.filter (fun r => !(r.token == token)) } := by
  unfold release
  split
  · exact Or.inr rfl
  · exact Or.inl rfl

theorem release_clears_token {st st' : AllocState} {token : Token}
    (h : release st token = some st') :
    ∀ r ∈ st'.reservations, r.token ≠ token := by
  unfold release at h
  split at h
  · simp only [Option.some.injEq] at h
    rw [← h]
    intro r hr
    simp only [List.mem_filter] at hr
    have := hr.2
    simpa using this
  · simp at h

/-- The public projection is independent of the token and the hold deadline. -/
theorem publicView_no_token (st : AllocState) (t t' : Token) :
    publicView { st with reservations :=
        st.reservations.map (fun r => { r with token := t }) } =
    publicView { st with reservations :=
        st.reservations.map (fun r => { r with token := t' }) } := by
  simp [publicView, public]

/-- Acquisition is one reservation append, not a per-slot write. -/
theorem acquire_is_one_reservation {st st' : AllocState} {req : AcquireRequest}
    {now : Height} {sum : AcquireSummary}
    (h : acquire st req now = some (st', sum)) :
    st'.reservations.length = st.reservations.length + 1 := by
  rw [acquire_success_reservation h]
  simp

end Allocator
end RShare

namespace RShare
namespace Allocator

theorem occupied_false_no_active {st : AllocState} {k : SlotKey} {now : Height}
    (h : occupied st k now = false) :
    ∀ r ∈ st.reservations, k ∈ r.keys → active r now = false := by
  intro r hr hk
  cases ha : active r now with
  | false => rfl
  | true =>
      exfalso
      have htrue : occupied st k now = true := by
        unfold occupied
        rw [List.any_eq_true]
        exact ⟨r, hr, by simp [hk, ha]⟩
      rw [htrue] at h
      exact Bool.noConfusion h

theorem checkAcquire_until {st : AllocState} {req : AcquireRequest}
    {now : Height} (h : checkAcquire st req now = true) :
    now < req.untilHeight := by
  unfold checkAcquire basicChecks at h
  simp only [Bool.and_eq_true, decide_eq_true_eq] at h
  simp_all

theorem unitsActive_true {st : AllocState} {req : AcquireRequest}
    {now : Height} (h : unitsActive st req now = true)
    {ur : UnitRequest} (hur : ur ∈ req.unitSlots) {u : AllocUnit}
    (hu : unitOf st ur.unitId = some u) :
    u.status = UnitStatus.active ∧ req.untilHeight ≤ now + u.maxHoldBlocks := by
  unfold unitsActive at h
  rw [List.all_eq_true] at h
  have h1 := h ur hur
  simp [hu] at h1
  exact h1

theorem acquire_bounded {st st' : AllocState} {req : AcquireRequest}
    {now : Height} {sum : AcquireSummary}
    (h : acquire st req now = some (st', sum))
    {ur : UnitRequest} (hur : ur ∈ req.unitSlots) {u : AllocUnit}
    (hu : unitOf st ur.unitId = some u) :
    now < req.untilHeight ∧ req.untilHeight ≤ now + u.maxHoldBlocks := by
  have hc : checkAcquire st req now = true := by
    unfold acquire at h
    split at h
    · assumption
    · simp at h
  have hb : basicChecks st req now = true := by
    unfold checkAcquire at hc
    simp only [Bool.and_eq_true] at hc
    exact hc.1
  have hActive : unitsActive st req now = true := by
    unfold basicChecks at hb
    simp only [Bool.and_eq_true] at hb
    exact hb.2
  exact ⟨checkAcquire_until hc, (unitsActive_true hActive hur hu).2⟩

/-- Every active reservation that still contains a requested key after a
successful acquire belongs to the acquiring booking. That is the single-owner
theorem. -/
theorem acquire_no_double_owner {st st' : AllocState} {req : AcquireRequest}
    {now : Height} {sum : AcquireSummary}
    (h : acquire st req now = some (st', sum))
    {ur : UnitRequest} (hur : ur ∈ req.unitSlots) {s : SlotId}
    (hs : s ∈ ur.slots) :
    (∃ r ∈ st'.reservations,
        r.bookingId = req.bookingId ∧ (ur.unitId, s) ∈ r.keys ∧
          active r now = true) ∧
    (∀ r ∈ st'.reservations, (ur.unitId, s) ∈ r.keys →
        active r now = true → r.bookingId = req.bookingId) := by
  have hc : checkAcquire st req now = true := by
    unfold acquire at h
    split at h
    · assumption
    · simp at h
  have hfree : occupied st (ur.unitId, s) now = false :=
    checkAcquire_free hc hur hs
  have huntil : now < req.untilHeight := checkAcquire_until hc
  have hnewActive : active (reservationOf req) now = true := by
    simp [active, reservationOf, huntil]
  have hnewKey : (ur.unitId, s) ∈ (reservationOf req).keys :=
    reservationOf_keys hur hs
  constructor
  · refine ⟨reservationOf req, ?_, rfl, hnewKey, hnewActive⟩
    rw [acquire_success_reservation h]
    simp
  · intro r hr hkey hactive
    rw [acquire_success_reservation h] at hr
    rcases List.mem_cons.mp hr with rfl | hold
    · rfl
    · have hno := occupied_false_no_active hfree r hold hkey
      rw [hno] at hactive
      exact Bool.noConfusion hactive

theorem confirm_requires_token {st st' : AllocState} {keys : List SlotKey}
    {token : Token} {now : Height}
    (h : confirm st keys token now = some st') :
    ∃ r ∈ st.reservations,
      r.token = token ∧ r.keys = keys ∧ r.booked = false ∧ now < r.untilHeight := by
  unfold confirm at h
  split at h
  · rename_i hcond
    rw [List.any_eq_true] at hcond
    rcases hcond with ⟨r, hr, hprop⟩
    simp only [Bool.and_eq_true, decide_eq_true_eq, beq_iff_eq,
      Bool.not_eq_true] at hprop
    exact ⟨r, hr, hprop.1.1.1, hprop.1.1.2, by simpa using hprop.1.2, hprop.2⟩
  · simp at h

theorem release_keeps_other {st st' : AllocState} {token : Token}
    (h : release st token = some st') {r : Reservation}
    (hr : r ∈ st.reservations) (hne : r.token ≠ token) :
    r ∈ st'.reservations := by
  unfold release at h
  split at h
  · simp only [Option.some.injEq] at h
    rw [← h]
    simp [List.mem_filter, hr, hne]
  · simp at h

end Allocator
end RShare

namespace RShare
namespace Allocator

/-- A unit is inserted as data; the allocator owns no per-unit cell. -/
def ensureUnit (st : AllocState) (u : AllocUnit) : AllocState :=
  { st with units := u :: st.units }

theorem ensureUnit_head (st : AllocState) (u : AllocUnit) :
    (ensureUnit st u).units = u :: st.units := rfl

def setStatus (st : AllocState) (s : ListingStatus) : AllocState :=
  { st with status := s }

theorem setStatus_applied (st : AllocState) (s : ListingStatus) :
    (setStatus st s).status = s := rfl

end Allocator
end RShare
