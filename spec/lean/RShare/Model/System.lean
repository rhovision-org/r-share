import RShare.Model.Allocator
import RShare.Model.Booking
import RShare.Model.Bond
import RShare.Model.Ledger

/-!
# System composition

The cornerstones compose by single-cell ownership: the allocator has one cell per listing,
each booking and Bond has one cell, and the ledger has one cell per guest. This module
records the composition obligations as aliases to the component theorems and names the
boundary assumptions the model does not prove.
-/

namespace RShare
namespace System

open Allocator Booking Bond

abbrev ListingId := Nat
abbrev BookingId := Nat

structure System where
  allocations : List (ListingId × AllocState)
  bookings : List (BookingId × BStatus)
  bonds : List (BookingId × BondState)

/-- A slot has one owner because acquisition is one reservation append. -/
def system_slot_single_owner := @Allocator.acquire_no_double_owner

/-- A successful ticketing transition is one `commit` on the allocator cell. -/
def alloc_transition_is_one_commit := @Allocator.acquire_is_one_reservation

/-- A Bond pays at most once. -/
def system_settle_once := @Bond.pay_once

/-- Every booking status has a path to a terminal status. -/
def system_terminal_progress := @Booking.progress

/-- The explicit boundary of the model. -/
structure Boundary where
  providerActiveCheckedByLedger : Bool
  callerHoldBoundedByMaxHold : Bool
  clockIsBlockHeight : Bool
  namesUnforgeable : Bool
  acquireHoldsListingCellAcrossAwait : Bool
  noRholangRefinement : Bool
  coverPolicyCellAtomic : Bool
  claimTokenUnforgeable : Bool
  ertpPurseAtomic : Bool
  operatorSettleTrusted : Bool
  nodeDeployAtomic : Bool
  bindReservedRequiresHold : Bool

def boundary : Boundary :=
  { providerActiveCheckedByLedger := true
    callerHoldBoundedByMaxHold := true
    clockIsBlockHeight := true
    namesUnforgeable := true
    acquireHoldsListingCellAcrossAwait := true
    noRholangRefinement := true
    coverPolicyCellAtomic := true
    claimTokenUnforgeable := true
    ertpPurseAtomic := true
    operatorSettleTrusted := true
    nodeDeployAtomic := true
    bindReservedRequiresHold := true }

/-- A caller can hold free slots without a booking; the hold is bounded by the unit's
`maxHoldBlocks`, so the grief is temporary. -/
def caller_hold_bounded := @Allocator.acquire_bounded

end System
end RShare
