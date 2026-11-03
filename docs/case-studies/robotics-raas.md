# Robotics-as-a-service

> **Status:** Mapping only for the booking path; RaaS-specific pieces are design intent.
> `unit_generality` is planned (M7).

RaaS is the north star. The machine is a unit, the operating window is a slot, and the
session is a booking. What makes RaaS harder than car rental is not the booking; it is the
operator, the envelope, the evidence, and safety.

## The mapping

| Concept | RaaS choice |
|---|---|
| Listing | a robot model or an operator's fleet |
| Unit | a machine, and optionally an operator |
| Slot | an operating window |
| Booking | a session or job |
| Ticket | the session credential |
| Attrs | certification, envelope, SLA, telemetry contract |

## Booking a machine and an operator atomically

A RaaS session usually needs two scarce resources: the machine and a certified operator.
The booking supports multiple units in one `unitSlots` list (up to `maxUnits`). Book both in
one deploy, so a machine is never held without an operator:

```text
unitSlots = [
  {"unitId": "drone-3",    "slots": ["field-7:2026-11-05T06:00"]},
  {"unitId": "pilot-2",    "slots": ["field-7:2026-11-05T06:00"]}
]
```

The per-listing allocator writes both reservations in one commit: both slots are held or
neither is. The release token is shared, so a partial session cannot
be released into a half-booked state.

## Concrete example

```
listing:  "spray-drone-service", kind "robot"
unit:     "drone-3"
attrs:    {"model": "S3", "certificationHash": "...", "envelopeHash": "...",
           "operatorUnit": "pilot-2", "slaHash": "..."}
slots:    "drone-3:field-7:2026-11-05T06:00", start 12000, end 12012
terms:    pricePerSlot 500, deposit 2000, minSlots 1, maxSlots 1,
          instantBook false, contiguous false,
          holdBlocks 20, depositReviewBlocks 288, disputeWindowBlocks 288,
          noShowBlocks 3, feeBps 500
```

The certification, envelope, SLA and operator references are hashes. The chain does not know
whether the pilot is current or the airspace is open.

## The session

1. The customer requests the window; the operator accepts after checking weather, airspace
   and certification off-chain.
2. At the window the operator calls `checkIn`; the session credential is the booking URI.
3. The machine runs under the operator's control. Telemetry, video and incident logs stay
   off-chain.
4. At the end the operator calls `checkOut` and the deposit review window opens.
5. A damage or crop-loss claim is `claimDeposit` with an evidence hash; a contested claim is
   `dispute` then `arbitrate`.

A cancelled window because of weather is a `cancel` inside the full-refund window, or a
provider `cancelByProvider` with a full refund. The policy owns the arithmetic; the operator
owns the decision.

## What stays off-chain

- operator licensing, training and currency;
- airspace, weather and site permissions;
- the machine's flight or motion controller;
- telemetry, video, sensor logs and incident reports;
- the remote stop / kill switch;
- insurance, liability and the safety case;
- usage metering and SLA credits, until a metering adapter exists.

## What to fork

- `kind: "robot"`;
- `attrs` for certification, envelope, operator and SLA hashes;
- two-unit bookings for machine + operator;
- a session/telemetry app and an evidence store;
- a policy branch for weather cancellation;
- a pause/delist runbook and an emergency arbitration path.

## Why the primitive survives

The booking path does not need a "robot mode". It needs multiple units, a window, a deposit,
a claim and an arbiter. Those already exist. The RaaS work is the off-chain safety case and
the evidence around the booking.

> **Where to go deeper.** [The north star](../orientation/north-star.md) is the mission.
> [Car rental](car-rental.md) is the closest implemented rung. [Governance and
> arbitration](../concepts/governance-and-arbitration.md) is the dispute path.
