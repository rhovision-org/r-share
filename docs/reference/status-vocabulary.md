# Status vocabulary

> **Status:** Implemented. The chain values are parsed from source; the client operation
> values are the SDK's deploy lifecycle.

## On-chain

| Domain | Values |
|---|---|
| Listing | `draft`, `active`, `paused`, `delisted` |
| Unit | `active`, `paused` |
| Slot | `free`, `held`, `booked` |
| Provider | `active`, `paused` |
| Booking | `requested`, `confirmed`, `checkedIn`, `checkedOut`, `redeemed`, `settled`, `rejected`, `cancelled`, `expired`, `noShow`, `noShowClaimed`, `noShowDisputed`, `noShowRefunded`, `disputed` |

Terminal booking values are `settled`, `rejected`, `cancelled`, `expired`, `noShow` and
`noShowRefunded`. `checkedOut` is not terminal; settlement is still pending. `redeemed`
is not terminal: an event can still be settled after it ends. `noShowClaimed` and
`noShowDisputed` are contestable states, not terminal outcomes.

The provider listing index stores a copied `status` that is set at registration. Treat the
listing's own `status` as authoritative; the client reads it.

## Client operation

| Value | Meaning |
|---|---|
| `drafted` | op id recorded; not submitted |
| `submitted` | deploy id known; not processed |
| `processed` | node processed; not final |
| `finalized` | containing block is final; the only success state |
| `refused` | deploy succeeded, contract replied `(false, reason)` |
| `failed` | deploy executed and errored; rolled back; phlo spent |
| `unknown` | the node did not settle it; do not resubmit blindly |

The activity journal uses these values. It is a device log, not a chain index.

## Refusal classes

`RefusalCode` groups the published vocabulary: authority (`not owner`), state (`closed`,
`replay`, `pending`), slots (`slot held`, `slot booked`, `slot lost`, `unknown slot`),
gates (`not active`, `paused`, `provider paused`, `unit paused`), validation (`terms`,
`slots`, `schedule`, `window`, `price`, `bps`, `amount`, `threshold`, `duplicate key`,
`reservation`, `listing`), rail (`payments unavailable`, `account`,
`bond factory unavailable`), time (`too early`, `too late`) and authority count (`below
threshold`).

A refusal is a designed answer, not a crash. Keep it verbatim and offer the next action.

> **Where to go deeper.** [Bookings and lifecycle](../concepts/bookings-and-lifecycle.md)
> is the transition model. [Client reads](client-reads.md) explains finality. [Arm
> reference](arms.md) lists every refusal.
