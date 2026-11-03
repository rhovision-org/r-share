# Payments through r-cash

r-cash is the payments rail. r-share does not mint, custody or clear its own asset.

## Resolution

The Payments adapter stores only the r-cash Root URI string. Every arm resolves at entry:

```
rl!(rcashRootUri, rootCh)
Root!("resolve", "component:account-factory", 1, afCh)
Root!("resolve", "component:asset-resolver", 1, arCh)
Root!("resolve", "component:clock", 1, clkCh)
```

Pinned r-cash commit: `49b441d`. Asset allowlist starts at `REV`; other assets must resolve
through r-cash's AssetResolver and be verified before enabling.

## One atomic guest deploy

`GuestLedger.book` runs inside the guest's signed deploy:

1. Validate listing, units, schedules and slot states.
2. Hold the requested slots under a fresh `releaseToken`.
3. `Payments.issuer(assetId)` → brand + issuer; `issuer.makeEmptyPurse()`.
4. `Payments.ensureAccount(dId)` → the guest's r-cash account.
5. `account.withdraw(dId, assetId, rent + deposit)` → an ERTP payment.
6. `issuer.getAmountOf(payment)` must equal the total; otherwise `1/0`.
7. `purse.deposit(payment)`; instant bookings confirm the slots.
8. Spawn the BookingContract with the purse, release token and brand; insert its URI.

The guest never sees the escrow purse or the release token. The BookingContract holds them
in its closure; its state cell contains only data.

## Settlement and refunds

Every lifecycle payout withdraws exact amounts from the booking's escrow purse and deposits
them into r-cash account deposit facets:

```
AccountFactory.accountOf(ownerId) → account
account.depositFacet(assetId)     → receive-only facet
purse.withdraw((brand, amount))   → payment
facet.deposit(payment)
```

Buckets: `providerRent`, `platformFee`, `guestRefund`, `providerDamage`. Before paying,
the contract asserts `providerRent + platformFee + guestRefund + providerDamage == rent +
deposit` and that the escrow's current amount matches. Deposit facets are receive-only and
cannot re-enter the booking.

## Failure modes

- r-cash unavailable before slots/value move → `(false, "payments unavailable")`.
- Withdrawal refused → release the slots and refuse; no value moved.
- Measurement mismatch or purse/deposit failure after value moved → `1/0`, which rolls back
  the entire deploy, including the ERTP withdrawal and slot holds.
- Settlement refused before the booking cell is consumed → escrow stays intact and is
  retryable.

## Why not TransferBook

r-cash's `TransferBook` is a secret-hash, all-or-nothing claim escrow with a sender-only
refund after expiry. Rentals need provider accept/reject, rent/fee/deposit splitting,
partial-cancellation refunds, damage claims, disputes and event redemption. A second escrow
layer would be required. Direct account withdrawal into a per-booking purse gives one
authoritative cell and exact multi-payout settlement.
