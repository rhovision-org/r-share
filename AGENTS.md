# r-share operating notes

Authority order: `contracts/src` and the static gates > `docs/` > this file.

## Platform pitfalls

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
- **r-cash capabilities are resolved per arm**, never captured in a bundle. Only the r-cash
  Root URI string is stored in the adapter.
- **Admin actions call the deployed `AdminGuard`** through the shared
  `lib/AdminGuard.rho` wrapper; owner checks use the inlined `Auth` fragment.
  `lib/Guard.rho` is inlined only in the bootstrap Root, which cannot resolve itself before
  it is registered.

## Commands

```
make spec-check      # build drift, budgets, arch, captures, no-select, money
make metrics         # structural metrics (bytes, cells, arms, resolve sites)
make metrics-compare # compare against contracts/metrics-baseline.json
make parse-check     # render and parse with the target node's parser
make docs            # build docs/book with mdBook
make docs-check      # docs links, orphans, coverage, then mdBook build
make devnet          # r-cash devnet via RCASH_DIR/RHOLO_DIR
make rcash-install   # install r-cash components
make keys install    # install r-share
make unit-all        # implemented behavioural suites
```

## Rules for changes

- A new resource class changes only `kind`, `attrs`, and a pure `Policy` branch. If it needs
  a new booking arm, the primitive is wrong.
- Never add a shared mutable counter or a mutable availability cache.
- Every new arm needs a refusal test and an idempotency test.
- Every value-moving path needs a balance-conservation assertion.

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
