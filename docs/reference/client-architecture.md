# Client architecture

> **Status:** Implemented. Rust workspace plus a React web app. The client reuses r-cash's
> signing and node primitives; it does not reimplement them in TypeScript.

## Layers

```
web (React + Vite + TypeScript)
  └── rshare-wasm (key, keystore, JSON boundary)
        └── rshare-sdk (typed reads/writes, Root resolution, receipts)
              ├── rshare-core (terms, slots, statuses, rendering)
              └── rcash-sdk::Client ── rcash-chain ── rcash-core
        rshare-keystore (Web3 v3)
rshare-cli ── rshare-sdk + rcash-chain::keystore::DevnetKeys
```

## Crates

| Crate | Owns |
|---|---|
| `rshare-core` | manifest, component keys, 20-field terms, slot shapes, statuses, op ids, JSON→Rholang rendering, 56 KiB budget |
| `rshare-sdk` | connected client, Root resolve/resolveMany cache, typed reads, writes, lifecycle actions, admin payload/sign, account funding, previews, refusals |
| `rshare-keystore` | Web3 Secret Storage v3 encrypt/decrypt (scrypt n=2^14, AES-128-CTR, Keccak MAC) |
| `rshare-cli` | `rshare-cli` reference client: reads, writes, admin, devnet fixtures/journey, keystore export |
| `rshare-wasm` | wasm-bindgen facade; the key never crosses into JS |
| `web` | React app, routes, IndexedDB journal, multisig console |

## Why no `rshare-chain`

`rcash-chain` already owns deploy encoding, secp256k1 signing, REV address derivation, the
node HTTP client, deploy-status polling, finality checks and `RhoValue` decoding. r-share
pins it by commit and reuses it. A second chain crate would duplicate consensus-facing code.

## r-cash boundary

`rshare-sdk` uses `rcash_sdk::Client` for `deploy_term`, `node().explore`, `height`,
`address` and key plumbing. It does **not** attach an r-cash manifest and does not call
r-cash's typed FX, oracle, pool or v4 operations. Everything r-cash-related goes through
r-share's `component:payments` adapter (`ensureAccount`, `accountOf`, `depositFacet`,
`issuer`, `health`).

## Network and phlo

- Default shard `/root`, HTTP `http://localhost:40403`.
- Discovery is disabled; r-share uses one node. There is no hostlist UI.
- Default phlo limit `4_000_000`, price `1`. A book deploy resolves several components.
- Deploy success means finalized, processed successfully, and a `(true, …)` reply.

## Key handling

- The browser creates/decrypts the key inside WASM.
- Only an encrypted v3 keystore is stored in IndexedDB.
- `lock` drops the key and rebuilds a reader client so public reads stay available.
- The CLI uses the plaintext `.devnet/keys.json` format, documented as devnet tooling.

## Compatibility

The Rust client pins r-cash `49b441d`, which pins rholo `841cb84`. The r-share devnet is
built from rholo `e2d073d`. Deploy encoding and the wire API are stable across those
revisions; `doctor` records the node version and the campaign records the pair. This is a
tested compatibility claim, not a formal refinement.

## Tests

- Rust unit and fixture-transport tests: `make client-test`.
- WASM boundary and keystore cross-decrypt: `npm --prefix web run test:wasm`.
- Web unit tests: `npm --prefix web test`.
- Browser journeys: `npm --prefix web run test:e2e` against a fresh devnet.
- CLI journey: `make client-journey` (positive and negative assertions).

> **Where to go deeper.** [Client reads](client-reads.md) is the explore/signed split.
> [Discovery and share links](discovery-and-share-links.md) is the read model.
> [Admin multisig](admin-multisig.md) is the signature ceremony.
