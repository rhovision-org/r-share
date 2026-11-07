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
ShareRoot("nonce", ret) -> (true, govNonce)
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
ShareAdmin("nonce", ret) -> (true, nonce)
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
                                   | (false, "asset" | "rcash unavailable")
Payments("platformOwner", ret) -> (true, ownerId)
Payments("admin", @nonce, @action, @args, @sigs, ret)
  -> (true, Nil) | (false, "replay" | "below threshold" | "asset" | "rcash unavailable" | "unknown action")
```

`Payments.issuer` refuses `"asset"` unless the asset id is in `paymentsState.assets`.
`setAssets` refuses an empty list and never removes `REV` implicitly; the admin must pass a
new list.

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
ProviderContract("removeListing", dId, @listingId, ret)
  -> (true, Nil) | (false, "not owner" | "unknown listing" | listing delist reason)
ProviderContract("adminPause", @nonce, @sigs, ret) -> (true, Nil) | (false, "replay" | "below threshold")
ProviderContract("setAdmins", @nonce, @newAdmins, @newThreshold, @sigs, ret)
  -> (true, Nil) | (false, "replay" | "below threshold" | "threshold")
```

## Unit helpers — `13_unit.rho`

The unit and slot components are gone. `UnitFactory` is stateless and validates schedules
or scans a unit's slot map for the listing allocator.

```
UnitFactory("validate", @kind, @slots, @maxLeadBlocks, @maxSlotBlocks, @contiguous, ret)
  -> (true, Nil) | (false, schedule reason)
UnitFactory("schedule", @specs, ret) -> (true, {slotId -> SlotSpec})
UnitFactory("slotScan", @ids, @specMap, @stateMap, @now, @maxLead, @maxSlot, ret)
  -> (true, {count, minStart, maxEnd}) | (false, "unknown slot" | "schedule" |
                                           "lead" | "duration" | "slot held")
```

## Listing allocator — `14_listing_contract.rho`

```
ListingFactory("create", dId, @providerUri, @listingId, @kind, @terms, ret)
  -> (true, listingUri)
   | (false, "not provider" | "terms" | "codec" | "listing factory unavailable")

ListingContractFactory("create", dId, @providerId, @listingId, @kind, @terms,
                       @termsHash, @admins, @threshold, ret)
  -> (true, listingUri) | (false, "not owner")
```

`ListingContract` owns units and slots in one `listingState` cell:

```
ListingContract("info", ret) -> (true, state without slots)
ListingContract("terms", ret) -> (true, terms)
ListingContract("isActive", ret) -> (true, Bool)
ListingContract("units", ret) -> (true, {unitId -> unit record})
ListingContract("unit", @unitId, ret) -> (true, unit record) | (false, "unknown unit")
ListingContract("schedule", @unitId, ret) -> (true, {slotId -> SlotSpec})
                                            | (false, "unknown unit")
ListingContract("quote", @unitSlots, ret) -> (true, summary) | all acquire reasons
ListingContract("acquire", @bookingId, @unitSlots, @untilHeight, releaseToken, ret)
  -> (true, {count, rent, minStart, maxEnd, checkIn, checkOut})
   | (false, "not active" | "units" | "slots" | "duplicate unit" |
             "unknown unit" | "unit paused" | "until" | "unknown slot" |
             "slot held" | "schedule" | "lead" | "duration" | "window" |
             "paused" | "provider paused" | "unit factory unavailable" | "clock")
ListingContract("confirm", @unitSlots, releaseToken, ret)
  -> (true, Nil) | (false, "slot lost" | "not holder" | "unknown unit" |
                           "unknown slot" | "clock")
ListingContract("release", @unitSlots, releaseToken, ret)
  -> (true, Nil) | (false, "not holder" | "unknown unit" | "unknown slot")
ListingContract("probe", @unitSlots, ret)
  -> (true, {unitId -> {slotId -> ["free", gen] | ["held", bookingId, until] |
                                     ["booked", bookingId]}})
ListingContract("reset", dId, @unitId, @slotId, ret)
  -> (true, Nil) | (false, "not owner" | "unknown unit" | "unknown slot" |
                           "slot held" | "slot booked" | "clock")
ListingContract("verifyHold", @bookingId, @unitSlots, releaseToken, ret)
  -> (true, Nil) | (false, "slot lost" | "unknown unit" | "unknown slot")
ListingContract("activate", dId, ret) -> (true, Nil) | (false, "not owner" | "already" | "no units")
ListingContract("pause", dId, ret) -> (true, Nil) | (false, "not owner" | "delisted")
ListingContract("resume", dId, ret) -> (true, Nil) | (false, "not owner" | "delisted")
ListingContract("delist", dId, ret) -> (true, Nil) | (false, "not owner")
ListingContract("pauseUnit", dId, @unitId, ret) -> (true, Nil) | (false, "not owner" | "unknown unit")
ListingContract("resumeUnit", dId, @unitId, ret) -> (true, Nil) | (false, "not owner" | "unknown unit")
ListingContract("ensureUnit", dId, @unitId, @attrs, @slots, @pricePerSlot, ret)
  -> (true, unitId) | (false, "not active" | "not owner" | "unit exists" |
                         "unit factory unavailable" | "price" | "refund window" |
                         schedule reason)
ListingContract("adminPause", @nonce, @action, @args, @sigs, ret)
  -> (true, Nil) | (false, "replay" | "below threshold" | "unknown action")
ListingContract("setAdmins", @nonce, @newAdmins, @newThreshold, @sigs, ret)
  -> (true, Nil) | (false, "replay" | "below threshold" | "threshold")
```

