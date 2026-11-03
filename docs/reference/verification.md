# Verification record

> **Status:** One recorded behavioural run. It covers the root directory, the r-cash adapter,
> and the dispute/lifecycle fixes. It does not cover claim classes, concurrency races,
> conservation, pause, or the other resource classes.

## Recorded run

| Field | Value |
|---|---|
| Date | 2026-11-03 |
| r-share commit | the commit that adds this chapter (`M2: run on the pinned node…`) |
| rholo | `e2d073dcd02f1e560ccb84749937138a1e184b0b` |
| r-cash | `49b441d` |
| Node | fresh local devnet, `localhost:40403` |
| Command | `make devnet-up keys rcash-bootstrap install && make unit-all` |

Results:

```
unit-root      pass   keys, resolve, unknown key, stale version
unit-payments  pass   health, platform owner, REV issuer, unknown asset
unit-lifecycle pass   dispute from requested refuses;
                      arbitrate outside dispute refuses;
                      checked-out booking resolves through resolveDispute
unit-claims    pass   four claim classes x five kinds (20/30 coverage cells)
unit-pricing   pass   units store 10 and 20; a zero price refuses "price"
unit-guestbook pass   getOrCreate is stable; ledger read-back matches
```

The runner writes `deployments/suite-results.json` (gitignored) and exits 0 only when every
selected suite passes.

## What the lifecycle suite proves

- `dispute` from a `requested` booking refuses `"closed"`; it cannot freeze escrow.
- `arbitrate` on a non-disputed booking refuses `"closed"`; the 2-of-3 set cannot bypass the
  state machine.
- A `confirmed` booking can `checkIn`, `checkOut`, `dispute`, and then any caller can
  `resolveDispute` once the dispute window has elapsed. The default pays the deposit to the
  guest and rent net of fee to the provider.

The suite uses a stub Bond and an empty unit set. It is a state-machine test, not a
marketplace scenario; it does not prove slot settlement or ERTP conservation.

## What the claims suite proves

- `cancel` from a requested booking returns the full rent and deposit.
- `claimNoShow` after the no-show window pays the provider rent net of fee and releases the
  deposit.
- `claimDeposit` followed by `resolveDeposit` splits the deposit between provider and guest.

The same three scenarios run for lodging, event, bike, car and robot. They exercise the
remedy arithmetic and the booking state machine; they do not exercise a real Bond, slots,
or ERTP conservation.

## What the pricing suite proves

Two units created with prices 10 and 20 report those prices from `UnitContract("info")`,
and a price of zero refuses `"price"`. It does not yet prove that the guest ledger sums
per-unit prices into a booking's rent; that needs a full listing/unit/booking scenario.

## What the guestbook and evidence checks prove

`GuestBook("getOrCreate")` returns the same ledger on a second call and `GuestBook("ledger")`
reads it back; the existing-guest path no longer consumes and rewrites the index.
`BookingContract("evidence")` returns the dispute hash and height after a dispute, so the
anchor is readable rather than write-only.

## r-cash caveat

The pinned r-cash installer currently stops at its `51_pool` step because the node reports
`parse depth exceeded` for that term. `make rcash-bootstrap` tolerates that stop, runs the
r-cash `09_identity` step to install the account factory, and repairs
`deployments/devnet-v4.json` from the install state. r-share needs only the r-cash Root,
clock, asset resolver and account factory; the AMM pool is not on its path.

## What is still not run

- `providerDeliveryBreach` and `partialPerformance`, which have no code;
- concurrency races, conservation, pause and admin;
- event-only ticket arms (`redeem`, `closeEvent`, `transferTicket`);
- slot settlement and ERTP conservation (the claim fixture uses a stub Bond);
- an end-to-end booking whose rent is the sum of different per-unit prices.

Do not describe those as passing. The coverage manifest records the real status.
