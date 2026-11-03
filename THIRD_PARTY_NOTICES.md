# Third-party notices

r-share is licensed AGPL-3.0-or-later. It is designed against, and borrows architectural
patterns from, two AGPL-3.0 projects:

## r-cash — https://github.com/rhovision-org/r-cash

Local reference checkout: `/home/patrick/Rholphin`, commit `49b441d` (v0.5, devnet-only).

Patterns referenced:

- `contracts/src/deploy/v3/00_root.rho` — component directory, installer gate, freeze.
- `contracts/src/deploy/v3/09_identity.rho` — deployerId-derived owner ids, account gate,
  deposit facets, per-owner op reservation.
- `contracts/src/deploy/v3/40_transfer.rho` — per-object escrow cell and exactly-once
  terminal states.
- `contracts/src/deploy/v3/04_governor.rho`, `03_admin_guard.rho`,
  `contracts/src/lib/Guard.rho` — M-of-N secp256k1 verification over a canonical payload.
- `contracts/src/listing/ListingRegistry.rho` — staged listing lifecycle.
- `tools/build_contracts.py`, `harness/unit_v3.py`, `tools/deploy.py` — build inliner and
  behavioural harness structure.

r-share writes its own contracts. No r-cash source file is copied verbatim.

## rchain-rust — https://github.com/rchain-community/rchain-rust

Local reference checkout: `/home/patrick/RNodeRust`. The rholo node r-share targets.

Platform facts referenced:

- `rholang/src/normalizer.rs:256-274` — `PChoice` desugars to parallel receives; `select`
  is not committed choice.
- `rholang/src/system_processes.rs` — `rho:block:data`, ERTP, revVault, deployerId ops,
  registry, secp256k1 and blake2b system channels.
- `casper/src/multi_parent_casper.rs:99` — `MAX_DEPLOY_TERM_BYTES = 64 * 1024`.
- `casper/src/genesis/resources/ERTP.rho` — receive-only deposit facets.
