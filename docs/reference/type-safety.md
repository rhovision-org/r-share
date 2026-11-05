# Type safety and client gates

> **Status:** Implemented. The client builds with panic-free library lints, strict
> projection decoding and typed chain scalars. The integration campaign is acceptance,
> not the discovery mechanism.

## Why this exists

A contract can be correct and the client still unsafe. The admin browser journey failed
because the global nonce had no read arm and every client guessed one from block height or
wall-clock time. The type system did not encode "this value must be greater than the chain's
stored nonce", so only an integration test could catch the guess.

The client now validates at three boundaries:

1. **Construction.** `Amount`, `Height`, `Nonce`, `BasisPoints`, `BlockCount` and `OwnerId`
   in `crates/rshare-core/src/primitives.rs` reject negative, out-of-range and malformed
   values. Domain code cannot hold an unchecked raw `i64`.
2. **Decode.** Projection decoders reject a missing or wrong-typed required field with a
   path-aware `DecodeError`. Genuinely optional fields are `Option<T>`; no decoder invents
   `0`, `""`, an empty map or `false`.
3. **JSON/WASM.** `Amount`, `Height` and `Nonce` cross into TypeScript as decimal strings
   and become `bigint`. `BasisPoints` and contract-bounded counts stay numbers and are
   range-checked. The TypeScript decoders accept a safe JSON integer during the transition,
   reject values above `Number.MAX_SAFE_INTEGER`, and never coerce through `Number`.

## Nonce rule

Every admin domain exposes its current nonce: `ShareAdmin("nonce")`, `ShareRoot("nonce")`,
and the `nonce` field of listing, provider and booking `info`. A client reads the current
value and signs `current + 1`. It never derives a nonce from height or wall-clock time.

Two clients can read the same `N`; the chain accepts one `N+1`. Signatures bind the nonce,
so a replay is a conflict: re-read, invalidate the pending action, rebuild and re-sign.

## Gates

`make client-gates` runs without a devnet:

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy -p rshare-wasm --target wasm32-unknown-unknown -- -D warnings
cargo test --workspace
npm --prefix web run typecheck
npm --prefix web run typecheck:e2e
npm --prefix web run lint
npm --prefix web test
npm --prefix web run build
```

The workspace denies `unsafe_code`, `unwrap_used`, `expect_used`, `panic`,
`unreachable`, `todo` and `unimplemented` in library code. Tests may use them through
`clippy.toml`. ESLint runs type-aware rules over `src` and `e2e`; the safety rules are
warnings during the refactor and become errors before the acceptance campaign.

## Not claimed

- The decoders validate shape and ranges, not chain-level truth.
- The nonce conflict is surfaced, not eliminated; two parties can still race.
- `make client-gates` is not a substitute for the behavioural campaign; it is the
  prerequisite that stops the campaign from being the place where type errors are found.
