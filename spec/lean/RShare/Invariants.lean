import RShare.Linear.Cell
import RShare.Linear.Protocol
import RShare.Model.Allocator

/-!
# Invariant aliases

Stable names for the flow manifest. Each alias points at the theorem that carries
the invariant; the register does not restate the statement.
-/

namespace RShare
namespace Invariants

open Allocator

/-- S1 — acquisition is all-or-none and one reservation. -/
def S1_acquire_all_or_none := @acquire_none_or_reservation
def S1_acquire_one_reservation := @acquire_is_one_reservation

/-- S2 — no slot has two owners. -/
def S2_no_double_owner := @acquire_no_double_owner

/-- S3 — a live hold refuses the whole set. -/
def S3_refuses_occupied := @acquire_refuses_occupied

/-- S4 — confirm is one map over the reservation list. -/
def S4_confirm_atomic := @confirm_none_or_map

/-- S5 — release removes every reservation with the token. -/
def S5_release_clears := @release_clears_token
def S5_release_keeps_other := @release_keeps_other

/-- S6 — the public projection carries no token. -/
def S6_public_no_token := @publicView_no_token

/-- S7 — the hold is bounded by the unit's maxHoldBlocks. -/
def S7_bounded_hold := @acquire_bounded

/-- S8 — a successful confirm has a matching token reservation. -/
def S8_confirm_token := @confirm_requires_token

/-- D1/D2/D3 — cell defects. -/
def D1_peek_then_put := @RShare.peek_then_put_rejected
def D2_lost_cell := @RShare.take_without_put_not_wf
def D3_nested_call := @RShare.nested_call_rejected

end Invariants
end RShare
