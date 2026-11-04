# Use the web app

> **Status:** Implemented. A React reference client over the WASM SDK; every read and write
> goes to a live node. The recorded devnet campaign runs the guest, provider and admin
> browser journeys.

## Run it

```sh
make devnet-up rcash-keys rcash-bootstrap keys install
make client-fixtures              # provider, listing, unit, catalogue
cd web && npm install
npm run dev                       # builds WASM, syncs the manifest, serves :5173
```

`npm run dev` runs `scripts/build-wasm.mjs` and `scripts/sync-manifest.mjs` first. The
manifest and catalogue are generated files: `deployments/devnet.json` and
`deployments/catalogue.json` are gitignored and copied into `web/public/`.

If the node is not at `http://localhost:40403`, set the URL in **Settings** or set
`VITE_RSHARE_NODE` before `npm run dev`.

## Routes

| Route | What it does |
|---|---|
| `/` | Curated catalogue, paste-URI box, node height |
| `/listing/:uri` | Terms, units, schedule, free `probe`, signed `quote`, book |
| `/booking/:uri` | Bond, ticket, lifecycle actions, share link |
| `/provider` or `/provider/:uri` | Register a provider, create listings, add units, activate/pause/delist |
| `/me` | Native/account REV, faucet, funding, guest ledger and bookings |
| `/admin` | Global, listing, provider and booking admin actions with multi-profile signing |
| `/history` | This device's activity journal (not a chain index) |
| `/settings` | Node, manifest, profiles, export |

Disabled screens state why they are disabled: there is no indexer, provider inbox,
notification stream, fiat ramp or evidence store in this client.

## Wallet and keys

- The key is generated and decrypted inside WASM. JavaScript receives only the encrypted
  Web3 v3 keystore, public identifiers and results.
- Keystores live in IndexedDB (`rshare` → `keystores`); the password is never stored.
- Multiple saved profiles are supported, one unlocked at a time. The admin multisig page
  uses this to collect signatures.
- The activity journal and pending admin actions are also in IndexedDB. They are a device
  log, not authoritative state.

## What to expect

- A **quote** is a signed read and costs phlo. It is indicative at a finalized block; the
  book deploy revalidates atomically.
- A **probe** is a free read. A held slot past its `untilHeight` is shown as reclaimable.
- A booking is successful only when its block is finalized. The UI does not call a processed
  deploy "done".
- A refusal such as `slot held`, `terms` or `below threshold` is a designed answer; the app
  shows it and leaves the choice to you.

## Honest limits

There is no provider booking inbox and no global search. A guest shares the booking link;
a provider opens it. The catalogue is curated, not discovered on-chain. Funding uses the
devnet faucet and the ERTP `revFund` bridge; it is not a fiat rail.

> **Where to go deeper.** [Run a provider](run-a-provider.md) is the listing workflow.
> [Fund a devnet account](fund-a-devnet-account.md) is the balance path.
> [Client architecture](../reference/client-architecture.md) is the layer map.
