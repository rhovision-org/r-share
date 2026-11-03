# Invariants

> **Status:** Implemented as the repository's invariant list. The behavioural suites that
> check them are partial.

The following list is the single source of truth in `spec/invariants.md`. This chapter
includes it so the book and the repository cannot drift.

{{#include ../../spec/invariants.md}}

## How the invariants are used

- `unit_booking` checks the escrow debit, the refund and the Bond closure for one real
  booking: the account loses exactly `rent + deposit`, the refund restores it, and the Bond
  ends paid with zero remaining. `harness/conservation.py` is still a stub, so the full
  per-asset invariant set is not yet automated.
- The M8 hardening milestone adds property tests and a soak test across disjoint slots.
- A review that changes a money path must say which invariant it preserves and how.
- A failure is a bug in the contract, not a flaky test; fix the contract or the test.

## The four that catch most bugs

1. **Cell fullness.** A refusal that does not repost the state cell leaves an arm dead.
2. **Slot state.** A slot that is both free and booked is a double booking.
3. **Bucket sum.** A settlement that pays less or more than `rent + deposit` is a loss.
4. **Token match.** A release with the wrong token is a stolen slot.

> **Where to go deeper.** [Test your fork](../guides/test-your-fork.md) runs the suites.
> [Bookings and lifecycle](../concepts/bookings-and-lifecycle.md) is the transition model.
> [Escrow and settlement](../concepts/escrow-and-settlement.md) is the bucket model.
