# Trust assumptions

**Status:** Analysis. The list is the allowed trust set for the Pareto comparison.

## Assumptions

- **Root/installer bootstrap.** The installer registers the initial components and the
  2-of-3 admin set. A compromised installer can install a malicious component before
  `freeze`. This is a bootstrap assumption, not a per-booking one.
- **2-of-3 admin/arbiter.** The admin set can pause, delist, rotate and arbitrate within the
  class bounds. It cannot move money outside the remedy table or skip settlement. A
  compromised set can pause the platform and bias a bounded split.
- **r-cash rail.** Accounts, ERTP payments, deposit facets and the asset resolver are the
  payment rail. A compromised or unavailable rail can refuse settlement; the booking escrow
  remains in its Bond.

## Not trust assumptions

- **Provider bond.** Collateral, not a party. A forfeited bond pays the buyer; it does not
  vote or attest.
- **Evidence hash.** Existence at a height, not truth. The arbiter decides a split; the
  content is off-chain.
- **Deterministic remedy.** A pure function of on-chain inputs; no discretion.
- **Chain consensus.** Out of scope for this comparison; every on-chain system assumes it.

## Explicitly excluded

- On-chain oracles and attestors of quality, weather, telemetry or metering.
- KYC, legal identity, insurance and licensing.
- A live-name registry or any permissionless capability store.
- A trusted wall clock; only block heights are used.

## Phase 3/4/5 bar

A phase may add a trust assumption only with a spec change to this file and a new Pareto row.
The default is not to add one. Evidence anchors are hashes, not attestors; provider bonds are
collateral, not parties; metering is a bounded policy extension, not an oracle.
