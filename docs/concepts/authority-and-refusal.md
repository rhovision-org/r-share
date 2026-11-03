# Authority and refusal

> **Status:** Implemented. **Applies to:** every public arm.

Rholang has no `msg.sender`. Authority is either a name you hold or a signature you present.
r-share derives every owner from the deployer's own `deployerId`; it never accepts an owner
address as a caller argument.

## Identity

- `providerId`, `guestId` and every owner id are REV addresses derived from the deploy key
  with `rho:rchain:deployerId:ops` + `rho:rev:address`.
- Owner arms call `authIsOwner(dId, ownerId)` and refuse `"not owner"`.
- The account gate inside r-cash does the same for withdrawals: the account re-derives the
  owner from the deployer and compares.
- A caller cannot forge another party's `deployerId`; it can only present its own.

## Capabilities

- A listing, unit or booking is reached by a URI, but the authoritative contract re-checks
  ownership on every owner arm. Holding a URI is not authority.
- The BookingContract's release token is a fresh unforgeable name created in the booking
  deploy. A slot releases only for the matching token; an expired hold reclaimed by another
  booking cannot be released by the old booking.
- The unit's reset token is held by the unit contract. A provider calls `Unit.reset`, which
  presents the token to the slot; the provider never handles the token.
- Deposit facets are receive-only. A booking can pay a party, not act as that party.

## Refusal ordering

- Refuse with `(false, reason)` **before** moving value.
- After any ERTP withdraw or deposit, an unexpected branch calls `1/0` so the deploy rolls
  back.
- Every refusal reposts the state cell; no arm leaves a cell empty.
- A `for` that never matches is silence, not an error. Reply shapes are matched
  exhaustively in the reviewed code; tests must assert the shape.

## Replay and idempotency

- GuestLedger reserves `opId` before any external call and records the result afterward.
  A replay returns the stored result and moves no money; a pending replay returns
  `"pending"`.
- BookingContract records a successful mutating arm by `opId` and returns it on replay.
- Admin and arbiter nonces are per object and per Root; `nonce <= stored` is `"replay"`.
- The release token, not the booking id, authorizes a slot release. A replayed release with
  the old token returns `"not holder"` after a reclaim.

## The refusal vocabulary

Refusals are short strings, not error codes: `"not owner"`, `"not provider"`, `"not guest"`,
`"closed"`, `"replay"`, `"slot held"`, `"slot lost"`, `"unknown slot"`, `"listing inactive"`,
`"terms"`, `"amount"`, `"too early"`, `"too late"`, `"below threshold"`, `"unknown action"`,
`"payments unavailable"`, `"rcash unavailable"`. The [arm reference](../reference/arms.md)
groups them by contract.

## What authority is not

- It is not the ability to read. Reads are public peeks.
- It is not the ability to pause. Pausing is a 2-of-3 admin action, not an owner action.
- It is not the ability to arbitrate. Arbitration is a 2-of-3 admin action with a nonce.
- It is not an off-chain role. The chain only knows keys, names and signatures.

> **Where to go deeper.** [Governance and arbitration](governance-and-arbitration.md) covers
> the multisig. [Arm reference](../reference/arms.md) lists every refusal. [Operate
> safely](../guides/operate-safely.md) is the key-management checklist.
