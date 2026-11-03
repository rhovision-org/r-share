# Arm reference

> **Status:** Implemented. This chapter is derived from `contracts/src/deploy/*.rho`; the
> source is the authority.

Every public arm is a send whose first field is the arm name. `ret`, `ret2`, `ret3` are reply
channels and reply exactly once. `dId` is a deployer identity name (send it with `*`); `@x`
is data; `*x` is a name.

Conventions:

- Replies are `(true, value)` or `(false, reason)`.
- A refusal reposts the state cell; a terminal booking is permanent.
- A successful mutating booking arm records its `opId`; a replay returns the stored result.
- The `ret` channel is always the last argument unless noted.

## Root — `00_root.rho`

```
ShareRoot("resolve", @key, @minVersion, ret) -> (true, uri, version)
                                             | (false, "unknown component" | "retired" | "stale")
ShareRoot("resolveMany", @keys, @minVersion, ret)
  -> (true, {key -> {uri, version}}) | (false, "unknown component" | "retired" | "stale", key)
ShareRoot("get", @key, ret) -> (true, entry) | (false, "unknown component")
ShareRoot("keys", ret) -> (true, keys)
ShareRoot("register", @key, @uri, @version, @kind, dId, ret)
  -> (true, Nil) | (false, "frozen" | "not installer" | "bad version" | "duplicate")
ShareRoot("rotate", @key, @expectedVersion, @newUri, @newVersion, @nonce, @sigs, ret)
  -> (true, Nil) | (false, "replay" | "below threshold" | "unknown component" | "stale")
ShareRoot("deprecate", @key, @nonce, @sigs, ret)
  -> (true, Nil) | (false, "replay" | "below threshold" | "unknown component")
ShareRoot("freeze", @nonce, @sigs, ret) -> (true, Nil) | (false, "replay" | "below threshold")
ShareRoot("admin", @nonce, @action, @args, @sigs, ret)
  -> (true, Nil) | (false, "replay" | "below threshold" | "threshold" | "unknown action")
```

## Kernel — `01_kernel.rho`

```
Clock("now", ret) -> (true, blockHeight)
Verifier("hash", @bytes, ret) -> (true, digest)
Verifier("sigOk", @digest, @sig, @pub, ret) -> (true, Bool)
Codec("canonical", @schema, @items, ret) -> (true, text)
Codec("hash", @schema, @items, ret) -> (true, digest)
Codec("adminPayload", @domain, @nonce, @action, @args, ret) -> (true, text)
```

## Admin — `02_admin.rho`

```
AdminGuard("verify", @domain, @nonce, @action, @args, @sigs, @admins, @threshold, ret)
  -> (true, Bool) | (false, "below threshold")
ShareAdmin("keys", ret) -> (true, admins, threshold)
ShareAdmin("isPaused", ret) -> (true, Bool)
ShareAdmin("admin", @nonce, @action, @args, @sigs, ret)
  -> (true, Nil) | (false, "replay" | "below threshold" | "threshold" | "unknown action")
```

## Payments — `40_payments.rho`

```
Payments("health", ret) -> (true, {accountFactory, assetResolver, clock})
                         | (false, "rcash unavailable", key)
Payments("ensureAccount", dId, ret) -> (true, account, ownerId)
                                    | (false, "rcash unavailable" | "account")
Payments("accountOf", @ownerId, ret) -> (true, account) | (false, "no account" | "rcash unavailable")
Payments("depositFacet", @ownerId, @assetId, ret) -> (true, facet)
                                                   | (false, "no account" | "rcash unavailable")
Payments("issuer", @assetId, ret) -> (true, brand, issuer)
                                   | (false, "unknown asset" | "rcash unavailable")
Payments("platformOwner", ret) -> (true, ownerId)
Payments("admin", @nonce, @action, @args, @sigs, ret)
  -> (true, Nil) | (false, "replay" | "below threshold" | "rcash unavailable" | "unknown action")
```

## Terms — `03_terms.rho`

```
Terms("fields", ret) -> (true, [field])
Terms("values", @kind, @terms, ret) -> (true, [value])
Terms("validate", @kind, @terms, ret) -> (true, Nil) | (false, field)
Terms("hash", @kind, @terms, ret) -> (true, digest) | (false, "codec")
Terms("booking", @terms, ret) -> (true, projection)
Terms("remedy", @terms, ret) -> (true, projection)
Terms("scheduleOk", @kind, @slots, @now, @maxLeadBlocks, @maxSlotBlocks, @contiguous, ret)
  -> (true, Nil) | (false, "slots" | "slot id" | "duplicate slot" | "slot order" |
                    "slot past" | "slot lead" | "slot duration" | "slot gap")
```

