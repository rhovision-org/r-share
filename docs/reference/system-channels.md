# System channels

> **Status:** Implemented. These are the node channels r-share uses.

r-share uses a small set of native channels. It does not mint, custody or clear assets; the
asset channels are reached through r-cash.

## Registry

| Channel | Use |
|---|---|
| `rho:registry:lookup` | resolve a URI to a contract channel |
| `rho:registry:insertArbitrary` | publish a `bundle+{*Contract}` and get a URI |

A URI is data; the capability is the channel behind it. A registry-resolved capability is
never captured in a bundle. The Root is the directory of component URIs.

## Identity and deploys

| Channel | Use |
|---|---|
| `rho:rchain:deployerId` | the current deploy's deployer identity |
| `rho:rchain:deployerId:ops` | `pubKeyBytes` for the deployer's public key |
| `rho:rchain:deployId` | the result channel a signed deploy emits |
| `rho:rev:address` | derive a REV address from a public key |

`authOwner` calls `deployerIdOps("pubKeyBytes", *dId)` and `revAddress("fromPublicKey", pk)`.
No owner address is accepted as a caller argument.

## Time

| Channel | Use |
|---|---|
| `rho:block:data` | `(blockNumber, senderBytes, timestampMs)` |

Only `blockNumber` is used for enforcement. The timestamp is the proposer's unverified wall
clock and is never compared.

## Cryptography

| Channel | Use |
|---|---|
| `rho:crypto:blake2b256Hash` | terms hashes, admin payload digests |
| `rho:crypto:secp256k1Verify` | 2-of-3 admin and arbiter signatures |

The Guard payload is `rshare:admin:v1|<domain>|<nonce>|<action>|<args>`; the digest is
BLAKE2b-256 of its UTF-8 bytes; signatures are DER secp256k1 over the digest.

## Payments through r-cash

r-share does not call the native vault directly. The Payments adapter resolves the r-cash
Root and uses:

| r-cash component | Use |
|---|---|
| `component:account-factory` | `getOrCreate`, `accountOf` |
| `component:asset-resolver` | `issuer(assetId)` → brand + issuer |
| `component:clock` | r-cash's height source |

The guest's account `withdraw` is called inside the guest's deploy. Settlement uses account
`depositFacet` and ERTP purse `withdraw`/`deposit`. Payments are consumed exactly once.

## What r-share does not use

- `rho:rchain:revVault` directly (r-cash accounts are the rail);
- `rho:rchain:pos` (validator staking is not a product concern);
- `rho:io:stdout` (no console output in contracts);
- `rho:rchain:deployerId:ops` beyond `pubKeyBytes`.

## Reply conventions

- System channels return their documented tuple; the adapter matches it exhaustively.
- A missing match is silence, not an error. Every reviewed arm matches the shape it expects.
- `explore-deploy` has no block context; anything reading `rho:block:data` must be a signed
  deploy.

> **Where to go deeper.** [Escrow and settlement](../concepts/escrow-and-settlement.md) is
> the money path. [Platform constraints](platform-constraints.md) is the limit list. [Arm
> reference](arms.md) is the signatures.
