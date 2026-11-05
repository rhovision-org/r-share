# Fork and run

> **Status:** Implemented. The static and book paths work without a chain; the devnet path
> needs Docker and sibling checkouts.

This guide takes a fresh clone to a passing static check, a built book, and (optionally) a
local r-cash devnet with r-share installed.

## Prerequisites

- Python 3.12 (the tools and harness are stdlib-only).
- mdBook 0.5.4 or newer for the book.
- A Rust toolchain for `make parse-check` (it builds the target node's parser).
- For the devnet path: Docker, plus sibling checkouts of
  [r-cash](https://github.com/rhovision-org/r-cash) at `49b441d` and
  [rchain-rust](https://github.com/rchain-community/rchain-rust) as `../RNodeRust`.

## Clone and check the static surface

```sh
git clone git@github.com:rhovision-org/r-share.git
cd r-share
make venv
make spec-check
```

`make spec-check` runs the build drift check, the deploy-size and depth budgets, the
component catalogue, the capture lint, the no-`select` lint, the placeholder check and the
integer-money check. It needs no chain.

## Render and parse

```sh
make parse-check
```

This renders the install terms with placeholder admins, an r-cash root and a platform owner,
then parses every built and rendered term with the target node's own parser. It builds a
small parser crate under `.devnet/parse-check`; set `RCHAIN_RUST_DIR` if the node checkout is
not `../RNodeRust`.

## Build the book

```sh
make docs
make docs-check
```

`make docs` builds `docs/book/`. `make docs-check` runs `tools/check_docs.py` first: it
checks SUMMARY coverage, relative links, includes, README links, non-empty chapters and
reference coverage, then builds the book. No plugins are required.

## Optional devnet

```sh
make devnet-up       # r-cash devnet on the rholo node, then wait for health
make rcash-keys      # generate and faucet the r-cash keys
make rcash-bootstrap # install the r-cash components r-share needs
make keys install    # copy the keys and install r-share
make unit-all        # all 27 behavioural suites
```

`make devnet` delegates to r-cash's Makefile with `RHOLO_DIR` pointing at the node checkout.
Override `RCASH_DIR` and `RHOLO_DIR` if your layout differs. The installer writes
`deployments/devnet.json`; the devnet keys live in `.devnet/` and are gitignored.

All 27 suites are implemented. `make unit-all` runs them and writes
`deployments/suite-results.json`; an unhealthy devnet reports `not run`, never a pass.

## Fork checklist

- [ ] Keep `LICENSE` and `THIRD_PARTY_NOTICES.md`.
- [ ] Change `book.toml`'s `git-repository-url` and `edit-url-template` to your repo.
- [ ] Rename the product; do not imply endorsement by r-share, r-cash or rchain-rust.
- [ ] Change `kind`, `attrs`, slot IDs, terms and a pure policy branch for your inventory.
- [ ] Do not add a booking arm; if you think you need one, read
      [Adding a new resource class](../case-studies/new-resource-class.md) first.
- [ ] Add a fixture, a case-study chapter and, if you add a class, a suite.
- [ ] Keep the status labels honest: devnet-only, unaudited, not production-ready.
- [ ] Own the off-chain parts: UI, identity, calendar, telemetry, support, insurance,
      regulation.

## Common problems

| Symptom | Fix |
|---|---|
| `mdbook: command not found` | install mdBook 0.5.4+ (`cargo install mdbook --locked`) |
| `parse-check` cannot find `protoc` | install `protobuf-compiler` |
| parse crate build is slow | it is a one-time build under `.devnet/parse-check` |
| devnet refuses work | the node is degraded; restart the devnet or use a fresh one |
| `install` cannot find the r-cash manifest | run `make rcash-bootstrap` first |
| keys missing | run `make rcash-keys`, then `make keys` |

> **Where to go deeper.** [Test your fork](test-your-fork.md) is the gate. [Create a listing
> and book it](create-a-listing-and-book-it.md) is the first product path. [Provenance and
> license](../appendix/provenance-and-license.md) covers the AGPL.
