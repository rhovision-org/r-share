# Contributing and community

> **Status:** Community guide. The project has no code of conduct or security-disclosure
> policy yet; this chapter is explicit about that gap.

r-share is a case study as much as a codebase. Contributions are welcome when they make the
primitives clearer, safer or more reusable.

## Ways to contribute

- **A case study.** Map a new use case onto the five primitives. A good case study names the
  unit, the slot, the terms, the shared arms, the off-chain boundary and the status.
- **A fixture or suite.** Add a behavioural case for a resource class, a race, or a failure
  path. The race and conservation gates are the most valuable.
- **A primitive fix.** Concurrency, escrow, authority and policy changes need a design note
  and tests.
- **A documentation fix.** Broken links, unclear steps, a missing failure path. Run
  `make docs-check`.
- **A fork story.** Tell the community what you built and what the primitives did not cover.

## Before you open a change

1. Read `AGENTS.md` and the [concurrency doctrine](../concepts/concurrency-doctrine.md).
2. Run `make spec-check`, `make parse-check` and `make docs-check`.
3. If you changed a contract, run `make build` and commit `contracts/build/` and
   `contracts/measure.json`.
4. If you changed a component, deploy file or component key, update the
   [contract map](../reference/contract-map.md) and
   [install order and component keys](../reference/install-order-and-keys.md); the docs
   checker enforces it.
5. If you added a suite, update the suite table in
   [Test your fork](../guides/test-your-fork.md) and the suites under `harness/`.
6. Label the status honestly. A new class without a suite is "mapping only".

## The change rules

- A new resource class changes only `kind`, `attrs` and a pure `Remedy` branch. If it needs a
  booking arm, the primitive is wrong.
- Never add a shared mutable counter or a mutable availability cache.
- Every new arm needs a refusal test and an idempotency test.
- Every value-moving path needs a conservation assertion.
- No `select`, no unary `!`, no bare-variable sends.
- Keep the deploy term under 56 KiB.

## What the community owns

- The primitive layer and its tests.
- The book and the case studies.
- The reference coverage that keeps the book honest.

## What a fork owns

- The product, the UI, the backend and the data.
- Identity, calendars, telemetry, support and insurance.
- Licensing, consumer terms, data protection, KYC/AML and safety.
- Its own brand and its own promises.

## Governance gaps

- There is no code of conduct yet.
- There is no security-disclosure policy yet.
- There is no maintainer council or formal RFC process yet.
- The 2-of-3 admin set is the on-chain governance; the repository governance is informal.

If you need any of these to contribute, open an issue and say so. A community that names its
gaps is more trustworthy than one that hides them.

## Reporting a security issue

There is no private disclosure channel defined. Do not publish an exploit before a fix. Open
a minimal public issue asking for a private channel, or contact the maintainer through the
repository profile. This is a gap, not a policy.

## License

Contributions are under AGPL-3.0-or-later. By contributing, you agree to license your change
under the same terms and to keep the provenance notices.

> **Where to go deeper.** [Provenance and license](provenance-and-license.md) is the license.
> [Fork and run](../guides/fork-and-run.md) is the setup. [Roadmap](roadmap.md) is the plan.
