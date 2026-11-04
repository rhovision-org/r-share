# Who this is for

> **Status:** Orientation. No verification claim.

r-share is for people who want a booking and escrow layer they can own, audit and fork,
rather than a marketplace they rent. The same primitives serve several audiences; the
reading path changes, the contracts do not.

## The forker and the community builder

You want to build a rental or access product without rebuilding availability, holds, escrow,
cancellation and settlement. You are comfortable reading Rholang and Python, or you are
willing to copy the patterns and let a developer finish the integration.

Start with [Fork and run](../guides/fork-and-run.md), then
[Adding a new resource class](../case-studies/new-resource-class.md). Read the
[contract map](../reference/contract-map.md) before changing anything. Keep the AGPL notices
and read [Provenance and license](../appendix/provenance-and-license.md).

## The letting operator or host

You have rooms, cabins or apartments and want direct bookings without a platform taking a
cut and owning the relationship. You care about deposits, cancellation windows, damage
claims and disputes.

Read the [holiday-letting walkthrough](../case-studies/holiday-letting/index.md) end to end.
You can start from the reference client, but a production product still needs an off-chain
booking UI, a calendar, a payments on-ramp, support, and local
consumer-law compliance.

## The event organiser

You sell seats or capacity for shows, workshops or festivals. You care about capacity,
cutoffs, transferable tickets and redemption at the door.

Read [Events management and ticketing](../case-studies/events-and-ticketing.md). The ticket
is a projection of the booking; the venue still owns access control, scanning and refunds
policy.

## The municipal or fleet team

You operate bikes, scooters or cars at stations or depots. You care about unit-level
availability, hour or day windows, deposits, maintenance and rebalancing.

Read [Municipal bike share](../case-studies/bike-share.md) or
[Car rental](../case-studies/car-rental.md). The unlock credential and the inspection record
are off-chain; the booking and the money are on-chain.

## The RaaS builder

You operate machines and certified operators. You care about operating windows,
certification, telemetry, SLAs and safety. You are building toward the
[north star](north-star.md).

Read [Robotics-as-a-service](../case-studies/robotics-raas.md) and
[Governance and arbitration](../concepts/governance-and-arbitration.md). Plan for the
off-chain safety case first; the chain is the settlement layer, not the safety layer.

## The protocol contributor

You want to improve the primitives themselves: concurrency, escrow, policy, authority or
the test harness. Read [Concurrency doctrine](../concepts/concurrency-doctrine.md),
[Authority and refusal](../concepts/authority-and-refusal.md), and
[Test your fork](../guides/test-your-fork.md). The rules in `AGENTS.md` and the static gates
are the contract.

## What every reader should know

- The chain is the backend. This repository ships a reference client, but no backend
  server, indexer or off-chain database.
- "Implemented" means the contract path exists and passes the static and parser gates, not
  that a behavioural suite has passed on a healthy chain.
- The project is devnet-only and unaudited. Do not put real value on it.
- A fork owns its off-chain obligations: identity, calendars, telemetry, support, insurance,
  tax, consumer protection, and safety.

> **Where to go deeper.** [Status, scope and honest limits](status-and-scope.md) defines the
> labels. [Why primitives](why-primitives.md) explains the stance. [The use-case
> ladder](use-case-ladder.md) places your product.
