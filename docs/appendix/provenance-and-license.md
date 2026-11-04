# Provenance and license

> **Status:** Legal reference. This is not legal advice.

r-share is licensed **AGPL-3.0-or-later**. The book and the code carry the same license.

## What r-share is built on

- **r-cash** — https://github.com/rhovision-org/r-cash, AGPL-3.0. Reference checkout
  `~/Rholphin`, pinned at commit `49b441d` (v0.5, devnet-only). r-share reuses its
  architectural patterns: the Root directory, deployer-derived owner ids, the account gate,
  the per-object escrow cell, the M-of-N guard, and the build/harness structure.
- **rchain-rust** — https://github.com/rchain-community/rchain-rust, AGPL-3.0. The rholo
  node. r-share targets its parser, ERTP, registry and system channels.

No reference source file is copied verbatim into r-share. The notices and the exact files
referenced are in `THIRD_PARTY_NOTICES.md`.

## The AGPL obligation

The AGPL is a strong copyleft license with a network clause. If you run a modified version
as a network service, §13 requires you to offer the corresponding source to your users.
"Corresponding source" includes the modified contracts and the build tooling, not your
private customer data.

Practical rules for a fork:

- keep `LICENSE` and `THIRD_PARTY_NOTICES.md`;
- publish your modified source where your users can get it;
- do not relicense the r-share code;
- do not imply endorsement by r-share, r-cash or rchain-rust;
- use a distinct product name and brand.

## What is not covered

- Your off-chain product: UI, backend, database, telemetry, support, marketing.
- Your data: bookings, addresses, identity, telemetry.
- Your regulated obligations: licensing, insurance, consumer terms, data protection,
  KYC/AML, vehicle and robot safety.
- A warranty. The software is devnet-only, unaudited, and provided as-is.

## Status of the references

r-cash v0.5 is a devnet-only prototype and may change. r-share pins `49b441d` and
version-gates every resolve. A fork that moves to a different r-cash commit must re-run the
compatibility checks and update the pin.

## How to cite r-share

If you write about the primitives, cite the repository and the book:

```
r-share: booking and rental primitives for rholo.
https://github.com/rhovision-org/r-share
```

If you build a product, say that it is built on r-share and that you are responsible for
your own operation. Do not present a fork as an official r-share service.

> **Where to go deeper.** [Operate safely](../guides/operate-safely.md) is the operator
> checklist. [Fork and run](../guides/fork-and-run.md) is the setup. [Contributing and
> community](contributing-and-community.md) is the community path.
