import RShare.Model.Allocator

/-!
# Deterministic vectors

`lake env lean RShare/Emit.lean` prints `name|input|output` lines. The parser
and the on-chain allocator conformance suite consume these vectors; drift fails
`tools/check_vectors.py`.
-/

namespace RShare
namespace Emit

open Allocator

def terms : Terms :=
  { minSlots := 1, maxSlots := 4, maxUnits := 2,
    maxLeadBlocks := 1000, maxSlotBlocks := 100, holdBlocks := 10 }

def unitA : AllocUnit :=
  { unitId := 1, pricePerSlot := 10, maxHoldBlocks := 10, status := UnitStatus.active }
def unitB : AllocUnit :=
  { unitId := 2, pricePerSlot := 20, maxHoldBlocks := 10, status := UnitStatus.active }
def unitPaused : AllocUnit :=
  { unitId := 1, pricePerSlot := 10, maxHoldBlocks := 10, status := UnitStatus.paused }

def base (status : ListingStatus) (u : List AllocUnit) : AllocState :=
  { status := status, terms := terms, units := u, reservations := [] }

def one (booking : BookingId) (token : Token) (untilHeight : Height) : Reservation :=
  { bookingId := booking, keys := [(1, 0)], untilHeight := untilHeight, token := token,
    booked := false }

def two : AcquireRequest :=
  { bookingId := 1, untilHeight := 15, token := 7,
    unitSlots := [{ unitId := 1, slots := [0, 1] }, { unitId := 2, slots := [0] }] }

def single : AcquireRequest :=
  { bookingId := 1, untilHeight := 15, token := 7,
    unitSlots := [{ unitId := 1, slots := [0] }] }

def emit (name input output : String) : IO Unit :=
  IO.println s!"{name}|{input}|{output}"

def okNone (o : Option α) : String :=
  match o with
  | some _ => "ok"
  | none => "none"

def main : IO Unit := do
  emit "allocator.acquire.free" "1,0,15,7" (okNone (acquire (base .active [unitA, unitB]) single 5))
  emit "allocator.acquire.multi_all" "3" (okNone (acquire (base .active [unitA, unitB]) two 5))
  emit "allocator.acquire.held_refuse" "1,0"
    (okNone (acquire { base .active [unitA, unitB] with reservations := [one 9 9 10] } single 5))
  emit "allocator.acquire.expired" "1,0"
    (okNone (acquire { base .active [unitA, unitB] with reservations := [one 9 9 5] } single 6))
  emit "allocator.acquire.multi_one_held" "3"
    (okNone (acquire { base .active [unitA, unitB] with reservations := [one 9 9 10] } two 5))
  emit "allocator.acquire.paused_listing" "1,0"
    (okNone (acquire (base .paused [unitA, unitB]) single 5))
  emit "allocator.acquire.paused_unit" "1,0"
    (okNone (acquire (base .active [unitPaused]) single 5))
  emit "allocator.acquire.bounded" "100"
    (okNone (acquire (base .active [unitA, unitB])
      { single with untilHeight := 100 } 5))
  emit "allocator.confirm.ok" "7"
    (okNone (confirm { base .active [unitA] with reservations := [one 1 7 15] } [(1, 0)] 7 5))
  emit "allocator.confirm.wrong_token" "9"
    (okNone (confirm { base .active [unitA] with reservations := [one 1 7 15] } [(1, 0)] 9 5))
  emit "allocator.release.ok" "7"
    (okNone (release { base .active [unitA] with reservations := [one 1 7 15] } 7))
  emit "allocator.release.wrong_token" "9"
    (okNone (release { base .active [unitA] with reservations := [one 1 7 15] } 9))
  let a := { base .active [unitA] with reservations := [one 1 7 15] }
  let b := { base .active [unitA] with reservations := [one 1 9 15] }
  emit "allocator.probe.redact" "7,9"
    (if publicView a == publicView b then "same" else "different")

end Emit
end RShare

#eval RShare.Emit.main
