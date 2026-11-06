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
| `03_terms.rho` | Terms |
| `04_remedy.rho` | Remedy |
| `30_cover_terms.rho` | CoverTerms |
| `31_cover_offer.rho` | CoverBook |
| `33_cover_policy.rho` | CoverPolicyFactory |
| `32_cover_pool.rho` | CoverPool |
| `40_payments.rho` | Payments |
| `10_provider.rho` | ProviderBook, ProviderContract |
| `13_unit.rho` | UnitFactory (stateless schedule helpers) |
| `14_listing_contract.rho` | ListingFactory, ListingContract (per-listing allocator; plus the internal ListingContractFactory) |
| `15_bond.rho` | BondFactory, Bond |
| `20_booking.rho` | GuestBook |
| `22_booking_contract.rho` | BookingContractFactory |
| `34_booking_instance.rho` | BookingInstanceFactory, BookingContract |
| `23_guest_ledger.rho` | GuestLedgerFactory, GuestLedger |
| `24_guest_ledger_cover.rho` | GuestLedgerCover |

## Components

| Component | File | State cell | Shape |
|---|---|---|---|
| ShareRoot | `00_root.rho` | `rootState` | singleton directory |
| Clock | `01_kernel.rho` | none | stateless |
| Verifier | `01_kernel.rho` | none | stateless |
| Codec | `01_kernel.rho` | none | stateless |
| AdminGuard | `02_admin.rho` | none | stateless |
| ShareAdmin | `02_admin.rho` | `adminState` | singleton admin set |
| Terms | `03_terms.rho` | none | stateless schema/validation/hash |
| Remedy | `04_remedy.rho` | none | stateless arithmetic |
| CoverTerms | `30_cover_terms.rho` | none | stateless offer validation/hash |
| CoverBook | `31_cover_offer.rho` | `coverBookState` | singleton offer index |
| CoverPolicyFactory | `33_cover_policy.rho` | — | per-policy purse, release token and claim token |
| CoverPool | `32_cover_pool.rho` | `coverPoolState` | operator-controlled facility purse |
| Payments | `40_payments.rho` | `paymentsState` | singleton r-cash adapter |
| ProviderBook | `10_provider.rho` | `providerBookState` | singleton index |
| ProviderContract | `10_provider.rho` | `providerState` | one per provider |
| ListingFactory | `14_listing_contract.rho` | none | stateless |
| ListingContract | `14_listing_contract.rho` | `listingState` | one per listing; owns units and slots |
| UnitFactory | `13_unit.rho` | none | stateless |
| BondFactory | `15_bond.rho` | none | stateless |
| Bond | `15_bond.rho` | `bondState` | one per booking |
| GuestBook | `20_booking.rho` | `guestBookState` | singleton index |
| GuestLedgerFactory | `23_guest_ledger.rho` | none | stateless |
| GuestLedger | `23_guest_ledger.rho` | `ledgerState` | one per guest |
| BookingContractFactory | `22_booking_contract.rho` | none | stateless |
| BookingContract | `34_booking_instance.rho` | `bookingState`, `bookingCoverState` |
| BookingInstanceFactory | `34_booking_instance.rho` | — | per-booking instance factory |
| GuestLedgerCover | `24_guest_ledger_cover.rho` | — | atomic covered booking flow | one per booking |

## Dependencies

```
ShareRoot
  |
  +-- Clock / Verifier / Codec / AdminGuard / ShareAdmin
  +-- Terms ---- Codec
  +-- Remedy
  +-- CoverTerms ---- Codec
  +-- CoverBook ---- CoverTerms / Payments
  +-- CoverPool ---- Payments / Clock / CoverBook / CoverPolicyFactory
  +-- CoverPolicyFactory ---- Payments / Clock
  +-- Payments ---- r-cash Root (account-factory, asset-resolver, clock)
  +-- ProviderBook ---- ProviderContract
  +-- ListingFactory ---- ListingContractFactory ---- ListingContract
  |                                             \---- UnitFactory (stateless)
  +-- ListingContract ---- UnitFactory (stateless) / Clock / AdminGuard / ShareAdmin / ProviderContract
  |                    \--- owns units and slots in one allocState cell
  +-- BondFactory ---- Bond ---- Payments
  +-- GuestBook ---- GuestLedgerFactory ---- GuestLedger
  +-- BookingContractFactory ---- BookingContract ---- ListingContract (verifyHold)
  +-- GuestLedger / BookingContract ---- ListingContract (acquire/confirm/release)
```

Figure: component dependencies. Factories resolve their instance factory from the Root at
arm entry; no registry capability is cached in a bundle.

## Authority

- Installer-gated: ShareRoot `register` before `freeze`.
- Owner-gated: provider, listing, unit and booking owner arms derive the owner from the
  deployer with the inlined `Auth` fragment.
- 2-of-3 gated through the deployed `AdminGuard`: Root rotate/deprecate/freeze/admin,
  ShareAdmin, listing/booking adminPause, booking arbitrate, Payments admin.
- Permissionless: booking `settle`, `expire`, `resolveDeposit`, `closeEvent`; reads.

## Where the code lives

- `contracts/src/deploy/` — the 19 terms above (14 safe, 5 experimental cover).
- `contracts/src/lib/` — `Auth` (inlined into owner-gated terms), `AdminGuard` (the
  shared wrapper inlined into the four terms that verify admin signatures), `AdminsText`
  (canonicalises an admin list because the node has no list `toString`), `AdminSet`
  (distinct-key/threshold validation inlined into Root, ShareAdmin, provider, listing and
  booking), `Resolve` (the generic component resolver behind each term's resolve helper),
  `Guard` (only the bootstrap Root), `Money` (inlined into Remedy). The source metric counts
  one `Resolve` copy; the inlined helper still performs one Root lookup per call at runtime.
- `contracts/components.json` — the machine-readable catalogue.
- `contracts/measure.json` — per-term size, depth and contract count.

> **Where to go deeper.** [Arm reference](arms.md) is the signatures. [Install order and
> component keys](install-order-and-keys.md) is the deployment order. [Data schemas](data-schemas.md)
> is the state shapes.
