# Client reads

> **Status:** Implemented. Every read is either a free `explore-deploy` or a signed,
> phlo-charged deploy. The distinction is deliberate and visible in the UI.

## The split

| Read | Transport | Why |
|---|---|---|
| Root `resolve` / `resolveMany`, `keys` | explore | stateless directory |
| `Terms` fields/validate/hash/projections | explore | no block context needed |
| `Codec("adminPayload")` | explore | stateless canonicalisation |
| provider/listing/booking/bond read arms | explore | state peeks, no clock or `deployerId` |
| `ListingContract("probe")` | explore | slot records only |
| `ShareAdmin` keys/paused, `Payments` health | explore | state peeks |
| native REV balance | explore | `revVault("getBalance")` |
| `GuestBook("getOrCreate")`, `GuestLedger("ops")` | signed | take `dId` |
| account balance | signed | takes `dId` |
| `ListingContract("quote")` | signed | calls `Clock` |
| every write | signed | state change |

`explore-deploy` has no block context: `rho:block:data` reports height 0 there. A quote run
through explore would see a false window. The signer is also absent in explore, so any arm
that checks the deployer must be signed.

## Metadata

Every SDK read returns `Read<T>` with the value, the block height and hash, the source, and
the deploy id when signed. The UI shows `free read · block H` or `signed read · block H`.

## Probe and quote

`probe` is free and does not apply the current height to a held record. The UI compares
`untilHeight` to the finalized height and labels an expired hold reclaimable. `quote` is
signed and returns the authoritative rent, check-in/out and window summary.

The quote is indicative. `book` revalidates the same request atomically in the listing
allocator; a slot that changed since the quote refuses `slot held`.

## Finality

A processed deploy is not success. The SDK requires a finalized block, a successful status,
and a `(true, …)` reply. A `(false, reason)` is a designed refusal. A deploy error is a
rollback; phlo is still spent. An unfinalized outcome is `unknown`, and a value-moving
deploy is never retried blindly with a new op id.

> **Where to go deeper.** [Client architecture](client-architecture.md) is the layer map.
> [Arm reference](arms.md) is the signature list. [Concurrency
> doctrine](../concepts/concurrency-doctrine.md) is the allocator rule.
