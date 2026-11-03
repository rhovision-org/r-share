# The use-case ladder

> **Status:** Holiday letting is the reference path; the other rungs are mappings over the
> same contracts.

The ladder is a way to grow the primitives without guessing. Each rung adds a real-world
constraint. The question at every rung is the same: does the listing → unit → slot → booking
path still fit, or does the model need a new primitive?

```
holiday letting  ->  events/ticketing  ->  bike share  ->  car rental  ->  RaaS
    nights              seats/shows         station-hours    vehicle-days    machine windows
    deposit             ticket              unlock proof     inspection      session evidence
    cancellation        transfer/redeem     fleet ops        insurance       operator + SLA
```

Figure: the ladder. The left side is the simplest interval; the right side has the most
off-chain dependencies.

## The rungs

| Rung | Unit | Slot | What is new | What stays |
|---|---|---|---|---|
| Holiday letting | room | night | cancellation windows, deposit, damage | booking, escrow, settlement |
| Events | seat or section | show/time window | ticket, redeem, transfer cutoff | booking, escrow, settlement |
| Ticketing | ticket tier | event | admission proof, resale rules | booking, escrow, settlement |
| Bike share | bike | station + hour window | unlock credential, fleet rebalancing | booking, escrow, settlement |
| Car rental | vehicle | day or time window | inspection, insurance, damage deposit | booking, escrow, settlement |
| RaaS | machine | operating window | certification, telemetry, SLA, safety | booking, escrow, settlement |

## What changes and what does not

A new rung should change only:

- `kind` — `"lodging"`, `"event"`, `"bike"`, `"car"`, `"robot"`;
- `attrs` — free-form data the off-chain product needs (station, seat map, plate,
  certification hash, operating envelope hash);
- slot IDs and slot windows — `room-1:2026-11-05`, `GA-A:12`,
  `bike-17:2026-11-05T09:00`, `car-7:2026-11-05`, `arm-3:2026-11-05T14:00`;
- a pure `Policy` branch if the cancellation or no-show arithmetic differs;
- the off-chain product around it.

A new rung should **not** change the booking state machine, the escrow mechanics, the
settlement buckets, or the authority model. If it does, the primitive is wrong, and the
change belongs in a fork or in a new primitive discussed with the community.

## How to choose your fork point

- **Same business shape, different inventory?** Fork the repo, add a `kind` and `attrs`,
  keep the contracts. This is the bike/car/robot path.
- **Different cancellation arithmetic?** Add a pure branch to
  [`Policy`](../reference/arms.md), keep the booking arms.
- **Different money flow (subscriptions, usage metering, revenue splits)?** That is a new
  primitive. Treat it as a design discussion, not a patch.
- **Different trust model (custodial, regulated, insured)?** Keep r-share as the settlement
  core and put the regulated layer off-chain.

## The sequence to follow

1. Model the inventory as units and the bookable time as slots.
2. Write the terms: price per slot, deposit, minimum and maximum, cancellation windows,
   lead time, hold time.
3. Prove the mapping in a fixture and a case-study chapter.
4. Only then write the off-chain product.

The [holiday-letting walkthrough](../case-studies/holiday-letting/index.md) is the reference
implementation of that sequence. The four mapping chapters apply it to the higher rungs.

> **Where to go deeper.** [Capacity over an interval](../concepts/capacity-over-an-interval.md)
> is the model in one page. [Adding a new resource class](../case-studies/new-resource-class.md)
> is the checklist. [Robotics-as-a-service](north-star.md) is the ceiling.