`Terms` owns the only ordered field list. `booking` and `remedy` return the projections the
booking and ledger consume; neither caller repeats the field order.

## Remedy — `04_remedy.rho`

```
Remedy("fee", @rent, @feeBps, ret) -> (true, fee, net) | (false, "bps")
Remedy("cancel", @kind, @policy, @now, @startHeight, @rent, @deposit, @actor, ret)
  -> (true, Settlement) | (false, "actor" | "bps")
Remedy("noShow", @rent, @deposit, @feeBps, ret) -> (true, Settlement) | (false, "bps")
Remedy("depositSplit", @deposit, @providerBps, ret)
  -> (true, {providerDamage, guestRefund}) | (false, "bps")
```

`Remedy` is the only place with cancellation/fee/no-show/deposit arithmetic. It is
registered as `component:remedy`.

## Provider — `10_provider.rho`

```
ProviderBook("register", dId, @profile, ret) -> (true, providerUri, ownerId)
                                              | (false, "not owner" | "account" | "profile")
ProviderBook("providerOf", @ownerId, ret) -> (true, uri) | (false, "no provider")
```

`ProviderContract` (spawned per provider):

```
ProviderContract("info", ret) -> (true, state)
ProviderContract("isActive", ret) -> (true, Bool)
ProviderContract("owner", ret) -> (true, providerId)
ProviderContract("profile", ret) -> (true, profile)
ProviderContract("listings", ret) -> (true, listings)
ProviderContract("listing", @listingId, ret) -> (true, entry) | (false, "unknown listing" | "removed")
ProviderContract("checkOwner", dId, ret) -> (true, Nil) | (false, "not owner")
ProviderContract("updateProfile", dId, @newProfile, @opId, ret) -> (true, Nil) | (false, "not owner" | "replay")
ProviderContract("pause", dId, ret) -> (true, Nil) | (false, "not owner" | "already")
ProviderContract("resume", dId, ret) -> (true, Nil) | (false, "not owner" | "already")
ProviderContract("registerListing", dId, @listingId, @uri, @kind, @termsHash, ret)
  -> (true, Nil) | (false, "paused" | "listing exists" | "not owner")
ProviderContract("removeListing", dId, @listingId, ret) -> (true, Nil) | (false, "not owner" | "unknown listing")
ProviderContract("adminPause", @nonce, @sigs, ret) -> (true, Nil) | (false, "replay" | "below threshold")
ProviderContract("setAdmins", @nonce, @newAdmins, @newThreshold, @sigs, ret)
  -> (true, Nil) | (false, "replay" | "below threshold" | "threshold")
```

## Unit — `13_unit.rho`

```
UnitFactory("validate", @kind, @slots, @maxLeadBlocks, @maxSlotBlocks, @contiguous, ret)
  -> (true, Nil) | (false, schedule reason)
UnitFactory("create", dId, @unitId, @providerId, @listingId, @kind, @attrs, @slots,
            @maxHoldBlocks, @admins, @threshold, @pricePerSlot, ret)
  -> (true, unitUri) | (false, "not owner" | "price")
UnitFactory("scan", @unitSlots, @listingId, @now, @maxLead, @maxSlot, ret)
  -> (true, {count, minStart, maxEnd, checkIn, checkOut, rent}) | (false, reason)
UnitFactory("holdAll", @unitSlots, @bookingId, @until, releaseToken, ret)
  -> (true, Nil) | (false, slot reason)
UnitFactory("releaseAll", @unitSlots, releaseToken, ret) -> (true, Nil)
UnitFactory("confirmAll", @unitSlots, releaseToken, ret) -> (true, Nil) | (false, "slot lost")
```

`UnitContract` (spawned per unit):

```
UnitContract("info", ret) -> (true, state)
UnitContract("slots", ret) -> (true, [slotId])
UnitContract("schedule", @slotIds, ret) -> (true, {slotId -> SlotSpec}) | (false, "unknown slot")
UnitContract("probe", @slotIds, ret) -> (true, {slotId -> state}) | (false, "unknown slot")
UnitContract("hold", @bookingId, @slotIds, @untilHeight, releaseToken, ret)
  -> (true, Nil) | (false, "unit paused" | "until" | "unknown slot" | "slot held")
UnitContract("release", @slotIds, releaseToken, ret) -> (true, Nil)
UnitContract("confirm", @slotIds, releaseToken, ret) -> (true, Nil) | (false, "slot lost")
UnitContract("reset", dId, @slotId, ret) -> (true, Nil) | (false, "not owner" | "unknown slot")
UnitContract("adminPause", @nonce, @sigs, ret) -> (true, Nil) | (false, "replay" | "below threshold")
UnitContract("setAdmins", @nonce, @newAdmins, @newThreshold, @sigs, ret)
  -> (true, Nil) | (false, "replay" | "below threshold" | "threshold")
```