`delisted` is terminal for the listing state: `pause` and `resume` (owner and admin) refuse
it. `removeListing` calls `delist` first and only marks the provider index removed after the
listing refuses to stay active; a failed delist reposts the provider index unchanged and
returns the failure.

`acquire` is one take/await/put: it takes `listingState`, checks the global pause and
provider gates, awaits `UnitFactory.slotScan`, and reposts every requested slot as held or
the unchanged state. Other listing arms block while the cell is empty. `confirm` and
`release` are token-gated. The release token is never returned.

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
  -> (true, bookingUri) | (false, "not guest" | "reservation" | "listing")
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
BookingContract("claimNoShow", dId, @opId, ret) -> (true, Nil) | (false, "not owner" | "closed" | "too early" | "window" | "replay")
BookingContract("disputeNoShow", dId, @opId, @reasonHash, ret) -> (true, Nil) | (false, "not owner" | "closed" | "too late" | "replay")
BookingContract("resolveNoShow", @opId, ret) -> (true, Nil) | (false, "closed" | "too early" | "replay")
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

## Cover terms and offers — `30_cover_terms.rho`, `31_cover_offer.rho`

```
CoverTerms("validate", @offer, ret) -> (true, Nil) | (false, reason)
CoverTerms("hash", @offer, ret) -> (true, digest)

CoverBook("publish", dId, @offer, ret) -> (true, offerId, digest)
  | (false, "publisher" | "platform" | "payer" | "premium" | "offer exists" | "retired")
CoverBook("offer", @offerId, ret) -> (true, entry) | (false, "unknown offer")
CoverBook("offers", ret) -> (true, offers)
CoverBook("pause", dId, @offerId, ret) -> (true, Nil)
  | (false, "publisher" | "unknown offer" | "retired")
CoverBook("retire", dId, @offerId, ret) -> (true, Nil)
  | (false, "publisher" | "unknown offer")
```

`CoverTerms` enforces the premium-payer rule: the payer is the insured or the platform,
never the counterparty. It also enforces the coverage cap/deductible, premium floor, claim
window, evidence schema and the K-Asset/insurer pricing fields. `CoverBook` stores an
immutable offer record keyed by `offerId`; a platform-funded offer must be published by the
platform owner.


## Cover pool — `32_cover_pool.rho`

```
CoverPool("fund", dId, @payment, ret) -> (true, amount)
CoverPool("premium", @payment, ret) -> (true, amount)
CoverPool("reserve", dId, @payerId, @payment, ret)
  -> (true, amount) | (false, "reserve cap")
CoverPool("allocate", dId, @policyId, @cap, ret)
  -> (true, policyPurse) | (false, "paused" | "cap" | "policy exists" |
                                   "capacity" | "solvency")
CoverPool("bind", dId, @subject, @offerId, @cap, @payment, @policyId,
          @attachKey, ret) -> (true, policyId, policyUri)
CoverPool("bindReserved", dId, @subject, @offerId, @cap, @policyId,
          @attachKey, @unitSlots, releaseToken, ret)
  -> (true, policyId, policyUri) | (false, "hold" | "guest" | "reserve")
CoverPool("policy", @policyId, ret) -> (true, entry) | (false, "unknown policy")
CoverPool("release", dId, @policyId, @payment, @paidAmount, ret)
  -> (true, amount) | (false, "unknown policy" | "attached" | "amount")
CoverPool("close", dId, @policyId, @paidAmount, ret)
  -> (true, Nil) | (false, "unknown policy" | "attached" | "amount")
CoverPool("info", ret) -> (true, state)
CoverPool("solvency", ret) -> (true, {assets, cash, outstanding, solvent})
CoverPool("configure", dId, @capacity, @solvencyBps, @reserveCap, ret)
  -> (true, Nil) | (false, "config")
CoverPool("pause", dId, ret) -> (true, Nil)
CoverPool("resume", dId, ret) -> (true, Nil)
```

