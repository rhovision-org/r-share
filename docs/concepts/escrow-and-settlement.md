# Escrow and settlement

> **Status:** Implemented. **Applies to:** every paid booking.

r-cash is the payments rail. r-share does not mint, custody or clear its own asset. It holds
a per-booking ERTP purse and settles exact buckets into r-cash account deposit facets.

## Resolution

The Payments adapter stores only the r-cash Root URI string. Every arm resolves at entry:

```
rl!(rcashRootUri, rootCh)
Root!("resolve", "component:account-factory", 1, afCh)
Root!("resolve", "component:asset-resolver", 1, arCh)
Root!("resolve", "component:clock", 1, clkCh)
```

Pinned r-cash commit: `49b441d`. The asset allowlist starts at `REV`; other assets must
resolve through r-cash's AssetResolver and be verified before enabling.

## One atomic guest deploy

`GuestLedger.book` runs inside the guest's signed deploy:

1. Validate the listing, units, schedules and slot states.
2. Hold the requested slots under a fresh release token.
3. `Payments.issuer(assetId)` → brand + issuer.
4. `Payments.ensureAccount(dId)` → the guest's r-cash account;
   `issuer.makeEmptyPurse()`.
5. `account.withdraw(dId, assetId, rent + deposit)` → an ERTP payment.
6. `issuer.getAmountOf(payment)` must equal the total; otherwise `1/0`.
7. `purse.deposit(payment)`; instant bookings confirm the slots.
8. Create the per-booking `Bond` with the purse and expected total; spawn the BookingContract
   with the Bond URI, release token and a fresh settle token; insert its URI.

The guest never sees the escrow purse or the release token. The GuestLedger creates a
per-booking `Bond` that owns the purse; the BookingContract stores the Bond URI and closes
over a fresh `settleToken`. The Bond is the only component that can move the money.

## Settlement

Every lifecycle payout is delegated to the per-booking `Bond`. `Bond.pay` consumes its
state, checks the settle token and the expected total, resolves Payments once, withdraws
exact amounts from the escrow purse, deposits them into r-cash account deposit facets, and
posts `paid: true`. A second `pay` is `"closed"`.

```
AccountFactory.accountOf(ownerId) → account
account.depositFacet(assetId)     → receive-only facet
purse.withdraw((brand, amount))   → payment
facet.deposit(payment)
```

The buckets are:

| Bucket | Recipient |
|---|---|
| `providerRent` | the provider |
| `platformFee` | the platform owner configured in Payments |
| `guestRefund` | the guest |
| `providerDamage` | the provider, from the deposit |

The BookingContract asserts `expected == rent + deposit`; the Bond asserts the bucket sum
and ends at zero. A mismatch fails the deploy. Deposit facets are receive-only; they cannot
re-enter the booking.

## Failure modes

- r-cash unavailable before slots or value move → `(false, "payments unavailable")`.
- Withdrawal refused → release the slots and refuse; no value moved.
- Measurement mismatch or purse/deposit failure after value moved → `1/0`, which rolls back
  the entire deploy, including the ERTP withdrawal and the slot holds.
- Settlement refused before the booking cell is consumed → escrow stays intact and is
  retryable.

## Why not r-cash TransferBook

r-cash's `TransferBook` is a secret-hash, all-or-nothing claim escrow with a sender-only
refund after expiry. Rentals need provider accept/reject, rent/fee/deposit splitting, partial
cancellation refunds, damage claims, disputes and event redemption. A second escrow layer
would be required. Direct account withdrawal into a per-booking purse gives one authoritative
cell and exact multi-payout settlement.

## The platform fee

`Payments.platformOwner` is configured at install. The install also ensures that account
exists, so a settlement to the platform cannot fail for a missing account. A fork should
decide whether it wants a platform fee at all; setting `feeBps` to zero is allowed.

## What is not escrowed

- Off-chain deposits, insurance or security held by a regulated provider.
- Usage metering beyond `pricePerSlot * slotCount`.
- Fiat on-ramp or off-ramp.
- Tax withholding.

> **Where to go deeper.** [Bookings and lifecycle](bookings-and-lifecycle.md) shows when each
> payout happens. [Payments](../reference/system-channels.md) lists the channels. [Operate
> safely](../guides/operate-safely.md) covers key and fee configuration.
