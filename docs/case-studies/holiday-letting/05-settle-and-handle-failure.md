# Settle, dispute and clean up

> **Status:** Implemented, not yet exercised.

The stay is over and the room has been checked out. Settlement is permissionless: Ana, Ben,
or anyone else can finish the booking after the review window.

## Normal settlement

```rholang
BookingContract!("settle", "ben-op-009", *ret)
```

Requires status `checkedOut`, `now >= depositReleaseHeight`, and no pending claim. `Remedy.fee`
splits the rent:

- `fee = floor(rent * feeBps / 10000)`;
- provider rent = `rent - fee`;
- platform fee = `fee`;
- guest refund = the deposit.

For rent 300 and fee 5%, the buckets are 285 to Ana, 15 to the platform, and 50 to Ben. The
contract asserts that the buckets sum to `rent + deposit` before paying. Status becomes
`settled`.

## Damage claim

If the room was damaged, Ana records a claim during the review window:

```rholang
BookingContract!("claimDeposit", *deployerId, "ana-op-010",
                 30, evidenceHash, *ret)
```

- provider only;
- status `checkedOut`;
- `now < depositReleaseHeight`;
- `0 <= amount <= deposit`.

The claim stores the amount, the evidence hash and the height. It does not move money yet.
Status stays `checkedOut` with a pending claim.

## Dispute

Ben can dispute while the booking is not terminal:

```rholang
BookingContract!("dispute", *deployerId, "ben-op-011", reasonHash, *ret)
```

Either party can dispute. Status becomes `disputed`. A disputed booking cannot be settled by
`settle` or `resolveDeposit`; only `arbitrate` resolves it.

## Arbitration

The 2-of-3 admin set splits the deposit:

```rholang
BookingContract!("arbitrate", nonce, splitBps, sigs, "arb-op-012", *ret)
```

`splitBps` is the share of the deposit to the provider, 0–10000. `Remedy.depositSplit`
computes the provider damage and the guest refund. Rent and fee are settled normally. The
signature payload is the `rshare:admin:v1` canonical string over
`[bookingId, splitBps]`; the nonce is stored on the booking. Status becomes `settled`.

The arbiter decides the split, not the facts. The evidence hash is opaque; inspection,
repair quotes and communication stay off-chain.

## Uncontested claim

If Ben does not dispute, anyone can finish after the dispute window:

```rholang
BookingContract!("resolveDeposit", "any-op-013", *ret)
```

Requires status `checkedOut`, a pending claim, and
`now >= claim.atHeight + disputeWindowBlocks`. The claim goes to Ana, the rest of the deposit
to Ben, and rent/fee normally. Status becomes `settled`.

## Expiry and cleanup

- `expire` refunds a requested booking after `holdUntilHeight`.
- A rejected booking is already refunded by `reject`.
- A cancelled booking is already paid by `cancel` or `cancelByProvider`.
- A no-show is already paid by `claimNoShow`.
- A settled, rejected, cancelled, expired or no-show booking is terminal. A replay returns
  the stored result; it does not move money again.

## What can go wrong

| Symptom | Cause | Action |
|---|---|---|
| `settle` refuses `"too early"` | the review window is still open | wait for `depositReleaseHeight` |
| `settle` refuses `"claim pending"` | Ana claimed | dispute or wait for resolution |
| `resolveDeposit` refuses `"not claimed"` | no claim | use `settle` |
| `arbitrate` refuses `"below threshold"` | fewer than 2 valid signatures | collect the signatures again |
| `arbitrate` refuses `"replay"` | the nonce was used | use the next nonce |
| `expire` refuses `"too early"` | the hold is still live | wait for `holdUntilHeight` |
| any arm refuses `"closed"` | terminal state | read `status`; the outcome is final |

## Conservation

After every terminal path, the invariants in [Invariants](../../reference/invariants.md) must
hold: the escrow is empty, the buckets sum to `rent + deposit`, and every slot is free,
held, or booked consistently. The planned M4 suite checks this on a healthy devnet.

## Running the intended suites

```sh
make devnet rcash-install keys install
make unit-lifecycle
```

`unit-lifecycle` is planned (M4) and currently exits 2. Treat a not-run suite as not run.

> **Where to go deeper.** [Escrow and settlement](../../concepts/escrow-and-settlement.md)
> explains the buckets. [Governance and arbitration](../../concepts/governance-and-arbitration.md)
> explains the multisig. [Test your fork](../../guides/test-your-fork.md) is the gate.
