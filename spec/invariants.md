# Invariants

1. Every state cell is Full at a quiescent point; a refusal reposts the cell.
1a. A multi-slot acquisition is one cell commit: every requested slot is held, or none is.
2. A slot is exactly one of `free`, `held`, `booked`.
3. A booking has exactly one terminal outcome (`settled`, `rejected`, `cancelled`,
   `expired`, `noShow`, `noShowRefunded`).
4. For each asset, `Σ r-cash account balances + Σ booking escrow purses` is invariant under
   every operation. When cover is enabled, the closed set also includes every cover pool
   purse and provider/platform reserve sub-ledger; premiums and claim payouts conserve.
5. A terminal booking's settlement buckets sum to `rent + deposit`.
6. A slot is confirmed or released only for the release token that holds it. Release is a
   cleanup: it frees every requested slot the token still owns and ignores reclaimed keys.
7. A booking's `rent` equals the sum of each booked unit's `pricePerSlot` times its slot
   count; the deposit is always separate.
8. No two bookings hold the same slot at the same time; a lapsed hold may be reclaimed. The
   per-listing allocator cell is the single owner of every slot.
9. `bookingId` is unique per `(guestId, opId)`; a replay returns the stored result.
10. Provider, guest and listing authority is derived from `deployerId`; unit pause and
    reset are listing arms under the same owner check. No owner id is accepted as an argument.
11. A terminal settlement releases the booking's slots under its release token before the
    Bond pays. A slot is not left booked after the service ends.
12. A cover premium is paid by the insured or the platform, never by the counterparty.
13. Premium conservation: commission + facility/pool legs + reserves equal the premium paid;
    no money is created by a cover offer or policy.
14. Claim conservation: `depositPart + coverPart + otherRecovery` equals the award and never
    exceeds the assessed loss; a second policy pays only the unrecovered excess of the same
    `lossId`.
15. Pool solvency: `cash >= solvencyBps * outstandingCover / 10000`; an insolvent pool
    refuses `"solvency"` before moving value and the subject remains retryable.
16. A cover claim cannot reopen a terminal booking/ticket/session/service contract.
17. Only the subject contract's private cover token can authorize a pool payout; a party
    cannot pay itself.
18. A quoted subject premium is never below the facility's loaded-premium floor.
19. `approval.liability.shareBps` is an indemnity/co-insurance input, not an engineering
    variance signal.
20. The cover engine is subject-kind agnostic: booking, ticket, session and service contract
    share one policy state machine; use case is configuration.
21. An evidence hash is an anchor, not proof; adjudication remains a trust point.

Cover invariants 12–21 are design obligations for the Phase 1+ implementation. They are not
yet in the Lean model or the flow manifest.

## Formal status

`spec/lean/` proves the model-level obligations for 1a, 2, 3, 5, 6 and 8. Invariant 7 is
only partially modelled: `summaryOf_rent` and `summaryOf_count` exist, but `rentOf_eq_sum`
is still open. The result is `proved-model`; the flow manifest in `spec/flows/core.yaml`
names the declaration for each core arm. See [Formal model](formal-model.md) for the
boundary and [Audit findings](audit-findings.md) for the current gaps.
