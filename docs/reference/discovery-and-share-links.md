# Discovery and share links

> **Status:** Implemented as a curated manifest/catalogue plus share links. There is no
> indexer and no on-chain provider or booking directory.

## What exists

- The deployment manifest supplies the Root URI. Every component URI is resolved through
  the Root, not trusted from the file.
- `deployments/catalogue.json` is a generated, curated list of provider and listing URIs.
  `make client-fixtures` writes one; a fork can replace it.
- Share links:
  - `#/listing/<rho:id:…>`
  - `#/provider/<rho:id:…>`
  - `#/booking/<rho:id:…>`
- The Explore page accepts a pasted listing or provider URI.

## What does not exist

| Missing | Consequence |
|---|---|
| Provider directory | the catalogue is curated; there is no global provider list |
| Booking inbox | a provider needs the guest's booking link |
| Aggregate calendar | the client composes unit schedules and probes |
| Search and notifications | disabled screens state the reason |
| Chain index | the guest ledger is authoritative for a guest's bookings |

## Why

A provider directory or booking inbox would be a new shared mutable cell or a new
capability flow. Both are contract-level design changes with their own proofs and tests,
not a client feature. This client does not fake either.

The guest ledger enumerates a guest's own operations and booking URIs. The provider index
enumerates a provider's listings. Neither enumerates bookings for a provider.

## Operating a fork

1. Install the contracts and run the fixtures.
2. Curate `deployments/catalogue.json` (labels and URIs). Verify every entry live before
   publishing it.
3. Run your own indexer if you need search, inboxes or notifications; keep the chain as the
   authority and never serve a cached balance as truth.

> **Where to go deeper.** [Use the web app](../guides/use-the-web-app.md) is the UI path.
> [Client architecture](client-architecture.md) is the layer map. [Platform
> constraints](platform-constraints.md) covers deploy limits.
