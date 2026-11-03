# Write a client

> **Status:** Implemented. The harness is the reference client on the devnet.

A client does three things: resolve a component from the r-share Root, sign and submit a
deploy, and read the result. This repository deliberately does not ship an SDK; the harness
shows the devnet path.

## Signing and submitting

The devnet harness submits through the node's own CLI inside the devnet container:

```text
rnode --grpc-host localhost deploy \
  --phlo-limit <limit> --phlo-price <price> \
  --private-key <secret> --shard-id /root \
  --valid-after-block-number <tip> <term-file>
```

The node prints `DeployId is: <hex>`. Poll `GET /api/v1/deploy-status/{id}` until the status
is `ProcessedWithSuccess` or `ProcessedWithError`. For production clients, sign the
`DeployDataProto` yourself and post to `/api/v1/deploy` (HTTP) or call the gRPC
`DeployService.doDeploy`. The exact wire shape is in the node's documentation.

Never put a private key in a term. The key signs the deploy; the contract sees only the
`deployerId`.

## Resolving a component

The r-share Root is a directory of `component:<name>` entries. A client that has the Root URI
can resolve any component:

```rholang
new rl(`rho:registry:lookup`), rootCh, ret, deployId(`rho:rchain:deployId`) in {
  rl!(`<ROOT_URI>`, *rootCh) |
  for (Root <- rootCh) {
    Root!("resolve", "component:listing-factory", 1, *ret) |
    for (@r <- ret) { deployId!(r) }
  }
}
```

`resolve` returns `(true, uri, version)` or `(false, "unknown component" | "retired" |
"stale")`. Then look up the URI and call the arm.

## Calling an arm

Every arm is a send whose first field is the arm name. Data arguments use plain values; names
(payments, purses, facets, tokens) are sent with `*`:

```rholang
new rl(`rho:registry:lookup`), ch, ret, deployId(`rho:rchain:deployId`) in {
  rl!(`<COMPONENT_URI>`, *ch) |
  for (C <- ch) {
    C!("isActive", *ret) |
    for (@r <- ret) { deployId!(r) }
  }
}
```

Replies are `(true, value)` or `(false, reason)`. Read them exhaustively; an unmatched `for`
is silence, not an error.

## Reading without a deploy

`POST /api/v1/explore-deploy` runs a term read-only. It has no block context, so anything
that reads `rho:block:data` must be a signed deploy. Use explore for static reads and a
signed deploy for `Clock`, `probe` at a height, or anything that depends on the current
block.

For state at a pinned block, use `data-at-name-by-block-hash` or
`explore-deploy-by-block-hash`.

## The read model a client needs

- **Provider page:** `ProviderContract("info")`, `listings`.
- **Listing page:** `ListingContract("info")`, `units`, `unit`, `isActive`.
- **Calendar:** `UnitContract("schedule", slotIds)` and `probe(slotIds)`.
- **Booking page:** `BookingContract("info")`, `status`, `ticket`, `ops`.
- **Ops:** `GuestLedger("ops")` for the guest's op results.

There is no indexer in this repo. A production client either walks the registry or runs its
own indexer over deploy events.

## Idempotency

Use a unique `opId` per user intent. A replay with the same `opId` returns the stored result
and moves no money. A new intent gets a new `opId`, even if it targets the same booking.

## OCapN

The node exposes OCapN for non-Rholang clients. An OCapN call becomes a signed deploy using
the node's configured deployer key. It is a transport, not a different authority model; the
contract still checks the deployer.

## Error handling

- `(false, reason)` is a designed refusal. Show the reason and let the user choose.
- A deploy that errors is a rollback; nothing moved.
- A deploy that never processes is a liveness problem; retry with a new deploy and the same
  `opId`.
- Never retry a mutating deploy with a new `opId` unless you intend a second operation.

> **Where to go deeper.** [Bookings and lifecycle](../concepts/bookings-and-lifecycle.md) is
> the state machine. [Arm reference](../reference/arms.md) is the signature list. [Platform
> constraints](../reference/platform-constraints.md) covers deploy limits.
