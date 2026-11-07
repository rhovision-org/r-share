# Design note — closing the free `Listing.acquire` hold (HZ-061)

Status: design spike, not implemented.

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

## Recommendation

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
