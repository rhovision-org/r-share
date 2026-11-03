# Platform constraints

> **Status:** Implemented. These are hard limits and behaviours of the target rholo node.

r-share is written for rchain-rust (the rholo node). The following constraints shape the
contracts and the build.

## Size and depth

| Constraint | Value | Consequence |
|---|---|---|
| Deploy term cap | 64 KiB inclusive | a term larger than 64 KiB is refused |
| Project hard budget | 56 KiB | leaves 8 KiB headroom for install rendering |
| Parser depth | 128 grammar levels | a deeply nested term is refused before evaluation |
| Contract count budget | 32 distinct names per term | a soft catalogue limit |

`make measure-check` enforces the byte, depth and contract-count budgets.
`make parse-check` enforces the node parser's depth and syntax.

## Syntax the node parser rejects

- `select { ... }` parses but is **not committed choice**; it is desugared to parallel `for`
  receives. r-share forbids it. `make check-select` enforces the ban.
- Unary `!` is not accepted. Write `ok == false` instead of `!ok`.
- A bare-variable send such as `state!s` is not accepted. Write `state!(s)`.
- An unmatched `for` is silence, not an error. Match reply shapes exhaustively.

## Time

- `rho:block:data` returns `(blockNumber, senderBytes, timestampMs)`.
- The timestamp is the proposer's unverified wall clock. No r-share logic reads it.
- There are no timers. Expiry is lazy and needs a caller.
- Deploys are valid for 50 blocks. Anchor `validAfterBlockNumber` at the tip.

## Execution

- A deploy is atomic with rollback. A failed deploy moves nothing.
- Blocks execute deploys sequentially. Deploys do not interleave.
- `explore-deploy` is read-only and has no block context. Anything reading `rho:block:data`
  must be a signed deploy.
- Phlo is charged per deploy; a settlement does several resolves and ERTP calls. Budget phlo
  accordingly.
- `bundle+` is the only bundle mode used for registration. Do not capture a registry-resolved
  capability in a bundle.

## Naming and sends

- `@x` matches or sends data; `*x` sends a name.
- A capability returned in a tuple is called with `@cap!(...)`; a capability bound by a
  top-level `for (Cap <- ch)` is called with `Cap!(...)`.
- A state cell is consumed with `for (@s <- cell)` and peeked with `for (@s <<- cell)`.

## The 64 KiB budget in practice

The largest term is `22_booking_contract.rho` at 23,284 B. The per-file targets are 24 KiB
for `22`, 28 KiB for `14_listing_contract.rho` (the allocator) and 12 KiB for
`23_guest_ledger.rho`; `make measure-check` enforces them alongside the 56 KiB project
budget. The budget leaves room for a fork's additions. If a fork pushes a term over 56 KiB, split the
factory from the instance. r-share keeps the listing factory and the allocator in one term;
the ledger and booking factories are separate deploy files. It splits by deploy file when
depth or size requires it (terms, remedy, bond, booking).

> **Where to go deeper.** [Concurrency doctrine](../concepts/concurrency-doctrine.md) is the
> design response. [Test your fork](../guides/test-your-fork.md) runs the gates. [Contract
> map](contract-map.md) is the term list.
