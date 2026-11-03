# Testing

## Static gates (no chain)

```
make spec-check     # build drift, 64 KiB budgets, catalogue, captures, no-select, money
make parse-check    # renders install terms and parses with the target node's parser
```

`parse-check` builds a small crate against `RCHAIN_RUST_DIR` (default `../RNodeRust`) and
uses `rchain_rholang::parser::parse`. The node parser enforces a depth limit of 128 and
rejects unary `!`, bare-variable sends (`state!s` must be `state!(s)`), and `select`-like
constructs. Render output is written to `.devnet/rendered/`.

## Behavioural suites (r-cash devnet)

```
make devnet rcash-install keys install
make unit-all
```

| Suite | Status | Covers |
|---|---|---|
| `unit_root` | implemented | resolve, keys, unknown/stale |
| `unit_payments` | implemented | r-cash health, platform owner, REV issuer |
| `unit_provider` | planned (M2) | registration, ownership refusals |
| `unit_listing` | planned (M2) | terms, units, slot schedule |
| `unit_booking` | planned (M3) | instant/request, escrow, opId |
| `unit_concurrency` | planned (M3) | same-block races |
| `unit_lifecycle` | planned (M4) | accept/reject/cancel/checkout/settle |
| `unit_events` | planned (M5) | redeem, transfer, close |
| `unit_admin` | planned (M6) | 2-of-3 pause/delist/arbitrate |
| `unit_generality` | planned (M7) | bike/car/robot fixtures |

Planned suites exit 2 with the milestone they belong to. They are not reported as passing.

## Race tests (release gate)

Use two deploys submitted before a single propose, so both land in the same block:

1. **One slot, two guests** → exactly one `(true, ...)`, one `(false, "slot held", id)`,
   one debit.
2. **Disjoint slots** → both succeed, two debits.
3. **Overlapping multi-slot** → A books [1,2], B books [2,3]; exactly one wins; the loser
   holds none of its slots.
4. **Accept vs cancel** → exactly one terminal transition; the other returns `"closed"`;
   funds match the winner.
5. **Expiry race** → A's hold expires, B books the slot, A.expire refunds A and A.release
   returns `"not holder"`; B's slot is intact.

Every suite also checks conservation: for each asset,
`Σ account balances + Σ escrow purses` is constant, and every slot is exactly one of
free/held/booked.
