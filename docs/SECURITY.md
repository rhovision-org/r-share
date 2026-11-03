# Security notes

## Authority

- `providerId`, `guestId` and every owner id are REV addresses derived from the deploy key
  with `rho:rchain:deployerId:ops` + `rho:rev:address`. No address is accepted as a caller
  argument.
- Owner arms call `authIsOwner(dId, ownerId)` and refuse `"not owner"`.
- Admin and arbiter actions use 2-of-3 secp256k1 over
  `rshare:admin:v1|<domain>|<nonce>|<action>|<args>`, with a nonce stored in the same cell
  as the effect.
- The BookingContract's release token is a fresh unforgeable name created in the booking
  deploy. A slot releases only for the matching token; an expired hold reclaimed by another
  booking cannot be released by the old booking.

## Refusal ordering

- Refuse with `(false, reason)` **before** moving value.
- After any ERTP withdraw/deposit, an unexpected branch calls `1/0` so the deploy rolls back.
- Every refusal reposts the state cell; no arm leaves a cell empty.

## Replay and idempotency

- GuestLedger reserves `opId` before any external call and records the result afterward.
  A replay returns the stored result and moves no money; a pending replay returns `"pending"`.
- BookingContract records a successful mutating arm by `opId` and returns it on replay.
- Admin/arbiter nonces are per object and per Root; `nonce <= stored` is `"replay"`.

## Known limits

- **Hold squatting.** A free hold requires no payment. Anyone can hold a slot until
  `maxHoldBlocks` (default 20). Bounded by phlo, the provider `reset` arm, and the fact that
  a confirmed booking requires escrow. A non-refundable hold bond is a later option.
- **Expiry liveness.** Rholang has no timers. `expire`, `settle` and `closeEvent` are
  permissionless; guests are refund-incentivized. A never-called booking leaves escrow in
  its purse.
- **Block timestamp.** It is the proposer's unverified wall clock. No r-share logic reads it.
- **Public data.** Bookings, dates, prices and addresses are on-chain. There is no privacy.
- **r-cash v0.5 is devnet-only and unaudited.** Pin `49b441d` and version-gate every resolve.
- **No arbitration of facts.** The 2-of-3 arbiter decides a deposit split; it does not
  adjudicate insurance, damage evidence or regulatory disputes.
