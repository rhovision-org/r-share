# r-share operating notes

Authority order: `contracts/src` and the static gates > `docs/` > this file.

## Platform pitfalls

- **One allocation cell per listing.** `ListingContract` owns units and slots in one
  `listingState` cell; `acquire` is one take/commit and writes every requested slot or none.
  Do not reintroduce per-slot contracts or publish a slot URI/release token.
- **`select` is not committed choice.** `rholang/src/normalizer.rs:256-274` desugars
  `PChoice` into parallel `for` receives. Never use it for exclusivity. `make check-select`
  fails the build. Use one consumable datum.
- **Deploy term cap is 64 KiB inclusive** (`casper/src/multi_parent_casper.rs:99`). The hard
  budget is 56 KiB per term. Measure after every change.
- **A deploy is atomic, blocks are sequential.** Deploys do not interleave. A multi-slot
  acquire may be sequential as long as every consumed record is reposted before the arm
  replies. Do not describe it as one comm event.
- **Peeks are hints.** Never carry a peeked record across an external await into a write.
  Never use a peek as a lock or reservation.
- **Unmatched `for` is silent.** A wrong arity means a deploy that never replies, not an
  error. Match reply shapes exhaustively.
- **`*x` sends a name; `@x` sends data.** Names (payments, purses, facets, tokens) are
  passed with `*`.
- **After value moves, fail.** Return `(false, reason)` only before a withdrawal/deposit;
  afterwards an unexpected branch calls `1/0` so the deploy rolls back.
- **`rho:block:data` timestamp is the proposer's wall clock**, not a consensus input. Use
  only the block number.
- **No timers.** Expiry is lazy: a later holder may reclaim an expired hold, and
  `expire`/`settle`/`closeEvent` are permissionless.
- **`explore-deploy` has no block context.** Anything reading `rho:block:data` must be a
  signed deploy.
- **Deploy lifespan is 50 blocks.** Anchor `validAfterBlockNumber` at the tip.
- **r-cash capabilities are resolved per arm**, never captured in a bundle. The common
  resolution body is `lib/Resolve.rho`; the source metric counts one copy, but each inlined
  call performs its own Root lookup. Only the r-cash Root URI string is stored in the
  adapter.
- **Admin actions call the deployed `AdminGuard`** through the shared
  `lib/AdminGuard.rho` wrapper; owner checks use the inlined `Auth` fragment.
  `lib/Guard.rho` is inlined only in the bootstrap Root, which cannot resolve itself before
  it is registered.
- **The client key never crosses into JavaScript.** It is created/decrypted in
  `rshare-wasm`; the browser stores only a Web3 v3 keystore in IndexedDB. Do not add a
  plaintext-key path or a second TypeScript signer.
- **Explore vs signed.** `explore-deploy` has no block context and no deployer. `quote`,
  guest ledger/ops, account balance and every write are signed. Do not "fix" a stale quote
  by switching it to explore.
- **Finality.** A processed deploy is not success. Require finalized + processedWithSuccess
  + `(true, …)`. Never retry a value-moving deploy with a new op id unless you proved the
  first had no effect.
- **Client term budget is 56 KiB**, checked by `rshare-core` before signing. The node cap is
  64 KiB. Keep previews under the budget.
- **No indexer.** Discovery is a curated catalogue plus share links. A provider booking
  inbox requires the guest's link. Do not add a fake search or inbox.
- **`setAdmins` payload quirk.** The signed payload uses the comma-joined public-key text
  and threshold; the call uses byte arrays. Build payloads with the deployed
  `Codec("adminPayload")`, not TypeScript string joining.
- **Admin nonce.** Listing/provider/booking expose a nonce; global `ShareAdmin` does not.
  Use height + margin and treat `replay` as a state read, not a retry.

## Commands

```
make spec-check      # build drift, budgets, arch, captures, no-select, money
make metrics         # structural metrics (bytes, cells, arms, resolve sites)
make metrics-compare # compare against contracts/metrics-baseline.json
make parse-check     # render and parse with the target node's parser
make docs            # build docs/book with mdBook
make docs-check      # docs links, orphans, coverage, then mdBook build
make devnet          # r-cash devnet via RCASH_DIR/RHOLO_DIR
make rcash-keys      # generate and faucet the r-cash keys
make rcash-bootstrap # install the r-cash components r-share needs
make keys install    # copy the keys and install r-share
make unit-all        # all 16 behavioural suites
make formal-check    # Lean build, axiom audit, vectors and flow manifest
make check-flows-strict # L0 proof-quality gate; currently 0 warnings
make audit-check     # audit registers, arm inventory and traceability
make audit-dynamic   # expected-failure audit reproductions on a devnet
make client-test     # Rust SDK, core and keystore tests
make client-lint     # fmt and clippy on the client crates
make client-wasm     # wasm32 check
make client-check    # client gates + WASM smoke + web tests/build
make client-fixtures # real provider/listing/unit + catalogue
make client-journey  # CLI journey with positive/negative checks
make client-keystores # encrypted admin1/admin2 profiles for the browser
npm --prefix web run dev / test / test:e2e
```

## Rules for changes

- A new resource class changes only `kind`, `attrs`, and a pure `Policy` branch. If it needs
  a new booking arm, the primitive is wrong.
- Never add a shared mutable counter or a mutable availability cache.
- Every new arm needs a refusal test and an idempotency test.
- Every new public arm must appear in `audit/registers/arms.yaml`; `make audit-check`
  fails on inventory drift.
- Every cornerstone arm names a Lean declaration in `spec/flows/core.yaml`; a new
  mutating arm without one is `owed`, not proved. `make check-flows-strict` rejects a
  `proved-model` target that is a definition or constructor.
- Every value-moving path needs a balance-conservation assertion.
- `reset` may clear only a free slot or an expired hold. A live hold or booked slot is
  refused. Terminal settlement releases the booking's slots before the Bond pays.
- A no-show claim enters `noShowClaimed` and pays nothing. The guest may contest inside the
  `disputeWindowBlocks` window; uncontested claims pay the provider after it, contested
  claims default to a full guest refund through `resolveNoShow`.
- Admin sets require at least two distinct keys and threshold at least 2. Duplicate keys,
  threshold 1 and threshold greater than the key count are refused.
- Direct `acquire` is gated by global pause and provider status. `ListingFactory.create`
  refuses a paused provider.
- `BookingContractFactory.create` requires a listing `verifyHold` reservation and matching
  provider, listing and kind. Do not weaken this for a test; use a test-only fake listing.
- Do not claim an audit finding is fixed unless the change and its regression test are
  in the tree and recorded in `audit/registers/findings.yaml`.

## Documentation

- `docs/` is the mdBook source; `docs/book/` is generated and gitignored. `book.toml` sets
  `src = "docs"` and `build-dir = "docs/book"`.
- Stock mdBook only: no mermaid, KaTeX, toc or other preprocessors. Use ASCII diagrams.
- Every `docs/**/*.md` chapter except `SUMMARY.md` must be listed in `SUMMARY.md` exactly
  once. `make docs-check` enforces links, orphans, includes, README links and reference
  coverage.
- `reference/` is canonical for arms, schemas, constraints and install order. Update
  `reference/contract-map.md` and `reference/install-order-and-keys.md` whenever a
  component, deploy file or component key changes; the checker enforces it.
- Use a status label near the top of every concept and case-study chapter. "Implemented"
  means source plus static/parser gates, never behavioural verification.
- Link between chapters with relative `.md` links. Link to source code with GitHub URLs or
  plain backticked paths. Never link to generated `docs/book/` paths.
