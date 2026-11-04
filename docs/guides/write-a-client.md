# Write a client

> **Status:** Implemented. r-share ships a Rust SDK, a CLI, a WASM facade and a React
> reference app. This chapter is the contract they share.

A client resolves a component from the r-share Root, signs and submits a deploy, and reads
the result. The repository's own client is the reference; a fork can reuse it or port the
SDK surface.

## The three-layer client

| Layer | Crate | What it does |
|---|---|---|
| Pure | `rshare-core` | terms, slots, statuses, op ids, Rholang rendering, budgets |
| Chain-aware | `rshare-sdk` | Root resolution, reads/writes, lifecycle, admin, funding |
| Surfaces | `rshare-cli`, `rshare-wasm`, `web` | scripts, browser wallet, UI |

`rshare-sdk` reuses r-cash's `rcash-sdk`/`rcash-chain` for signing, deploy encoding, node
HTTP, finality polling and `RhoValue` decoding. Do not write a second signer in TypeScript.

## Resolve

```rholang
new return, rl(`rho:registry:lookup`), rootCh, ret in {
  rl!(`<ROOT_URI>`, *rootCh) |
  for (Root <- rootCh) {
    Root!("resolve", "component:listing-factory", 1, *ret) |
    for (@r <- ret) { return!(r) }
  }
}
```

`resolve` returns `(true, uri, version)` or `(false, "unknown component" | "retired" |
"stale")`. `resolveMany` returns a map of `{uri, version}`. Explore-deploy captures the
first `new`-bound private name; a signed deploy replies on `rho:rchain:deployId`.

## Read vs write

- Use `explore-deploy` for height-independent peeks: provider, listing, booking, Bond,
  `probe`, `Terms`, `ShareAdmin`, `Payments`.
- Use a signed deploy for `GuestBook("getOrCreate")`, `GuestLedger("ops")`, account balance,
  `quote`, and every write. `quote` calls `Clock`; the guest arms take a `deployerId`.

Every SDK read returns the block height, hash and source, so a UI can say what it read and
when.

## Call an arm

```rholang
new return, rl(`rho:registry:lookup`), ch, ret in {
  rl!(`<COMPONENT_URI>`, *ch) |
  for (C <- ch) {
    C!("isActive", *ret) |
    for (@r <- ret) { return!(r) }
  }
}
```

Data arguments use plain values; names (payments, purses, facets, tokens) use `*`. Replies
are `(true, value)` or `(false, reason)`; a few stateless arms (`Terms("fields")`,
`Terms("booking")`, `Terms("remedy")`) reply with a bare value.

## Booking

```ts
const quote = await client.listing_quote(listingUri, unitSlotsJson);
const opId = client.new_op_id("book", BigInt(Date.now()));
const receipt = await client.book(listingUri, unitSlotsJson, opId, listing.termsHash);
```

Use a unique op id per user intent. `bookingId = guestId:opId`. A replay returns the stored
booking and moves no money. A new intent gets a new op id.

## Sign and submit

The SDK signs `DeployData` with secp256k1, waits for the deploy to be processed and its block
to be final, and decodes the reply. Success requires all three:

1. the deploy is finalized;
2. the status is `processedWithSuccess`;
3. the reply is `(true, …)`.

A `(false, reason)` is a refusal. A deploy error is a rollback. An unfinalized outcome is
unknown; do not resubmit a value-moving deploy blindly.

## Wallets

The WASM facade keeps the key in Rust. The browser stores an encrypted Web3 v3 keystore in
IndexedDB. The CLI uses `.devnet/keys.json`, which is plaintext devnet tooling. Never put a
private key in a term; the key signs the deploy and the contract sees only the deployer id.

## Terms and sizes

The 20-field terms order is authoritative in `Terms("fields")`. Local validation is
advisory; `Terms("validate")` decides. Read the listing's `termsHash` and pass it to `book`;
the SDK renders it as bytes, not as a string.

The node cap is 64 KiB; the client budget is 56 KiB. The SDK checks before signing. Preview
terms when a fork adds large attrs or schedules.

## What the client does not do

No indexer, search, provider inbox or notifications. The catalogue is curated and share
links are first-class. Discovery is an operator concern.

> **Where to go deeper.** [Client architecture](../reference/client-architecture.md) is
> the layer map. [Client reads](../reference/client-reads.md) is the read split. [Admin
> multisig](../reference/admin-multisig.md) is the signature ceremony.
