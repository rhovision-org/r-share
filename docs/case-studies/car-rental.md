# Car rental

 > **Status:** The shared path is implemented and `unit_generality` passes for the kind. Inspection and insurance are mapping only.

Car rental is bike share with a bigger deposit, an inspection, and an insurance layer. The
booking and escrow path is unchanged.

## The mapping

| Concept | Car rental choice |
|---|---|
| Listing | a vehicle class or a depot |
| Unit | a vehicle |
| Slot | a day or a time window |
| Booking | a rental |
| Ticket | the key/fob handover credential |

## Concrete example

```
listing:  "coast-cars", kind "car"
unit:     "car-7"
attrs:    {"plateHash": "...", "class": "estate", "fuel": "electric",
           "insuranceHash": "...", "inspectionTemplate": "..."}
slots:    "car-7:2026-11-05", startHeight 12000, endHeight 12288
          "car-7:2026-11-06", startHeight 12288, endHeight 12576
terms:    pricePerSlot 200, deposit 500, minSlots 1, maxSlots 14,
          instantBook false, contiguous true,
          holdBlocks 20, depositReviewBlocks 96, disputeWindowBlocks 96,
          noShowBlocks 6, feeBps 500
```

The unit's `attrs` carry hashes, not the insurance policy or the inspection report. The
off-chain product owns the documents and the verification.

## The rental

1. The guest requests the booking; the provider accepts after checking the licence and
   insurance off-chain.
2. At pickup the provider calls `checkIn` and hands over the key or fob. The ticket is the
   handover credential; the physical key is off-chain.
3. At return the provider inspects the vehicle and calls `checkOut`. The review window is
   longer than lodging because repair quotes take time.
4. The provider can `claimDeposit` with an inspection evidence hash inside the review
   window; the guest can `dispute`; the 2-of-3 arbiter splits.
5. `settle` pays the rental, the fee and the deposit remainder.

A late return is an extra slot, a policy fee, or an off-chain charge, depending on the
product. Do not add a booking arm for it.

## What stays off-chain

- driving licence and insurance verification;
- the physical key, immobiliser or app unlock;
- fuel, tolls, fines and congestion charges;
- the inspection report and repair quotes;
- roadside assistance and recovery;
- the regulated rental agreement.

## What to fork

- `kind: "car"`;
- `attrs` for plate hash, class, insurance hash and inspection template;
- slot IDs and depot windows;
- a handover/inspection app;
- a policy branch if the cancellation or late-return arithmetic differs.

## Why this rung matters

Car rental adds a long deposit review window, an inspection artifact and a regulated
off-chain layer. It is the closest lower rung to RaaS: the machine is expensive, the user
needs a credential, and the damage process is contested.

> **Where to go deeper.** [Bike share](bike-share.md) is the simpler fleet case. [Robotics-as-
> a-service](robotics-raas.md) adds operator certification. [Governance and
> arbitration](../concepts/governance-and-arbitration.md) is the dispute path.
