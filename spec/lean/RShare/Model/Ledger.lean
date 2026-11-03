import RShare.Basic

/-!
# Guest ledger

The ledger reserves an `opId` before the book pipeline and finishes or drops it. A pending
or finished op is a replay, not a second booking.
-/

namespace RShare
namespace Ledger

abbrev OpId := Nat
abbrev BookingId := Nat

inductive OpEntry where
  | pending
  | finished (ok : Bool) (bookingId : BookingId) (uri : Nat)
  deriving Repr

structure Ledger where
  ops : List (OpId × OpEntry)
  deriving Repr

def lookup (l : Ledger) (opId : OpId) : Option OpEntry :=
  (l.ops.find? (fun entry => entry.1 == opId)).map (fun entry => entry.2)

def reserve (l : Ledger) (opId : OpId) : Bool × Ledger :=
  match lookup l opId with
  | none => (true, { ops := (opId, .pending) :: l.ops })
  | some _ => (false, l)

def finish (l : Ledger) (opId : OpId) (ok : Bool) (bookingId : BookingId)
    (uri : Nat) : Ledger :=
  { ops := (opId, .finished ok bookingId uri) :: l.ops.filter (fun e => e.1 != opId) }

def drop (l : Ledger) (opId : OpId) : Ledger :=
  { ops := l.ops.filter (fun e => e.1 != opId) }

theorem reserve_fresh {l l' : Ledger} {opId : OpId}
    (h : reserve l opId = (true, l')) : lookup l opId = none := by
  unfold reserve at h
  split at h
  · assumption
  · simp at h

theorem reserve_duplicate_refused {l : Ledger} {opId : OpId} {e : OpEntry}
    (h : lookup l opId = some e) : (reserve l opId).1 = false := by
  unfold reserve
  simp [h]

theorem finish_terminal (l : Ledger) (opId : OpId) (ok : Bool)
    (bookingId : BookingId) (uri : Nat) :
    lookup (finish l opId ok bookingId uri) opId = some (.finished ok bookingId uri) := by
  unfold finish lookup
  simp

end Ledger
end RShare
