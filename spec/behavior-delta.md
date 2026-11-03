# Behavior deltas for the base-layer refactor

The refactor is not fully behavior-preserving. Everything below is an intentional delta;
anything else is a regression. Money amounts, slot states, terminal statuses and ownership
refusals must match the R0 baseline exactly.

| Delta | New behavior | Test |
|---|---|---|
| Global pause | `ShareAdmin.pauseAll` makes `GuestLedger.book` refuse `"paused"`; existing lifecycle arms still work | not implemented (planned `unit_pause`) |
| Provider pause | A paused provider blocks new bookings on its existing listings with `"provider paused"`; resume restores | not implemented (planned `unit_pause`) |
| Unit pause | `Unit.hold` refuses `"unit paused"`; `release` and `confirm` still work for existing holds | not implemented (planned `unit_pause`) |
| Contiguity | `contiguous` is enforced from the terms flag for every kind, not hardcoded to lodging | not implemented (planned `unit_listing`) |
| Component keys | `component:policy` alias is removed; `component:remedy` remains; `component:listing-contract-factory` and `component:booking-factory` are removed or repointed | `unit_root` keys (passing) |
| Bond refusals | Bond `"closed"`/`"not holder"` are mapped to the booking's documented refusal strings at the BookingContract boundary | not implemented (planned `unit_bond`) |
| Slot lookup fallback | If the `slotCaps` closure is not viable, per-slot `rl!` remains; this is a performance delta, not an observable behavior delta | not implemented (planned `slot_closure_spike`) |

Only `unit_root`, `unit_payments` and `unit_lifecycle` are executed today. The other rows
are design deltas with no passing test; do not read them as verified.

## Non-deltas

- Terms hashes for v1 must be byte-identical.
- Remedy outputs for `fee`, `cancel`, `noShow`, `depositSplit` must match the legacy `Policy`.
- Owner derivation, admin payloads, opId idempotency, slot records, release tokens and
  terminal states must match.
