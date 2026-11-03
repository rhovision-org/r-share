# Capacity over an interval

> **Status:** Implemented. **Applies to:** all resource classes.

A rental is not an object. It is the right to use some capacity for a bounded interval. A
room for a night, a seat for a show, a bike for an hour, a car for a day, a machine for an
operating window. Once you state the problem that way, the same primitives cover all of them.

## The model

```
provider --publishes--> listing --owns--> units --have--> slots
   ^                                                       |
   |                                                       v
   +------------------ settlement <-- booking <-- guest books
```

Figure: the four nouns and the booking path.

- **Provider.** The party that owns or operates the capacity. Identified by a REV address
  derived from the deploy key.
- **Listing.** A published offer: a kind, terms, a status, and a set of units.
- **Unit.** The thing that is booked: a room, a seat, a bike, a vehicle, a machine. A unit
  has attributes and a schedule of slots.
- **Slot.** The atom of capacity: a named interval. It has a start height, an end height, a
  price through the listing terms, and a state (`free`, `held`, `booked`).
- **Booking.** A guest's claim on one or more slots, with rent and deposit in escrow, moving
  through a lifecycle.
- **Settlement.** The deterministic split of the escrow into provider rent, platform fee,
  guest refund and provider damage.

## Why capacity, not inventory

Inventory says "one room exists". Capacity says "one room is usable by one party in this
interval". The difference matters:

- Two bookings on different nights of the same room are independent. They should not
  serialize.
- Two bookings on the same seat of the same show conflict. Exactly one must win.
- A bike at a station for an hour and the same bike at another station for the next hour are
  two slots, not a mutable row.
- A machine with an operating window is bookable only when the window is open.

Modeling capacity as a set of independent slots is what lets the chain arbitrate with one
consumed datum per slot instead of a global lock.

## The shared shape

Every class is the same shape with different labels:

| Concept | Lodging | Events | Bike | Car | Robot |
|---|---|---|---|---|---|
| Listing | property/room type | event | fleet/class | vehicle class | robot model/class |
| Unit | room | tier/section | bike | vehicle | machine |
| Slot | room-night | seat/show | station-hour | vehicle-day | operating window |
| Booking | stay | ticket order | ride | rental | session/job |
| Ticket | check-in pass | seat ticket | unlock credential | key/fob | session credential |

A new class adds only `kind`, `attrs`, and a pure policy branch. It does not add booking
arms. If it needs a new arm, the model is wrong or the requirement is a new primitive.

## What the model deliberately excludes

- **The calendar UI.** Slot IDs are opaque strings; the provider maps dates to heights.
- **The item's identity.** A unit has attrs, not an on-chain NFT. A bike's serial number or a
  robot's certification hash is off-chain metadata.
- **Telemetry.** Usage metering, GPS, video and sensor logs are off-chain.
- **Insurance and regulation.** The chain holds money and enforces policy, not liability.

> **Where to go deeper.** [Listings, units and slots](listings-units-slots.md) gives the
> concrete types. [Time without clocks](time-without-clocks.md) explains the interval.
> [The use-case ladder](../orientation/use-case-ladder.md) is the progression.
