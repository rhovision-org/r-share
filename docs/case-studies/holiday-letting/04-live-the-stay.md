# Live the stay

> **Status:** Implemented, not yet exercised.

The stay is a sequence of explicit, height-windowed actions. There are no timers and no
background jobs; someone calls each arm.

## Check in

```rholang
BookingContract!("checkIn", *deployerId, "ana-op-002", *ret)
```

Ana must call it at or after `checkInHeight` and at or before `checkOutHeight`. Status goes
from `confirmed` to `checkedIn`. A call too early refuses `"too early"`; too late refuses
`"too late"`.

The chain records `checkInAtHeight` for the audit trail. It does not check whether Ben
actually arrived; that is Ana's operational decision.

## Check out

```rholang
BookingContract!("checkOut", *deployerId, "ana-op-003", *ret)
```

Ana calls it after check-in. Status becomes `checkedOut`, and the contract sets
`depositReleaseHeight = now + depositReviewBlocks`. The deposit is not released yet: Ana has
the review window to inspect the room and, if needed, claim against the deposit.

If Ana never checks out, Ben can call:

```rholang
BookingContract!("guestCheckOut", *deployerId, "ben-op-004", *ret)
```

after `checkOutHeight`. This is the guest's escape hatch; it opens the same review window.

## Cancellation before the stay

Ben can cancel while the booking is `requested` or `confirmed`:

```rholang
BookingContract!("cancel", *deployerId, "ben-op-005", *ret)
```

`Remedy.cancel` evaluates the terms against `now` and `checkInHeight`:

- before `checkInHeight - 48`: full rent + deposit back;
- before `checkInHeight - 12`: 50% of rent + deposit back;
- later: deposit only;
- the platform fee is zero on a cancellation.

The contract releases the slots and pays the buckets. Status becomes `cancelled`.

A booking that is already `checkedIn` cannot be cancelled; it must be checked out. That is
deliberate: a stay in progress is settled by the checkout and deposit path, not by a
cancellation refund.

## Provider cancellation

```rholang
BookingContract!("cancelByProvider", *deployerId, "ana-op-006", *ret)
```

Ana can cancel a `requested` or `confirmed` booking. The guest receives a full refund
(rent + deposit), and the slots are released. This is the "something broke at the house"
path; it costs Ana the rent but keeps the relationship honest.

## No-show

If Ben never checks in, Ana can claim after the grace window:

```rholang
BookingContract!("claimNoShow", *deployerId, "ana-op-007", *ret)
```

Requires status `confirmed` and `now >= checkInHeight + noShowBlocks`. The buckets are:
rent minus fee to Ana, fee to the platform, deposit back to Ben. The slots are released.
Status becomes `noShow`.

## Expiry

If Ana never accepts, Ben can recover his money after the hold:

```rholang
BookingContract!("expire", "ben-op-008", *ret)
```

Requires status `requested` and `now >= holdUntilHeight`. Full rent + deposit back; the
slots are released. Status becomes `expired`.

`expire` is permissionless. Anyone can finish a stalled request, but the refund always goes
to the guest.

## The state at each step

```
requested --accept--> confirmed --checkIn--> checkedIn --checkOut--> checkedOut --settle--> settled
    |                     |                       |
    +--reject             +--cancel               +--dispute
    +--expire             +--cancelByProvider
                          +--claimNoShow
```

Figure: the lodging lifecycle. `settle` is covered in the next chapter.

> **Where to go deeper.** [Bookings and lifecycle](../../concepts/bookings-and-lifecycle.md)
> is the full machine. [Time without clocks](../../concepts/time-without-clocks.md) explains
> the windows. [Settle, dispute and clean up](05-settle-and-handle-failure.md) is next.
