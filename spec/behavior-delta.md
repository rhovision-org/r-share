# Behavior deltas for the base-layer refactor

The refactor is not fully behavior-preserving. Everything below is an intentional delta;
anything else is a regression. Money amounts, slot states, terminal statuses and ownership
refusals must match the R0 baseline exactly.

| Delta | New behavior | Test |
|---|---|---|
| Global pause | `ShareAdmin.pauseAll` makes `GuestLedger.book` refuse `"paused"`; existing lifecycle arms still work | `unit_pause` (passing) |
| Provider pause | A paused provider blocks new bookings on its existing listings with `"provider paused"`; resume restores | `unit_pause` (passing) |
| Unit pause | `ListingContract.acquire` refuses `"unit paused"`; release and confirm still work for existing holds | `unit_pause` (passing) |
| Contiguity | `contiguous` is enforced from the terms flag for every kind, not hardcoded to lodging | enforced in `Terms.scheduleOk`; no dedicated case |
| Component keys | `component:policy` alias is removed; `component:remedy` remains; `component:listing-contract-factory` and `component:booking-factory` are removed or repointed | `unit_root` keys (passing) |
| Bond refusals | Bond `"closed"`/`"not holder"` are mapped to the booking's documented refusal strings at the BookingContract boundary | not implemented (no dedicated suite) |
| Slot lookup fallback | The per-slot contracts and the `slotCaps` fallback are removed; slots are data in the per-listing allocator cell | `unit_allocator` (passing) |

The 16-suite campaign in `docs/reference/verification.md` is the executed set. The other
rows are design deltas with no passing test; do not read them as verified.

## Non-deltas

- Terms hashes for v1 must be byte-identical.
- Remedy outputs for `fee`, `cancel`, `noShow`, `depositSplit` must match the legacy `Policy`.
- Owner derivation, admin payloads, opId idempotency, slot records, release tokens and
  terminal states must match.
