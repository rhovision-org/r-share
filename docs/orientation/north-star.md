# The north star: robotics-as-a-service

> **Status:** Design intent. The RaaS-specific pieces below are not on-chain in r-share yet;
> the shared booking and escrow path is.

Robotics-as-a-service is the ceiling this repository is pointed at. A customer does not buy a
machine; they book a machine's capacity for an interval, under a service agreement, with
payment released when the work is done. The interesting part is not the robot. It is the
booking, the operator's accountability, the money held between the parties, and the evidence
that the session happened.

## A concrete RaaS booking

A farm co-operative wants a crop-spraying drone for a two-hour window on Thursday. The
operator has three drones and two certified pilots. The co-operative needs:

- a machine and a pilot available in the same window;
- an operating envelope (field, weather limits, no-fly zones);
- proof that the pilot is certified for that machine;
- a price per hour and a damage deposit;
- a record of the session (start, end, area covered, incidents);
- a refund if the weather cancels the window;
- an arbitration path if the crop is damaged.

r-share models the bookable part of that as a listing (`kind: "robot"`), units (the
machines), slots (operating windows), a booking with rent + deposit in escrow, and a policy
for cancellation, no-show and damage. The pilot certification, weather, telemetry, and
physical safety remain off-chain facts that the operator and the customer rely on.

## What r-share provides, and what it does not

| Concern | r-share | Off-chain |
|---|---|---|
| Machine availability | unit + slot state | dispatch and maintenance schedules |
| Operating window | `startHeight` / `endHeight` | weather, airspace, site access |
| Operator certification | `attrs` on the unit or listing | registry, training, insurance checks |
| Price, fee, deposit | terms and settlement buckets | pricing strategy, invoicing, tax |
| Session record | booking status, ticket, claim | telemetry, video, audit logs |
| Cancellation and no-show | policy evaluated on-chain | goodwill and support |
| Damage | deposit claim, dispute, arbitration | inspection, repair quotes, liability |
| Remote stop / kill switch | out of scope | machine and operator safety systems |
| SLA penalties | design intent | contract law and operations |

The boundary is intentional. A chain is good at holding value and enforcing a deterministic
policy. It is bad at knowing whether it rained, whether a pilot is competent, or whether a
robot is safe to fly.

## Why this is the north star

RaaS stresses every primitive at once:

- **Capacity is scarce and multi-dimensional.** A machine is only bookable if the machine,
  the operator and the site are all available.
- **The interval matters more than the item.** A drone without its window is useless.
- **Money is held against future performance.** Deposits, partial refunds and damage claims
  are normal, not exceptional.
- **Evidence is contested.** The session record and the inspection record decide disputes.
- **Safety is non-negotiable.** A booking primitive that cannot be paused, delisted or
  arbitrated is not ready for machines that move.

If the primitives survive holiday letting, events, bike share and car rental, the remaining
RaaS work is mostly off-chain: certification, telemetry, and the operator's safety case.

## What would move RaaS from design intent to implemented

1. A `kind: "robot"` fixture that books a machine window on the shared path (the M7
   generality proof).
2. Unit `attrs` carrying certification and operating-envelope hashes, with a documented
   off-chain verifier.
3. A session-evidence arm or an external evidence contract that the deposit claim can
   reference.
4. A pause/delist runbook and a 2-of-3 emergency path, exercised in the admin suite.
5. A metering adapter for usage-based pricing, if per-hour pricing is not enough.

None of these requires a new booking arm. That is the test of the primitive model.

> **Where to go deeper.** [The use-case ladder](use-case-ladder.md) places RaaS at the top
> of the progression. [Adding a new resource class](../case-studies/new-resource-class.md)
> is the extension recipe. [Robotics-as-a-service](../case-studies/robotics-raas.md) maps
> the concrete fields and the off-chain boundary.
