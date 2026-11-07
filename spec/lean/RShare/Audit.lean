import RShare.Invariants
import RShare.Model.System
import RShare.Model.Cover
import RShare.Linear.Concurrency
import RShare.Linear.Deploy

/-!
# Axiom audit

`tools/check_lean.py` runs this file and accepts only `propext`,
`Classical.choice` and `Quot.sound`. `sorryAx` is never accepted.
-/

#print axioms RShare.Allocator.summaryOf_rent
#print axioms RShare.Allocator.summaryOf_count
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

-- Cover state machine.
#print axioms RShare.Cover.authorize_fixes_token
#print axioms RShare.Cover.authorize_refuses_second
#print axioms RShare.Cover.attach_requires_authorized_token
#print axioms RShare.Cover.claim_token_one_time
#print axioms RShare.Cover.pay_requires_attach
#print axioms RShare.Cover.pay_once
#print axioms RShare.Cover.payout_implies_authorized_attached
#print axioms RShare.Cover.expire_cannot_precede_pending_claim
#print axioms RShare.Cover.terminal_settle_idempotent
#print axioms RShare.Cover.settle_ignores_deleted_pool
#print axioms RShare.Cover.claimDeposit_frozen_after_attach
#print axioms RShare.Cover.claimDeposit_refuses_existing_claim
#print axioms RShare.Cover.dispute_refused_pending_claim

-- Disjoint-commit concurrency and deploy atomicity.
#print axioms RShare.run_append
#print axioms RShare.commit_commute
#print axioms RShare.run_disjoint_commute
#print axioms RShare.disjoint_cells_linearizable
#print axioms RShare.commit_preserves_wf
#print axioms RShare.run_failure_no_leak
#print axioms RShare.deploy_atomic_rollback
#print axioms RShare.held_across_external_call_unsafe
#print axioms RShare.external_read_then_commit_safe

-- Added by the audit: previously unaudited source theorems.
#print axioms RShare.Store.set_same
#print axioms RShare.Store.set_other
#print axioms RShare.wf_quiescent
#print axioms RShare.commit_success
#print axioms RShare.h1_pure
#print axioms RShare.commit_counts
#print axioms RShare.Allocator.reservationOf_keys
#print axioms RShare.Allocator.occupied_false_no_active
#print axioms RShare.Allocator.checkAcquire_until
#print axioms RShare.Allocator.unitsActive_true
#print axioms RShare.Allocator.ensureUnit_head
#print axioms RShare.Allocator.setStatus_applied
#print axioms RShare.Bond.pay_requires_token
#print axioms RShare.Bond.pay_requires_expected
#print axioms RShare.Bond.pay_requires_sum
#print axioms RShare.Bond.bucket_nonneg
#print axioms RShare.Booking.cancel_only_from_open
#print axioms RShare.Booking.accept_only_from_requested
#print axioms RShare.Booking.expire_only_from_requested
#print axioms RShare.Booking.reject_only_from_requested
#print axioms RShare.Booking.checkIn_only_from_confirmed
#print axioms RShare.Booking.checkOut_only_from_checkedIn
#print axioms RShare.Booking.settle_from_checked_or_redeemed_or_disputed
#print axioms RShare.Booking.resolve_step_only_from_disputed
#print axioms RShare.Booking.noShowClaim_only_from_confirmed
#print axioms RShare.Booking.noShowDispute_only_from_claimed
#print axioms RShare.Booking.noShowResolves
#print axioms RShare.Ledger.reserve_fresh
#print axioms RShare.Ledger.reserve_duplicate_refused
#print axioms RShare.Ledger.finish_terminal
#print axioms RShare.Remedy.fee_le_rent
#print axioms RShare.Remedy.noShow_conserves
