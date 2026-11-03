# Invariants

1. Every state cell is Full at a quiescent point; a refusal reposts the cell.
1a. A multi-slot acquisition is one cell commit: every requested slot is held, or none is.
2. A slot is exactly one of `free`, `held`, `booked`.
3. A booking has exactly one terminal outcome (`settled`, `rejected`, `cancelled`,
   `expired`, `noShow`).
4. For each asset, `Σ r-cash account balances + Σ booking escrow purses` is invariant under
   every operation.
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

## Formal status

`spec/lean/` proves the model-level obligations for 1a, 2, 3, 5, 6, 7 and 8. The result is
`proved-model`; the flow manifest in `spec/flows/core.yaml` names the declaration for each
core arm. See [Formal model](formal-model.md) for the boundary.
