# Invariants

1. Every state cell is Full at a quiescent point; a refusal reposts the cell.
2. A slot is exactly one of `free`, `held`, `booked`.
3. A booking has exactly one terminal outcome (`settled`, `rejected`, `cancelled`,
   `expired`, `noShow`).
4. For each asset, `Σ r-cash account balances + Σ booking escrow purses` is invariant under
   every operation.
5. A terminal booking's settlement buckets sum to `rent + deposit`.
6. A slot releases only for the release token that holds it.
7. A booking's `rent` equals the sum of each booked unit's `pricePerSlot` times its slot
   count; the deposit is always separate.
8. No two bookings hold the same slot at the same time; a lapsed hold may be reclaimed.
9. `bookingId` is unique per `(guestId, opId)`; a replay returns the stored result.
10. Provider, guest, listing and unit authority is derived from `deployerId`; no owner id is
    accepted as an argument.
