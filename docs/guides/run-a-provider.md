# Run a provider

> **Status:** Implemented in the CLI and web client; exercised by the provider browser
> journey and the devnet journey.

A provider owns one provider contract and any number of listings. Each listing owns units
and slots in one allocator cell.

## Register

```sh
rshare-cli --key business provider register --profile '{"name":"Harbour Lets"}'
```

The reply is `(providerUri, ownerId)`. `ownerId` is derived from the deployer key, never
supplied. In the web app, open **Provider**, fill the profile JSON and click **Register
provider**.

The profile and unit `attrs` are public on-chain. Do not put personal data or secrets there.

## Create a listing

```sh
rshare-cli --key business listing create \
  --provider <providerUri> --id harbour-flat --kind lodging --terms @terms.json
```

Terms are immutable for the listing. To change price or policy, create a new listing.
`Terms("validate")` on-chain is authoritative; the client's local check is advisory.

## Add units and slots

```sh
rshare-cli --key business unit ensure \
  --listing <listingUri> --unit unit:1 --price 100000000 \
  --attrs '{"name":"Flat 1"}' \
  --slots '[{"slotId":"s1","startHeight":3000,"endHeight":3300}]'
```

`--price` is base units (1 REV = 100000000). Slot windows are block heights; the start must
be in the future and within the terms' lead and duration bounds.

Activate only after at least one unit exists:

```sh
rshare-cli --key business listing activate --uri <listingUri>
```

The provider's listing index keeps a copy of the entry; the listing's own state is the
authority for `draft`/`active`/`paused`/`delisted`. The web dashboard reads the listing
state, not the copied index.

## Manage bookings

The guest shares a booking link (`#/booking/<uri>`). Paste it into **Provider** or open it.
The provider can accept or reject a requested booking, check in/out, cancel, claim a no-show
or claim the deposit. Every action is owner-gated by the booking contract.

There is no on-chain provider inbox. Do not expect the listing or provider state to
enumerate bookings.

## Pause and delist

- provider pause blocks new bookings on all its listings;
- listing pause blocks acquisition on that listing;
- unit pause blocks that unit;
- delist is terminal for new business.

Existing holds and lifecycle actions continue to work.

> **Where to go deeper.** [Listings, units and slots](../concepts/listings-units-slots.md)
> is the data model. [Arm reference](../reference/arms.md) is the signature list.
> [Use the web app](use-the-web-app.md) is the UI path.
