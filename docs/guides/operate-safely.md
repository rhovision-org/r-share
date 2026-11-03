# Operate safely

> **Status:** Operational guidance. r-share is devnet-only and unaudited.

A fork that operates this software is operating a financial and sometimes safety-critical
service. This chapter is the minimum checklist; it is not legal, financial or safety advice.

## Keys

- The deploy key is the owner. Anyone with it can act as the provider or guest.
- Do not reuse the devnet deployer key in production.
- Keep admin keys offline or in a hardware-backed signer.
- The 2-of-3 admin set controls pause, delist, rotation and arbitration. Do not let one
  person hold two of the three keys.
- Rotate admin keys per object with `setAdmins`; rotation is not retroactive for objects
  created with the old set.

## The admin set

- Verify the threshold at install: `ShareAdmin("keys")` returns `(true, admins, threshold)`.
- Test a 2-of-3 action on a devnet before trusting it.
- Keep an offline record of the nonce per object; a replayed nonce is refused.
- A compromised key set can pause or arbitrate until it is rotated. Plan the response.

## Pause and delist

- Global pause blocks new bookings at the GuestLedger gate. Existing bookings can still
  cancel, check out and settle.
- Listing pause/delist blocks new bookings for that listing. Existing bookings are
  unaffected.
- Provider pause stops new listings and units from that provider.
- A unit pause is available for a compromised or unsafe unit.
- Pause is not revocation. Rotate the key set if a key is compromised.

## Payments

- Confirm `Payments.health` resolves the r-cash account factory, asset resolver and clock.
- Confirm the platform account exists; the installer ensures it.
- The asset allowlist starts at `REV`. Verify any other issuer before enabling it.
- A settlement to a missing account fails; do not delist a provider with an open escrow.

## Known limits

- **Hold squatting.** A free hold requires no payment. Anyone can hold a slot until
  `maxHoldBlocks`. Bounded by phlo, the provider reset arm, and the fact that a confirmed
  booking requires escrow. A non-refundable hold bond is a later option.
- **Expiry liveness.** No timers. `expire`, `settle` and `closeEvent` are permissionless;
  guests are refund-incentivized. A never-called booking leaves escrow in its purse.
- **Block timestamp.** The proposer's unverified wall clock. No r-share logic reads it.
- **Public data.** Bookings, dates, prices and addresses are on-chain. There is no privacy.
- **r-cash v0.5** is devnet-only and unaudited. Pin `49b441d`, version-gate every resolve.
- **No arbitration of facts.** The 2-of-3 arbiter decides a deposit split; it does not
  adjudicate insurance, damage evidence or regulatory disputes.

## Before you operate

- [ ] Local counsel has reviewed the rental, consumer and data-protection terms.
- [ ] The AGPL source-offer obligation is understood and satisfied for your deployment.
- [ ] Insurance covers the inventory and the activity.
- [ ] Consumer cancellation and refund terms match the on-chain policy.
- [ ] If fiat is involved, KYC/AML and payment-provider obligations are met.
- [ ] For vehicles or robots, a safety case, operator certification and a remote-stop
      process exist.
- [ ] An incident runbook covers pause, delist, rotation and arbitration.
- [ ] A support process exists for bookings that stall or are disputed.
- [ ] You have tested a full booking and a full failure path on a devnet.

## AGPL

r-share is AGPL-3.0-or-later. Running a modified version as a network service triggers the
§13 source-offer obligation. Keep `LICENSE` and `THIRD_PARTY_NOTICES.md`; publish the source
of your modified version to your users. Do not imply endorsement by r-share, r-cash or
rchain-rust.

> **Where to go deeper.** [Governance and arbitration](../concepts/governance-and-arbitration.md)
> is the multisig. [Provenance and license](../appendix/provenance-and-license.md) is the
> license. [Status, scope and honest limits](../orientation/status-and-scope.md) is the
> evidence.
