# Migration and versioning

The repository is devnet-only, but breaking changes should still be staged as if mainnet
discipline applied. The audit does not apply any of these changes; this is the plan that
accompanies the Rholang backlog.

## Version markers

- Add `schemaVersion` to each stateful cell: `rootState`, `adminState`, `paymentsState`,
  `providerBookState`, `providerState`, `listingState`, `bondState`, `guestBookState`,
  `ledgerState`, `bookingState`.
- Add `abiVersion` to each component's `info` projection.
- Add a `spec/versions.yaml` mapping component key, deployed version, schema version and
  compatibility window.

## Coexistence

1. Freeze `audit-baseline-<commit>` as v1.
2. Deploy v2 under new Root component versions or new keys. Do not silently reuse a v1 key
   with changed semantics.
3. Support v1 and v2 reads during the compatibility window. Clients read `abiVersion` and
   select the matching arm shape.
4. Run the conformance corpus and a devnet soak against both versions.
5. Switch the Root pointer to v2.
6. Deprecate v1 after one full devnet cycle; never delete v1 state before the cycle ends.

## Migration

- Prefer an idempotent, admin-gated `migrate` arm for stateful components where the state
  shape can be copied without ambiguity.
- For devnet, a clean re-seed is acceptable only as a documented, tested procedure. It is
  not an implicit reset and must not be described as a migration.
- Record the migration input hash, output hash and the exact commit in
  `audit/evidence/<date>-<commit>/`.

## Rollback

- Before state mutation: switch the Root pointer back to v1.
- After mutation: use a documented reverse migration or a snapshot restore. A Root pointer
  switch alone is not a rollback once v2 has written v2 state.
- Keep the v1 and v2 manifests and the migration transcript with the release.

## Change boundaries

- P0 changes are separate change requests from the audit.
- Docs corrections can ship independently, but must not claim a contract fix has landed.
- A change that alters the meaning of `delisted`, `redeemed`, `setAdmins` or `reset` is an
  ABI/state change even if the arm name is unchanged.
