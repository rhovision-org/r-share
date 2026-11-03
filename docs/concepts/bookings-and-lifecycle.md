# Bookings and lifecycle

> **Status:** Implemented. **Applies to:** all resource classes.

A booking is a per-booking contract with its own state cell, escrow purse, release token,
ticket projection and operation map. Its arms are the lifecycle. Every mutating arm consumes
the cell, decides, and reposts exactly once.

## The state machine

```
requested --accept--> confirmed --checkIn--> checkedIn --checkOut--> checkedOut --settle--> settled
    |                     |                       |                          |
    +--reject             +--cancel               +--dispute                 +--dispute
    +--expire             +--cancelByProvider                                 +--claimDeposit
                          +--claimNoShow
                          +--redeem  (events) --> redeemed --settle--> settled
```

Figure: booking transitions. A terminal state is permanent; a replay returns the stored
result.

## The arms

| Arm | Who | What it does |
|---|---|---|
| `accept` | provider | confirms the held slots, status `confirmed` |
| `reject` | provider | releases slots, full refund, status `rejected` |
| `cancel` | guest | evaluates the cancellation policy, releases, pays, status `cancelled` |
| `cancelByProvider` | provider | releases, full refund, status `cancelled` |
| `checkIn` | provider | inside the slot window, status `checkedIn` |
| `checkOut` | provider | sets the deposit review window, status `checkedOut` |
| `guestCheckOut` | guest | after `checkOutHeight`, same transition |
| `settle` | anyone | after review, pays rent/fee/deposit, status `settled` |
| `claimDeposit` | provider | records a damage claim up to the deposit |
| `resolveDeposit` | anyone | after the dispute window, pays the claim and the rest |
| `dispute` | guest or provider | status `disputed` |
| `arbitrate` | 2-of-3 admins | splits the deposit, status `settled` |
| `expire` | anyone | after `holdUntilHeight`, full refund, status `expired` |
| `claimNoShow` | provider | after the grace window, rent + fee, deposit refunded |
| `redeem` | provider | events only, marks the ticket used, status `redeemed` |
| `closeEvent` | anyone | events only, after the event, settles |
| `transferTicket` | guest | events only, before cutoff and with no deposit |
| `setAdmins` | current 2-of-3 | rotates the arbiter set |

The canonical signatures are in the [arm reference](../reference/arms.md).

## The reads

`info`, `status`, `ticket` and `ops` are peeks. They never consume the cell. A guest, a
provider or an indexer can read the booking without blocking a lifecycle action.

## Exactly once

Every mutating arm takes an `opId`. The booking records a successful result by `opId`; a
replay returns it and moves no money. The state-cell consume is the serialization point, so
two competing arms in the same block resolve to exactly one transition.

The `GuestLedger` performs the same trick for the booking itself: it reserves the `opId`
before any external call and records the result afterwards. A duplicate book request returns
the same booking URI and moves no money.

## Money at each transition

- **Request.** Rent + deposit are already in the escrow purse. Slots are held.
- **Accept.** Slots become booked; money does not move.
- **Cancel / reject / expire / provider cancel.** Slots release; the policy computes the
  refund; buckets are paid.
- **Check-out.** Money does not move; the deposit review window opens.
- **Settle.** Rent minus fee to the provider, fee to the platform, deposit to the guest.
- **Claim / dispute / arbitration.** The deposit is split; rent and fee follow the normal
  settlement.
- **No-show.** Rent minus fee to the provider, fee to the platform, deposit to the guest.

Every payout asserts that the buckets sum to `rent + deposit`. A bug in an arm fails the
deploy instead of misallocating funds.

## Failure paths are first-class

- A provider who never accepts: the guest expires the booking after the hold.
- A guest who never checks in: the provider claims a no-show after the grace window.
- A slot reclaimed while a booking waited: `accept` refuses `"slot lost"`; the guest
  expires or cancels.
- A damage claim: the provider records it, the guest disputes it, the arbiter splits it.

> **Where to go deeper.** [Escrow and settlement](escrow-and-settlement.md) explains the
> money. [Authority and refusal](authority-and-refusal.md) explains who may call each arm.
> [Arm reference](../reference/arms.md) is the signature list.
