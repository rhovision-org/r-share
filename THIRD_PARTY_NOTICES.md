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

r-share writes its own contracts.

The Rust client reuses r-cash packages rather than copying protocol logic:

- `rcash-core`, `rcash-chain` and `rcash-sdk` are git dependencies pinned to r-cash commit
  `49b441dea4f5561992387ed670ab263fcd5dd164` (AGPL-3.0-only). They provide amounts,
  secp256k1 deploy signing, `DeployData` encoding, the rholo HTTP client, finality polling
  and `RhoValue` decoding.
- `crates/rshare-keystore` is a port of `crates/rcash-wasm/src/keystore.rs` at the same
  commit (AGPL-3.0-only). A committed cross-decrypt fixture proves compatibility with a
  keystore written by r-cash's own code. The port keeps the r-cash terms.
- `rcash-chain` itself depends on rholo `rchain-crypto`/`rchain-shared` at rholo commit
  `841cb842876369deaa63e3b3e26c20d60e6c74c6` (AGPL-3.0-only).

The web app is original code; its dependencies (React, Vite, TanStack Query, Zustand,
React Router, Playwright, Vitest) keep their own licenses.

## rchain-rust — https://github.com/rchain-community/rchain-rust

Local reference checkout: `/home/patrick/RNodeRust`. The rholo node r-share targets.

Platform facts referenced:

- `rholang/src/normalizer.rs:256-274` — `PChoice` desugars to parallel receives; `select`
  is not committed choice.
- `rholang/src/system_processes.rs` — `rho:block:data`, ERTP, revVault, deployerId ops,
  registry, secp256k1 and blake2b system channels.
- `casper/src/multi_parent_casper.rs:99` — `MAX_DEPLOY_TERM_BYTES = 64 * 1024`.
- `casper/src/genesis/resources/ERTP.rho` — receive-only deposit facets.
