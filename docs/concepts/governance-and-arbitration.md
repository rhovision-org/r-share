# Governance and arbitration

> **Status:** Implemented. **Applies to:** Root, admin, listing, unit and booking actions.

r-share ships a deliberately small governance surface: a 2-of-3 multisig that can pause,
delist, rotate and arbitrate. It is not a full governor, and it is not a single-key admin.

## The admin set

- The Root and ShareAdmin contracts store three public keys and a threshold of 2.
- `AdminGuard.verify` checks `rshare:admin:v1|<domain>|<nonce>|<action>|<argsHash>` with
  BLAKE2b-256 and secp256k1, counting distinct signatures from the stored keys.
- Each object stores the admin set and threshold captured at creation. A listing, unit or
  booking can rotate its own set with `setAdmins`, signed by the current threshold.
- Nonces are per object; `nonce <= stored` is `"replay"`.

## What the admin set can do

| Object | Actions |
|---|---|
| Root | `rotate`, `deprecate`, `freeze`, `setAdmins`, `setThreshold` |
| ShareAdmin | `pauseAll`, `resumeAll`, `setAdmins`, `setThreshold` |
| ProviderContract | `adminPause`, `setAdmins` |
| ListingContract | `adminPause` (`pause`, `delist`), `setAdmins` |
| ListingContract | `pauseUnit`, `resumeUnit`, `adminPause`, `setAdmins` |
| BookingContract | `arbitrate`, `setAdmins` |
| Payments | `setPlatformOwner`, `setAssets`, `setRcashRoot` |

The admin set cannot move a booking's money outside the settlement arms. It can pause new
bookings, delist a listing, or split a deposit through arbitration. It cannot withdraw rent
or skip the bucket-conservation check.

## Pause and delist

- `ShareAdmin.pauseAll` blocks new bookings at the GuestLedger gate. Existing bookings can
  still cancel, check out and settle.
- A listing's `pause` or `delist` stops new bookings for that listing. Existing bookings are
  unaffected.
- A provider `pause` stops new listings and units from that provider.
- A unit `adminPause` is available for a compromised or unsafe unit.

Pause is not revocation. A compromised admin key set can still be used until the set is
rotated; rotation is per object and must be done deliberately.

## Arbitration

A damage claim is a deposit split:

1. The provider calls `claimDeposit` with an amount up to the deposit and an evidence hash.
2. The guest can `dispute`.
3. The 2-of-3 arbiter calls `arbitrate(nonce, splitBps, sigs, opId)`.
4. `Remedy.depositSplit` computes `providerDamage = floor(deposit * splitBps / 10000)` and
   the guest refund; the settlement pays rent and fee normally.

The arbiter decides the split, not the facts. Evidence hashes are opaque; the off-chain
dispute process owns inspection, repair quotes, insurance and communication.

## Root lifecycle

- The installer registers components before `freeze`.
- `rotate` updates a component URI with an expected version; a stale version is refused.
- `deprecate` marks an entry retired; `resolve` refuses it.
- `freeze` blocks new registrations; rotation and deprecation remain available.
- `admin` changes the admin set or threshold with a 2-of-3 signature.

## What is not governed on-chain

- The truth of a damage claim.
- Insurance, licensing or safety certification.
- The platform's commercial terms.
- A fork's brand, support or refunds beyond the policy.

> **Where to go deeper.** [Authority and refusal](authority-and-refusal.md) is the caller
> model. [Bookings and lifecycle](bookings-and-lifecycle.md) shows arbitration's place.
> [Arm reference](../reference/arms.md) lists the signatures.
