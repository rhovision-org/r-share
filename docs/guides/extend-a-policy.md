# Extend a policy

> **Status:** Implemented. `Remedy` is a pure, stateless contract.

`Remedy` is the only place where the rental arithmetic lives. It has no state, so a change is
a pure function change and easy to test. This guide shows how to add a branch without
touching the booking state machine.

## The existing arms

| Arm | Purpose |
|---|---|
| `fee(rent, feeBps)` | fee and provider rent |
| `cancel(kind, policy, now, startHeight, rent, deposit, actor)` | cancellation buckets |
| `noShow(rent, deposit, feeBps)` | no-show buckets |
| `depositSplit(deposit, providerBps)` | arbitration split |
The canonical signatures are in the [arm reference](../reference/arms.md). The contract is
`contracts/src/deploy/04_remedy.rho`. Validation and hashing live in `Terms`
(`validate`, `scheduleOk`, `hash`); Remedy only does arithmetic.

## The cancellation arithmetic

The standard branch is:

```
guest, now <= start - fullRefundBeforeBlocks      -> full rent + deposit
guest, now <= start - partialRefundBeforeBlocks   -> floor(rent * partialRefundBps / 10000) + deposit
guest, later                                      -> deposit only
provider                                          -> full rent + deposit
```

`fee` is zero on a cancellation. It applies only to earned rent at settlement or no-show.

## Add a kind-specific branch

Suppose boats need a weather clause: if the provider cancels for weather inside 12 blocks of
the start, the guest gets a full refund, and the provider gets nothing.

1. Add `"boat"` to the `kind` check in `Terms.validate`.
2. In `Remedy.cancel`, add a `kind == "boat"` branch before the standard one:

   ```rholang
   if (kind == "boat"
       && actor == "provider"
       && now >= startHeight - 12) {
     ret!((true, {"providerRent": 0, "platformFee": 0,
                  "guestRefund": rent + deposit, "providerDamage": 0,
                  "reason": "weather"}))
   } else { ... standard ... }
   ```

3. Keep the branch pure: no registry lookups, no state, no external calls.
4. Add vectors for the boundaries (`now == start - 12`, `now == start - 13`).
5. Add a suite case that books, cancels at each boundary, and checks the buckets.

## Add a fee model

If the platform fee is tiered, change `Remedy.fee` and keep the bucket sum invariant:
`fee + net == rent`. Do not change the settlement arm in `BookingContract`; it calls
`Remedy.fee` and asserts the buckets.

## Add a deposit split model

`Remedy.depositSplit` is `floor(deposit * providerBps / 10000)`. If you need a fixed
deductible or a cap, change only this arm. The arbitration arm passes `splitBps`; the
settlement asserts the buckets.

## Rules

- **Pure.** No state cell, no resolution, no sends to other contracts.
- **Integer.** No floats; floor division with a pinned denominator (10000 for bps).
- **Bounded.** Reject negative and out-of-range inputs.
- **Tested.** Every branch needs a boundary test.
- **Sum-preserving.** The output buckets must sum to `rent + deposit`.
- **No new booking arm.** If the policy needs one, the requirement is not a policy change.

## Where the policy is called

- `ListingFactory` calls `termsOk` and the codec hash.
- `UnitFactory.validate` calls `scheduleOk`.
- `BookingContract` calls `cancel`, `noShow`, `depositSplit` and `fee`.
- `ListingContract.ensureUnit` calls `UnitFactory.validate`.

A change to `Remedy` is visible to every listing and booking that resolves
`component:remedy` (or the `component:policy` alias). Version the Root entry if you deploy a
breaking change.

> **Where to go deeper.** [Governance and arbitration](../concepts/governance-and-arbitration.md)
> is the dispute path. [Arm reference](../reference/arms.md) is the signature list. [Test your
> fork](test-your-fork.md) is the gate.
