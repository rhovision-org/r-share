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
| Contract count budget | 36 distinct names per term | a soft catalogue limit |

`make measure-check` enforces the byte, depth and contract-count budgets.
`make parse-check` enforces the node parser's depth and syntax.

## Syntax the node parser rejects

- `select { ... }` parses but is **not committed choice**; it is desugared to parallel `for`
  receives. r-share forbids it. `make check-select` enforces the ban.
- Unary `!` is not accepted. Write `ok == false` instead of `!ok`.
- A bare-variable send such as `state!s` is not accepted. Write `state!(s)`.
- An unmatched `for` is silence, not an error. Match reply shapes exhaustively.

## `match` semantics

- `if`/`else` is sugar for `match` on booleans. A refactor between them is neutral only while
  every case remains total.
- An unmatched `match` is **silence**, not an error. Every match over a cell or reply value
  must end in `_`, `false`, or `Nil` that reposts the cell and replies.
- First matching case wins in written source order. Preserve refusal order.
- A pattern that binds the same variable twice never matches.
- `map.get(k)` returns `Nil` for an absent key; `Nil` and a stored `Nil` are indistinguishable.
- `list.nth(i)` out of bounds aborts the deploy. Keep the `i < length` guard.
- There are no match guards and no or-patterns. Range, membership and `||` checks stay
  `if/else` or a flat `else if` chain.
- A single-arm `match e { x => ... }` is the idiomatic `let`; do not rewrite it.
- List patterns have static arity; `["held", ...]` variants need separate cases.
- `<-` consumes and `<<-` peeks. Do not move that boundary during a match refactor.

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

The largest term is `34_booking_instance.rho` at about 34.4 KiB after the instance split; the
thin `22_booking_contract.rho` is about 1.7 KiB. The per-file targets are 28 KiB for
`14_listing_contract.rho` (the allocator; now 28,554 bytes, 118 free) and 14 KiB for
`23_guest_ledger.rho` (now 14,274 bytes, 62 free), whose extra 2 KiB covers the
`bookWithCover` dispatcher and the hold-deposit path; `make measure-check` enforces them
alongside the 56 KiB project budget. `41_hold_deposit.rho` is a separate term at about
6.7 KiB, so the hold custody does not consume the listing budget. The 22 KiB raise covers
the no-show contest arms, the factory origin check and the shared admin validation; if the
term passes 30 KiB, split the factory rather than raise the cap again. The budget leaves
room for a fork's additions. If a fork pushes a term over 56 KiB, split the
factory from the instance. r-share keeps the listing factory and the allocator in one term;
the ledger and booking factories are separate deploy files. It splits by deploy file when
depth or size requires it (terms, remedy, bond, booking, hold deposit).

> **Where to go deeper.** [Concurrency doctrine](../concepts/concurrency-doctrine.md) is the
> design response. [Test your fork](../guides/test-your-fork.md) runs the gates. [Contract
> map](contract-map.md) is the term list.

## Client constraints

The Rust client shares the node's limits and adds a few rules of its own.

| Constraint | Value | Consequence |
|---|---|---|
| Client term budget | 56 KiB | `rshare-core` refuses a larger term before signing |
| Default phlo | `4_000_000` at price `1` | a book deploy resolves several components |
| Deploy lifespan | 50 blocks | an inclusion window, **not** a booking hold |
| Read split | explore for peeks; signed for `Clock` and `deployerId` | a free quote is impossible |
| Success | finalized + processed + `(true, …)` | a processed deploy is not success |

The 50-block deploy lifespan anchors whether a deploy can be included. Booking holds,
check-in, review and dispute deadlines are contract heights and have nothing to do with the
deploy lifespan. Do not build UI timers on 50 blocks.

The client has one node and discovery disabled. There is no hostlist or pool UI. Funding
uses the devnet faucet and the ERTP `revFund` bridge; neither is a production deposit rail.

The catalogue is curated and share links are first-class. There is no indexer, provider
inbox or notification stream; disabled screens say so.
