# Bond arms

> **Status:** Phase 1 is implemented: one buyer leg (rent + deposit), `pay`, `balance`,
> `info`. The provider leg and `fund` are Phase 2/4 proposals.

## BondFactory — `15_bond.rho`

```
BondFactory("create", escrowPurse, brand, @assetId, @expected, settleToken, ret)
  -> (true, bondUri)
BondFactory("createFromAccount", dId, @assetId, @expected, settleToken, ret)
  -> (true, bondUri) | (false, withdraw reason)
```

The factory receives the funded ERTP purse, its brand, the asset id, the expected total and a
fresh settle token. It spawns one Bond and returns its URI.

`createFromAccount` is the path the guest ledger uses: the factory resolves Payments,
withdraws `expected`, checks the amount, funds the purse and creates the Bond. A failure
before the withdraw is returned as `(false, reason)`; a failure after it calls `1/0` so the
deploy rolls back.

## Bond — one per booking

```
Bond("pay", @buckets, @expected, settleToken, ret)
  -> (true, Nil) | (false, "closed" | "not holder" | "amount")
Bond("balance", ret) -> (true, amount)
Bond("info", ret) -> (true, {paid, expected, remaining})
```

- `pay` consumes `bondState`, checks the settle token and `expected`, checks every bucket is
  non-negative and sums to `expected`, resolves Payments once, withdraws and deposits each
  bucket, then posts `{paid: true, remaining: 0}`.
- A second `pay` is `"closed"`. A wrong token is `"not holder"`. A sum mismatch is `"amount"`.
- Any refusal after a withdrawal calls `1/0`; the deploy rolls back the whole payment.

## State

```
{"paid": Bool, "expected": Int, "remaining": Int}
```

The purse, brand, asset id and settle token live in the Bond closure, not in the state cell.

## Conservation

- `Σ buckets == expected`.
- `expected == rent + deposit` in Phase 1; `rent + deposit + providerBond` in Phase 2.
- After `pay`, the purse is empty and `paid` is true.
- The BookingContract asserts `expected` before calling; the Bond asserts the bucket sum.

## Phase 2 addition

```
Bond("fund", leg, payment, ret) -> (true, Nil) | (false, "closed" | "leg" | "amount")
```

The provider leg is funded at `accept` in per-booking mode, or reserved from a per-provider
prefunded pool for instant booking. The buyer leg remains the purse deposited at creation.
The Bond pays both legs through the same `pay` arm; the remedy table decides the buckets.

> **Where to go deeper.** [Obligations and bonds](../concepts/obligations-and-bonds.md) is the
> model. [Claim classes](claim-classes.md) is the remedy table. [Data schemas](data-schemas.md)
> is the Bond state.
