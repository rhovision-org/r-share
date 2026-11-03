# Events management and ticketing

> **Status:** Mapping only. The ticket arms are implemented; `unit_events` is planned (M5).

An event is capacity over a show time. The same listing → unit → slot → booking path applies;
what is new is the ticket projection and the redemption flow.

## The mapping

| Concept | Event choice |
|---|---|
| Listing | the event: name, venue, terms, sections |
| Unit | a seat or a tier section |
| Slot | the show or time window |
| Booking | a ticket order |
| Ticket | the booking's admission projection |

There are two workable seat models:

1. **Unit = seat.** Each seat is a unit; the slot is the show. This gives per-seat
   concurrency and a simple "sold" check.
2. **Unit = section.** Each section is a unit with many identical seats; the slot is the
   show. Capacity is implicit in the section's `attrs`, but two buyers can collide on the
   same section cell.

The first is the better fit for the primitives: it keeps one cell per bookable unit. A GA
area is still a set of ticket-numbered units, not a shared counter.

## Concrete example

```
listing:  "winter-show-2026", kind "event"
units:    GA-A:A1, GA-A:A2, GA-A:A3, VIP:V1, VIP:V2
slots:    "show-2026-12-12T19:00" on each unit (start/end heights)
terms:    pricePerSlot 50, deposit 0, minSlots 1, maxSlots 1,
          instantBook true, contiguous false,
          eventTransferCutoffBlocks 12, holdBlocks 10,
          noShowBlocks 0, feeBps 500
```

A guest books seat A1 for the show. The booking starts `confirmed` because `instantBook` is
true. The ticket is the booking's `ticket` projection: `holderId`, `redeemed`,
`redeemedAtHeight`, `transferCutoffHeight`.

## The event arms

| Arm | Who | Meaning |
|---|---|---|
| `redeem` | provider | admit the holder; exactly once; status `redeemed` |
| `transferTicket` | guest | move admission to a new holder before the cutoff, with deposit zero |
| `closeEvent` | anyone | after `checkOutHeight`, settle an unredeemed ticket |
| `cancel` | guest | before the cutoff: full refund; after: deposit only (zero here) |
| `claimNoShow` | provider | not the usual event path; use `closeEvent` or `redeem` |

`redeem` is provider-only because the venue controls the door. The ticket URI is the
credential; the venue's scanner checks the booking status and holder off-chain, then calls
`redeem`.

## Transfer rules

- before `ticket.transferCutoffHeight`;
- status `confirmed`;
- deposit must be zero, because a deposit claim belongs to the original payer;
- the guest (payer) authorizes the transfer;
- admission moves to the new holder; financial rights stay with the payer.

## Settlement

A redeemed event settles normally: rent minus fee to the provider, fee to the platform, the
deposit (zero or otherwise) back to the guest.

An unredeemed event can be closed after the show by anyone:

```rholang
BookingContract!("closeEvent", "ops-op-001", *ret)
```

The venue keeps the ticket price (minus fee), and the guest gets the deposit. This is the
no-show equivalent for events.

## What stays off-chain

- the seat map and accessibility seating;
- access control and scanning;
- age checks, concessions and comps;
- refunds outside the on-chain policy;
- event cancellation and rescheduling policy;
- marketing, pricing tiers and dynamic pricing.

## What to fork

- the `kind` value (`"event"`);
- `attrs` for the section, seat number, accessibility and view;
- `eventTransferCutoffBlocks` and `feeBps`;
- a ticket display and a door scanner;
- the `Policy.cancel` event branch if your refund arithmetic differs.

Do not add a new booking arm. The ticket is a projection, not a second state machine.

> **Where to go deeper.** [Bookings and lifecycle](../concepts/bookings-and-lifecycle.md)
> includes the event arms. [Data schemas](../reference/data-schemas.md) has the ticket
> shape. [Adding a new resource class](new-resource-class.md) is the extension recipe.