`SlotContract` (spawned per slot):

```
SlotContract("probe", ret) -> (true, ["free", gen] | ["held", bookingId, until] | ["booked", bookingId])
SlotContract("hold", @bookingId, @untilHeight, @now, releaseToken, ret)
  -> (true, Nil) | (false, "slot held", until)
SlotContract("release", releaseToken, ret) -> (true, Nil) | (false, "not holder")
SlotContract("confirm", releaseToken, ret) -> (true, Nil) | (false, "not held")
SlotContract("reset", resetToken, ret) -> (true, Nil) | (false, "not provider")
```

## Listing — `14_listing_contract.rho`

```
ListingFactory("create", dId, @providerUri, @listingId, @kind, @terms, ret)
  -> (true, listingUri)
   | (false, "not provider" | "terms" | "codec" | "listing factory unavailable")

ListingContractFactory("create", dId, @providerId, @listingId, @kind, @terms,
                       @termsHash, @admins, @threshold, ret)
  -> (true, listingUri) | (false, "not owner")
```

`ListingContract` (spawned per listing):

```
ListingContract("info", ret) -> (true, state)
ListingContract("terms", ret) -> (true, terms)
ListingContract("isActive", ret) -> (true, Bool)
ListingContract("units", ret) -> (true, {unitId -> unitUri})
ListingContract("unit", @unitId, ret) -> (true, uri) | (false, "unknown unit")
ListingContract("activate", dId, ret) -> (true, Nil) | (false, "not owner" | "already" | "no units")
ListingContract("pause", dId, ret) -> (true, Nil) | (false, "not owner")
ListingContract("resume", dId, ret) -> (true, Nil) | (false, "not owner")
ListingContract("delist", dId, ret) -> (true, Nil) | (false, "not owner")
ListingContract("ensureUnit", dId, @unitId, @attrs, @slots, @pricePerSlot, ret)
  -> (true, unitUri) | (false, "not active" | "not owner" | "unit exists" |
                         "unit factory unavailable" | "price" | schedule reason)
ListingContract("adminPause", @nonce, @action, @args, @sigs, ret)
  -> (true, Nil) | (false, "replay" | "below threshold" | "unknown action")
ListingContract("setAdmins", @nonce, @newAdmins, @newThreshold, @sigs, ret)
  -> (true, Nil) | (false, "replay" | "below threshold" | "threshold")
```

## Guest — `20_booking.rho` and `23_guest_ledger.rho`

```
GuestBook("getOrCreate", dId, ret) -> (true, ledgerUri, guestId)
GuestBook("ledger", @guestId, ret) -> (true, uri) | (false, "no ledger")
GuestBook("ledgers", ret) -> (true, ledgers)

GuestLedgerFactory("create", dId, @guestId, ret) -> (true, ledgerUri) | (false, "not owner" | "account")
GuestLedger("ops", dId, ret) -> (true, ops) | (false, "not owner")
GuestLedger("book", dId, @listingUri, @unitSlots, @opId, @expectedTermsHash, ret)
  -> (true, bookingId, bookingUri)
   | (false, "not owner" | "pending" | "paused" | "provider paused" | "listing inactive" |
             "terms" | "slots" | "units" | "schedule" | "slot held" |
             "payments unavailable" | "account" | reason)
```

## Bond — `15_bond.rho`

```
BondFactory("create", escrowPurse, brand, @assetId, @expected, settleToken, ret)
  -> (true, bondUri)
BondFactory("createFromAccount", dId, @assetId, @expected, settleToken, ret)
  -> (true, bondUri) | (false, withdraw reason)
Bond("pay", @buckets, @expected, settleToken, ret)
  -> (true, Nil) | (false, "closed" | "not holder" | "amount")
Bond("balance", ret) -> (true, amount)
Bond("info", ret) -> (true, {paid, expected, remaining})
```

## Booking — `22_booking_contract.rho`

```
BookingContractFactory("create", dId, @data, releaseToken, @bondUri, settleToken, ret)
  -> (true, bookingUri) | (false, "not guest" | "bond factory unavailable")
```

The factory is registered as `component:booking-factory`. `GuestLedger` creates the Bond
first, then passes its URI and the fresh `settleToken` to the factory.

