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

## r-cash caveat

The pinned r-cash installer currently stops at its `51_pool` step because the node reports
`parse depth exceeded` for that term. `make rcash-bootstrap` tolerates that stop, runs the
r-cash `09_identity` step to install the account factory, and repairs
`deployments/devnet-v4.json` from the install state. r-share needs only the r-cash Root,
clock, asset resolver and account factory; the AMM pool is not on its path.

## What is still not run

- claim-class scenarios (`buyerUseBreach`, `damage`, `noShow`, `noFaultCancellation`);
- `providerDeliveryBreach` and `partialPerformance`, which have no code;
- concurrency races, conservation, pause, admin, events and generality;
- any scenario on bike, car or robot fixtures.

Do not describe those as passing. The coverage manifest records the real status.
