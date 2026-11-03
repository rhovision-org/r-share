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
| 5 | `03_terms.rho` | `component:terms` |
| 6 | `04_remedy.rho` | `component:remedy` |
| 7 | `10_provider.rho` | `component:provider-book` |
| 8 | `13_unit.rho` | `component:unit-factory` |
| 9 | `14_listing_contract.rho` | `component:listing-factory` |
| 10 | `15_bond.rho` | `component:bond-factory` |
| 11 | `22_booking_contract.rho` | `component:booking-factory` |
| 12 | `23_guest_ledger.rho` | `component:guest-ledger-factory` |
| 13 | `20_booking.rho` | `component:guest-book` |

`20_booking` comes last because GuestBook resolves the guest-ledger factory at runtime;
`22_booking_contract` comes before the ledger because GuestLedger resolves the booking
factory. The old `21_booking` pass-through is gone.

## Placeholders

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

```
component:root
component:clock
component:verifier
component:codec
component:admin-guard
component:admin
component:payments
component:terms
component:remedy
component:provider-book
component:unit-factory
component:listing-factory
component:bond-factory
component:booking-factory
component:guest-ledger-factory
component:guest-book
```

`component:listing-contract-factory` and `component:booking-contract-factory` are gone; the
factories are internal to `14` and `22`, and `22` registers as `component:booking-factory`.

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
- `ShareRoot("resolveMany", ["component:clock", "component:terms", "component:remedy"], 1)`
  returns the three URIs in one call.
- `Terms("fields")` returns the ordered 20-field list.
- `Remedy("fee", 1000, 500)` returns `(true, 50, 950)`.
- `Payments("health")` returns all three r-cash versions.
- `Payments("platformOwner")` returns the deployer's REV address.
- `Bond("info")` reports `paid: false` before settlement and `paid: true` after.

## Freeze

`ShareRoot("freeze", nonce, sigs)` blocks new registrations. Rotation and deprecation remain
available. A fork that expects to add components later should delay the freeze or use
`rotate`.

> **Where to go deeper.** [Contract map](contract-map.md) is the component list. [Arm
> reference](arms.md) is the signatures. [Platform constraints](platform-constraints.md) is
> the deploy limit list.
