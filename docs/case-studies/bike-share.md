# Municipal bike share

> **Status:** Mapping only. The shared path is implemented; `unit_generality` is planned (M7).

A bike share is a fleet of units at stations with hour windows. The chain books the bike and
holds the deposit; the lock, the fleet app and the rebalancing trucks stay off-chain.

## The mapping

| Concept | Bike share choice |
|---|---|
| Listing | the fleet or a bike class |
| Unit | a bike |
| Slot | a station + hour window |
| Booking | a ride |
| Ticket | the unlock credential |

## Concrete example

```
listing:  "harbour-bikes", kind "bike"
unit:     "bike-17"
attrs:    {"station": "harbour", "class": "city", "serialHash": "..."}
slots:    "bike-17:harbour:2026-11-05T09:00"
          startHeight 12000, endHeight 12012   (one hour, illustrative)
terms:    pricePerSlot 5, deposit 50, minSlots 1, maxSlots 1,
          instantBook true, contiguous false,
          holdBlocks 10, noShowBlocks 1, feeBps 500
```

A slot ID encodes the bike, the station and the window. The unit's `attrs` carry the station
and the serial hash; the chain does not verify either. The provider maps the hour to heights.

## The ride

1. The guest books the slot. The booking is `confirmed` (instant) or `requested`.
2. The guest checks in at the station; the provider (or an automated operator) calls
   `checkIn`.
3. The off-chain lock API verifies the booking status and the ticket holder, then unlocks
   the bike. The unlock credential is the booking URI or a short code derived from it; do
   not put it on-chain as a secret.
4. The guest returns the bike; the operator calls `checkOut`, opening the deposit review
   window.
5. `settle` pays the usage, the fee and the deposit.

`checkIn`/`checkOut` are the pickup/return pair. A late return is a policy or an extra
booking, not a new arm. Damage is a `claimDeposit` with an evidence hash, then `dispute` and
`arbitrate` if contested.

## What stays off-chain

- the lock protocol and the unlock secret;
- station capacity and rebalancing;
- battery state, maintenance and safety checks;
- the rider's identity and age;
- fines for improper parking;
- the municipal concession and pricing rules.

## What to fork

- `kind: "bike"`;
- `attrs` for station, class and serial hash;
- slot IDs and the height mapping for stations and hours;
- a lock integration that reads the booking and ticket;
- a fleet app for rebalancing and maintenance.

Do not add a booking arm for "unlock". The ticket is the credential; the lock is the
off-chain verifier.

## Why this rung matters

Bike share stresses per-unit availability, short windows and a deposit that has to survive a
damage claim. It is the cheapest way to prove the primitives before a car or a machine.

> **Where to go deeper.** [Robotics-as-a-service](robotics-raas.md) adds certification and
> safety. [Car rental](car-rental.md) adds inspection. [New resource
> class](new-resource-class.md) is the checklist.
