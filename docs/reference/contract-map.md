# Contract map

> **Status:** Implemented. This chapter is checked against `contracts/components.json` and the
> deploy directory by `tools/check_docs.py`.

Every component, its deploy file, its state cell, and whether it is a singleton or spawned
per object. The arm signatures are in the [arm reference](arms.md); the install order is in
[Install order and component keys](install-order-and-keys.md).

## Deploy files

| Deploy file | Components |
|---|---|
| `00_root.rho` | ShareRoot |
| `01_kernel.rho` | Clock, Verifier, Codec |
| `02_admin.rho` | AdminGuard, ShareAdmin |
| `12_policy.rho` | Policy |
| `40_payments.rho` | Payments |
| `10_provider.rho` | ProviderBook, ProviderContract |
| `13_unit.rho` | UnitFactory, UnitContract, SlotContract |
| `11_listing.rho` | ListingFactory |
| `14_listing_contract.rho` | ListingContractFactory, ListingContract |
| `20_booking.rho` | GuestBook |
| `23_guest_ledger.rho` | GuestLedgerFactory, GuestLedger |
| `21_booking.rho` | BookingFactory |
| `22_booking_contract.rho` | BookingContractFactory, BookingContract |

## Components

| Component | File | State cell | Shape |
|---|---|---|---|
| ShareRoot | `00_root.rho` | `rootState` | singleton directory |
| Clock | `01_kernel.rho` | none | stateless |
| Verifier | `01_kernel.rho` | none | stateless |
| Codec | `01_kernel.rho` | none | stateless |
| AdminGuard | `02_admin.rho` | none | stateless |
| ShareAdmin | `02_admin.rho` | `adminState` | singleton admin set |
| Policy | `12_policy.rho` | none | stateless |
| Payments | `40_payments.rho` | `paymentsState` | singleton r-cash adapter |
| ProviderBook | `10_provider.rho` | `providerBookState` | singleton index |
| ProviderContract | `10_provider.rho` | `providerState` | one per provider |
| ListingFactory | `11_listing.rho` | none | stateless |
| UnitFactory | `13_unit.rho` | none | stateless |
| ListingContractFactory | `14_listing_contract.rho` | none | stateless |
| ListingContract | `14_listing_contract.rho` | `listingState` | one per listing |
| UnitContract | `13_unit.rho` | `unitState` | one per unit |
| SlotContract | `13_unit.rho` | `slotState` | one per slot |
| GuestBook | `20_booking.rho` | `guestBookState` | singleton index |
| GuestLedgerFactory | `23_guest_ledger.rho` | none | stateless |
| GuestLedger | `23_guest_ledger.rho` | `ledgerState` | one per guest |
| BookingFactory | `21_booking.rho` | none | stateless |
| BookingContractFactory | `22_booking_contract.rho` | none | stateless |
| BookingContract | `22_booking_contract.rho` | `bookingState` | one per booking |

## Dependencies

```
ShareRoot
  |
  +-- Clock / Verifier / Codec
  +-- AdminGuard / ShareAdmin
  +-- Policy
  +-- Payments ---- r-cash Root (account-factory, asset-resolver, clock)
  +-- ProviderBook ---- ProviderContract
  +-- ListingFactory ---- ListingContractFactory ---- ListingContract
  |                                             \---- UnitFactory
  +-- UnitFactory ---- UnitContract ---- SlotContract
  +-- GuestBook ---- GuestLedgerFactory ---- GuestLedger
  +-- BookingFactory ---- BookingContractFactory ---- BookingContract
```

Figure: component dependencies. Factories resolve their instance factory from the Root at
arm entry; no registry capability is cached in a bundle.

## Authority

- Installer-gated: ShareRoot `register` before `freeze`.
- Owner-gated: provider, listing, unit and booking owner arms derive the owner from the
  deployer.
- 2-of-3 gated: Root rotate/deprecate/freeze/admin, ShareAdmin, listing/unit/booking
  adminPause, booking arbitrate, Payments admin.
- Permissionless: booking `settle`, `expire`, `resolveDeposit`, `closeEvent`; reads.

## Where the code lives

- `contracts/src/deploy/` — the 13 terms above.
- `contracts/src/lib/` — `Auth`, `Guard`, `Terms`, `Money`, inlined at build time.
- `contracts/components.json` — the machine-readable catalogue.
- `contracts/measure.json` — per-term size, depth and contract count.

> **Where to go deeper.** [Arm reference](arms.md) is the signatures. [Install order and
> component keys](install-order-and-keys.md) is the deployment order. [Data schemas](data-schemas.md)
> is the state shapes.
