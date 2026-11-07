# Design note — closing the free `Listing.acquire` hold (HZ-061)

Status: implemented (option B + the funded-booking confirm gate).

## Problem

`ListingContract("acquire")` is permissionless and takes no payment, escrow reference or
booking proof. Any caller can acquire up to `maxUnits` and hold every slot for
`maxHoldBlocks`, then repeat. This is an availability/griefing issue, not a race: the
allocator cell is serialized.

## Requirements

- A legitimate booking must still acquire/confirm/release in one deploy.
- A direct caller must not be able to create a free hold.
- The proof must be a capability or a funded commitment, not a caller-supplied string.
- No new behavior may bypass the allocator's all-or-none and token-gated confirm/release.
- Keep the arm inventory and existing slot semantics intact.

## Options

### A. Ledger-issued hold proof (recommended direction)

`GuestLedger` mints a private `holdKey` for the booking attempt and registers
`(bookingId, unitSlots, holdKey)` in a `HoldBook`. `Listing.acquire` requires the key and asks
`HoldBook` to consume the entry before writing the hold. A direct caller cannot mint a valid
entry.

Open design point: `HoldBook` must authenticate the issuer. A guest can create a ledger, so
"the entry came from a ledger" is not enough; the entry must be bound to a funded booking
attempt (Bond or escrow). That tie is the real control. If the tie cannot be made cheaply,
this option does not close the DoS.

### B. Refundable hold deposit

`Listing.acquire` takes a small payment, held by the listing purse and refunded on
`release`/`confirm`. This directly prices the hold and is simpler than issuer
authentication, but changes the acquire API and the economics. It still needs a purse and a
refund path.

### C. Accept and monitor

Keep the public arm; rely on `maxUnits`, `maxHoldBlocks`, pause and off-chain monitoring.
Document HZ-061 as accepted. This is honest but does not eliminate the exploit.

## Implemented design

Option B plus the confirm gate. `Listing.acquire` takes a payment equal to
`terms.pricePerSlot`; `41_hold_deposit.rho` holds it by `bookingId`. The listing calls
`HoldDeposit("settle", bookingId, "release", ...)` after a successful release and the
ledgers/booking call `"confirm"` after a successful confirm. A settle is refused while the
listing `probe` still reports the booking's hold; after the hold window a release forfeits
the deposit to the provider. `BookingInstanceFactory("spawn")` verifies a real Bond via
`Bond("verify")` and `BookingFactory("verify")` looks the booking up in a registry; the
listing's `confirm` calls `HoldDeposit("authorize")`, which requires both the deposit record
and the verified funded booking. This closes the acquire→confirm permanent-hold path that
the deposit alone would leave open.

The release probe is anchored to the listing the **provider** registered, not a
caller-supplied URI: the record stores `providerUri`/`listingId`/`listingKey`, and
`HoldDeposit("settle", "release")` resolves `ProviderContract("listingUri", listingId)` and
probes that URI. A caller cannot substitute a free listing to unlock the deposit while still
holding. The confirm gate compares the booking's `listingKey` with the record's, so a funded
booking on one listing cannot authorize a confirm on another. The spawn also asks the trusted
`BondFactory("issued")` index whether the bond URI was factory-issued before it calls the
bond's `verify`, so a caller-supplied fake Bond cannot satisfy the funded-booking gate, and
it resolves the provider book and `ProviderContract("listingUri", listingId)` to reject a
caller-supplied listing URI that is not the provider's registered listing for that id.

Residual: a funded caller can still acquire and release inside the window at gas cost, and a
confirmed booking keeps the deposit locked until terminal release. The deposit is not a
per-release fee; adding one is a separate product decision. A direct caller must pass a real
payment and must settle explicitly; the legacy acquire arity is an unmatched contract
(silence), so all callers are updated. The flow adds several resolves (hold deposit, provider
book, bond index, booking verify); the client default phlo stays at 4M and must be measured
on the next runtime run before assuming a book deploy fits.

The earlier option A analysis is retained below as the design record.

## Recommendation (superseded)

Run a short design spike on A. The deliverable is a concrete answer to one question: **what
funded object does `HoldBook` verify before it issues a hold?** If the answer is "a Bond or
escrow that already exists", implement A. If it is not cheap, choose B (hold deposit) rather
than shipping a public token that any guest can mint for themselves.

## Test plan for the chosen option

- Direct `acquire` without the proof is refused; the listing slot map is unchanged.
- A legitimate `book`/`bookWithCover` still acquires, confirms and releases.
- The proof cannot be replayed after release or confirm.
- A stale/foreign proof is refused.
- Pause/delist still gates acquisition.
- A griefer cannot acquire more than `maxUnits` and cannot renew for free.

## Non-goals

- Do not add a `deployerId`-only check; it does not identify the caller.
- Do not accept a caller-supplied `"ledger"` URI as proof.
- Do not change the allocator's single-owner/confirm/release invariants.