`BookingContract` (spawned per booking):

```
BookingContract("info", ret) -> (true, state)
BookingContract("status", ret) -> (true, status)
BookingContract("ticket", ret) -> (true, ticket)
BookingContract("ops", ret) -> (true, ops)
BookingContract("accept", dId, @opId, ret) -> (true, Nil) | (false, "not owner" | "closed" | "expired" | "slot lost" | "replay")
BookingContract("reject", dId, @opId, @reason, ret) -> (true, Nil) | (false, "not owner" | "closed" | "replay")
BookingContract("cancel", dId, @opId, ret) -> (true, Nil) | (false, "not owner" | "closed" | "bps" | "replay")
BookingContract("cancelByProvider", dId, @opId, ret) -> (true, Nil) | (false, "not owner" | "closed" | "replay")
BookingContract("checkIn", dId, @opId, ret) -> (true, Nil) | (false, "not owner" | "closed" | "too early" | "too late" | "replay")
BookingContract("checkOut", dId, @opId, ret) -> (true, Nil) | (false, "not owner" | "closed" | "replay")
BookingContract("guestCheckOut", dId, @opId, ret) -> (true, Nil) | (false, "not owner" | "closed" | "too early" | "replay")
BookingContract("settle", @opId, ret) -> (true, Nil) | (false, "closed" | "claim pending" | "too early" | "bps" | "replay")
BookingContract("claimDeposit", dId, @opId, @amount, @evidenceHash, ret) -> (true, Nil) | (false, "not owner" | "closed" | "too late" | "amount" | "replay")
BookingContract("resolveDeposit", @opId, ret) -> (true, Nil) | (false, "closed" | "not claimed" | "too early" | "bps" | "replay")
BookingContract("dispute", dId, @opId, @reasonHash, ret) -> (true, Nil) | (false, "not party" | "closed" | "replay")
BookingContract("resolveDispute", @opId, ret) -> (true, Nil) | (false, "closed" | "too early" | "release" | "bps" | "replay")
BookingContract("arbitrate", @nonce, @splitBps, @sigs, @opId, ret) -> (true, Nil) | (false, "replay" | "below threshold" | "bps" | "closed")
BookingContract("expire", @opId, ret) -> (true, Nil) | (false, "closed" | "too early" | "replay")
BookingContract("claimNoShow", dId, @opId, ret) -> (true, Nil) | (false, "not owner" | "closed" | "too early" | "bps" | "replay")
BookingContract("redeem", dId, @opId, ret) -> (true, Nil) | (false, "not owner" | "not event" | "closed" | "already" | "replay")
BookingContract("closeEvent", @opId, ret) -> (true, Nil) | (false, "not event" | "closed" | "too early" | "bps" | "replay")
BookingContract("transferTicket", dId, @newGuestId, @opId, ret) -> (true, Nil) | (false, "not owner" | "not event" | "closed" | "too late" | "deposit" | "replay")
BookingContract("setAdmins", @nonce, @newAdmins, @newThreshold, @sigs, ret) -> (true, Nil) | (false, "replay" | "below threshold" | "threshold")
```

## Refusal index

| Refusal | Meaning |
|---|---|
| `"not owner"` / `"not provider"` / `"not guest"` / `"not party"` | caller is not the authority |
| `"closed"` | terminal or wrong state |
| `"replay"` | nonce or opId already used |
| `"slot held"` | another hold or booking owns the slot |
| `"slot lost"` | the hold was reclaimed before confirm |
| `"unknown slot"` | the slot ID is not on the unit |
| `"listing inactive"` / `"not active"` / `"paused"` | listing or provider is not bookable |
| `"terms"` | terms validation or hash mismatch |
| `"amount"` / `"bps"` / `"threshold"` | numeric bound failed |
| `"too early"` / `"too late"` | height window failed |
| `"below threshold"` | fewer than 2 valid signatures |
| `"payments unavailable"` / `"rcash unavailable"` | the rail cannot be resolved |
| `"pending"` | the same opId is in flight |
| `"paused"` / `"provider paused"` / `"unit paused"` | a pause gate refused the action |
| `"not holder"` | the Bond or slot token does not match |
| `"bond factory unavailable"` | the Bond factory could not be resolved |
| `"codec"` | the Terms hash could not resolve Codec |

> **Where to go deeper.** [Contract map](contract-map.md) is the component list. [Data
> schemas](data-schemas.md) is the state shapes. [Bookings and lifecycle](../concepts/bookings-and-lifecycle.md)
> is the transition model.
