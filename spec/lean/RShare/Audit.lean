import RShare.Invariants
import RShare.Model.System

/-!
# Axiom audit

`tools/check_lean.py` runs this file and accepts only `propext`,
`Classical.choice` and `Quot.sound`. `sorryAx` is never accepted.
-/

#print axioms RShare.Allocator.keysOf_mem
#print axioms RShare.Allocator.acquire_none_or_reservation
#print axioms RShare.Allocator.acquire_success_reservation
#print axioms RShare.Allocator.acquire_marks_requested
#print axioms RShare.Allocator.freeAll_true
#print axioms RShare.Allocator.checkAcquire_free
#print axioms RShare.Allocator.acquire_refuses_occupied
#print axioms RShare.Allocator.acquire_no_double_owner
#print axioms RShare.Allocator.acquire_bounded
#print axioms RShare.Allocator.acquire_is_one_reservation
#print axioms RShare.Allocator.confirm_none_or_map
#print axioms RShare.Allocator.confirm_requires_token
#print axioms RShare.Allocator.release_none_or_filter
#print axioms RShare.Allocator.release_clears_token
#print axioms RShare.Allocator.release_keeps_other
#print axioms RShare.Allocator.publicView_no_token
#print axioms RShare.peek_then_put_rejected
#print axioms RShare.put_without_take_rejected
#print axioms RShare.take_without_put_not_wf
#print axioms RShare.nested_call_rejected
#print axioms RShare.take_put_commit
#print axioms RShare.commit_all_or_none
#print axioms RShare.commit_keeps_held
#print axioms RShare.commit_requires_single
#print axioms RShare.h2a_caller_token
#print axioms RShare.h2b_delegated
#print axioms RShare.p2_await_then_commit

#print axioms RShare.Remedy.fullRefund_conserves
#print axioms RShare.Remedy.feeSettle_conserves
#print axioms RShare.Remedy.depositSplit_conserves
#print axioms RShare.Bond.pay_once
#print axioms RShare.Bond.pay_closes
#print axioms RShare.Bond.pay_conserves
#print axioms RShare.Booking.terminal_no_step
#print axioms RShare.Booking.terminal_sticky
#print axioms RShare.Booking.settle_once
#print axioms RShare.Booking.dispute_only_from_checked
#print axioms RShare.Booking.progress
#print axioms RShare.System.system_slot_single_owner
#print axioms RShare.System.alloc_transition_is_one_commit
#print axioms RShare.System.system_settle_once
#print axioms RShare.System.system_terminal_progress
