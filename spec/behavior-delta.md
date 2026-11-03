# Behavior deltas for the base-layer refactor

The refactor is not fully behavior-preserving. Everything below is an intentional delta;
anything else is a regression. Money amounts, slot states, terminal statuses and ownership
refusals must match the R0 baseline exactly.

| Delta | New behavior | Test |
|---|---|---|
| Global pause | `ShareAdmin.pauseAll` makes `GuestLedger.book` refuse `"paused"`; existing lifecycle arms still work | `harness/unit_pause.py` |
| Provider pause | A paused provider blocks new bookings on its existing listings with `"provider paused"`; resume restores | `harness/unit_pause.py` |
| Unit pause | `Unit.hold` refuses `"unit paused"`; `release` and `confirm` still work for existing holds | `harness/unit_pause.py` |
| Contiguity | `contiguous` is enforced from the terms flag for every kind, not hardcoded to lodging | `unit_listing` schedule cases |
| Component keys | `component:policy` may remain as a one-release alias for `component:remedy`; `component:listing-contract-factory` and `component:booking-factory` are removed or repointed | `unit_root` keys |
| Bond refusals | Bond `"closed"`/`"not holder"` are mapped to the booking's documented refusal strings at the BookingContract boundary | `unit_bond` |
| Slot lookup fallback | If the `slotCaps` closure is not viable, per-slot `rl!` remains; this is a performance delta, not an observable behavior delta | `slot_closure_spike` |

## Non-deltas

- Terms hashes for v1 must be byte-identical.
- Remedy outputs for `fee`, `cancel`, `noShow`, `depositSplit` must match the legacy `Policy`.
- Owner derivation, admin payloads, opId idempotency, slot records, release tokens and
  terminal states must match.
