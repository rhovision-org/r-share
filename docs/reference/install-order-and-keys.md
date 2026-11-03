# Install order and component keys

> **Status:** Implemented. This chapter is checked against `harness/install.py`,
> `contracts/src/deploy/*.rho` and `configs/rcash.json`.

The installer deploys the terms in dependency order, captures the r-share Root URI after
`00_root.rho`, and renders that URI into every later term. It also ensures the platform
account after `40_payments.rho`.

## Order

| # | Deploy file | Registers |
|---|---|---|
| 1 | `00_root.rho` | `component:root` |
| 2 | `01_kernel.rho` | `component:clock`, `component:verifier`, `component:codec` |
| 3 | `02_admin.rho` | `component:admin-guard`, `component:admin` |
| 4 | `40_payments.rho` | `component:payments` |
| 5 | `12_policy.rho` | `component:policy` |
| 6 | `10_provider.rho` | `component:provider-book` |
| 7 | `13_unit.rho` | `component:unit-factory` |
| 8 | `11_listing.rho` | `component:listing-factory` |
| 9 | `14_listing_contract.rho` | `component:listing-contract-factory` |
| 10 | `20_booking.rho` | `component:guest-book` |
| 11 | `23_guest_ledger.rho` | `component:guest-ledger-factory` |
| 12 | `21_booking.rho` | `component:booking-factory` |
| 13 | `22_booking_contract.rho` | `component:booking-contract-factory` |

The numeric filenames are not the install order. `40_payments` and `12_policy` are deployed
before the provider/listing/unit group; `13_unit` and `11_listing` are deployed before
`14_listing_contract`; the booking and guest-ledger group comes last.

## Placeholders

The installer renders these before deploying:

| Placeholder | Value |
|---|---|
| `__ROOT_URI__` | the r-share Root URI, captured after step 1 |
| `__RCASH_ROOT_URI__` | the r-cash Root URI from `deployments/devnet-v4.json` |
| `__PLATFORM_OWNER__` | the deployer's REV address |
| `__ADMINS__` | three public keys as `"hex".hexToBytes()` |
| `__THRESHOLD__` | `2` |

`harness/install.py --dry-run` renders with placeholder values and writes
`.devnet/rendered/`; no chain is needed.

## r-share component keys

The Root registers these keys. Clients resolve them with `ShareRoot("resolve", key,
minVersion, ret)`.

```
component:root
component:clock
component:verifier
component:codec
component:admin-guard
component:admin
component:payments
component:policy
component:provider-book
component:listing-factory
component:unit-factory
component:listing-contract-factory
component:guest-book
component:guest-ledger-factory
component:booking-factory
component:booking-contract-factory
```

## r-cash component keys

The Payments adapter resolves these through the **r-cash** Root, not the r-share Root:

```
component:account-factory
component:asset-resolver
component:clock
```

`configs/rcash.json` lists them as `required_components`. The adapter pins r-cash v0.5 at
`49b441d` and version-gates every resolve.

## Post-install checks

- `ShareRoot("keys")` lists every r-share key above.
- `ShareRoot("resolve", "component:clock", 1)` returns `(true, uri, 1)`.
- `Payments("health")` returns all three r-cash versions.
- `Payments("platformOwner")` returns the deployer's REV address.
- The platform account exists (the installer ensures it).
- `deployments/devnet.json` has the Root URI and the component results.

## Freeze

`ShareRoot("freeze", nonce, sigs)` blocks new registrations. Rotation and deprecation remain
available. A fork that expects to add components later should delay the freeze or use
`rotate`.

> **Where to go deeper.** [Contract map](contract-map.md) is the component list. [Arm
> reference](arms.md) is the signatures. [Platform constraints](platform-constraints.md) is
> the deploy limit list.