`reserve` prefunds a provider/platform reserve and refuses a deposit above `reserveCap`
without changing the reserve. `configure`, `pause` and `resume` are operator-only. `bind` and `bindReserved` validate
an active offer and listing, reserve the cap, create a per-policy purse and call
`CoverPolicyFactory("spawn", ...)`. `bind` takes an exact premium payment;
`bindReserved` debits the payer reserve, authenticates the guest, and requires a live
`Listing("verifyHold")` for the booking id, slot set and release token. The private
`CoverSettle` capability created in the same call records loss-ledger payouts and
measures the returned cover at settlement. `cash` is free cash
(`assets - outstanding`). All pool mutations except the permissionless premium
deposit and reserve-funded bind are operator- or payer-gated.

## Cover policy — `33_cover_policy.rho`

```
CoverPolicyFactory("spawn", @spec, @settleCh, @purse, @payments, @clock, ret)
  -> (true, policyUri)
CoverPolicy("pay", dId, @claimId, @lossId, @amount, @assessed,
            @nonCoverRecovery, @evidenceHash, ret)
  -> (true, amount) | (false, "attached")
CoverPolicy("authorize", dId, @claimId, @lossId, @amount, @assessed,
            @nonCoverRecovery, @evidenceHash, ret)
  -> (true, claimToken) | (false, "not attached" | "claim exists" | "expired" | "amount")
CoverPolicy("settle", dId, ret) -> (true, Nil)
CoverPolicy("expire", ret)
  -> (true, Nil) | (false, "too early" | "attached")
CoverPolicy("attach", @subjectUri, @subjectId, @expiresAtHeight, @key, ret)
  -> (true, Nil)
CoverPolicy("info", ret) -> (true, redactedState)
claimToken("pay", ret) -> (true, amount)
claimToken("info", ret) -> (true, fixedClaim)
```

`pay` is the operator/manual adjudication path for an unattached policy only.
`authorize` is attached-only, refuses a repeated claim id, and mints the fixed claim token;
the booking calls it through `attachClaim` and `bcCoverPay`, and it pays once. `expire`
refuses an attached policy. `settleKey` is the attached booking's release path and requires
the booking's `settleToken` key; a missing pool entry is treated as already settled. No
purse, attach key or token name appears in `info`.

## Covered booking — `23_guest_ledger.rho`, `24_guest_ledger_cover.rho`,
## `34_booking_instance.rho`

```
GuestLedger("bookWithCover", dId, @listingUri, @unitSlots, @opId,
            @expectedTermsHash, @offerId, @policyId, ret)
  -> (true, bookingId, bookingUri) | (false, reason)
CoverGuestLedger("book", dId, @guestId, @listingUri, @unitSlots, @opId,
                 @expectedTermsHash, @offerId, @policyId, ret)
  -> (true, bookingId, bookingUri) | (false, reason)
BookingContract("attachCover", dId, @policyUri, @policyId, ret)
  -> (true, Nil) | (false, reason)
BookingContract("attachClaim", dId, @claimId, claimToken, ret)
  -> (true, Nil) | (false, reason)
```

`bookWithCover` is a thin guest-auth/gate/dispatch arm. The atomic flow validates
the active offer and listing, acquires slots, creates `releaseToken` and
`settleToken`, binds reserved cover, creates the Bond for `rent + effectiveDeposit`,
creates the booking, and attaches the policy in one deploy. Any failure after bind
is `1/0`, so slots, reserve, premium, Bond and booking roll back together.
The booking info merges a redacted `cover` map; no token, key or purse is included.
Cover payout is operator-adjudicated; the booking verifies the policy `paid` amount
before paying the deposit part and closes the policy with its settle key. `claimDeposit`
is one-shot; a second claim and a dispute with a pending or attached claim are refused.
`resolveDeposit` refuses with `"cover"` if the fixed claim token has not been attached,
rather than blocking on an empty claim cell.
