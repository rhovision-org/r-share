# r-share

On-chain booking and rental primitives for the rholo chain, settled through
[r-cash](https://github.com/rhovision-org/r-cash).

r-share models a rental as **capacity over an interval**: a provider publishes a listing,
a listing owns units, and each unit owns private per-slot cells. Lodging (room-nights),
events (seats), bike rental (bike + station + hour window), car rental (vehicle + window),
and robotics rental (machine + operating window) are all the same primitive with different
`kind`, `attrs`, and slot IDs. Booking, cancellation, escrow, deposit, dispute, and ticket
logic is shared.

The chain is the backend. There is no server, no UI, and no off-chain database in this repo.

## Status

Devnet-only, unaudited, and coupled to r-cash v0.5 at commit `49b441d`. Do not put real
value on it.

The contracts for the full lodging/event path are implemented and pass the static and
parser gates. The behavioural campaign is being built milestone by milestone:
`unit_root` and `unit_payments` run today; the provider, listing, booking, concurrency,
lifecycle, event, admin and generality suites are scheduled in `docs/TESTING.md` and exit
with their milestone rather than reporting a pass.

## What is here

| Path | Contents |
|---|---|
| `contracts/src/deploy/` | Deployable Rholang terms (root, kernel, admin, provider, listing, policy, booking, payments) |
| `contracts/src/lib/` | Fragments inlined at build time (`Auth`, `Guard`, `Terms`, `Money`) |
| `harness/` | Python devnet installer, fixtures, scenarios, and behavioural suites |
| `tools/` | Build inliner, deploy-size and concurrency lints, keygen, signer |
| `docs/` | Primitives, concurrency doctrine, payments, security, testing |
| `spec/` | Acceptance map and invariants |

## Quickstart

```sh
make venv
make spec-check          # static gates, no chain needed
make devnet              # r-cash devnet on rholo (needs Docker + the sibling checkouts)
make rcash-install       # install r-cash v4 components
make keys install        # install r-share
make unit-all            # behavioural campaign
```

The devnet expects sibling checkouts at `../Rholphin` (r-cash) and `../RNodeRust`
(rchain-rust). Override with `RCASH_DIR` and `RHOLO_DIR`.

## Design rules

- Every public arm replies `(true, value)` or `(false, reason)`. Refuse before value moves;
  after an ERTP operation an unexpected refusal calls `1/0` to roll back the deploy.
- One state cell per component. Consume, mutate, repost. Every refusal leaves the cell Full.
- One private single-datum cell per slot. Disjoint slots commit independently.
- No global counters. Ids are client- or provider-supplied and uniqueness-checked locally.
- `select` is forbidden: the node desugars it to parallel receives, so it is not a choice.
- All deadlines are block heights. `rho:block:data`'s timestamp is not consensus.

See `docs/CONCURRENCY.md` for the full doctrine.

## License

AGPL-3.0-or-later. See `LICENSE` and `THIRD_PARTY_NOTICES.md`.
