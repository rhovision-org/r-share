# Fund a devnet account

> **Status:** Implemented for the devnet. Not a production deposit rail.

r-share needs two balances:

| Balance | Used for | How it is funded |
|---|---|---|
| Native REV | deploy phlo | devnet faucet |
| r-cash account REV | booking escrow and Bonds | ERTP `revFund` from the caller's native vault |

## Faucet

```sh
rshare-cli --key sender wallet faucet
```

The node's faucet route exists only in dev mode with a funded deployer key. It drips a small
amount of native REV to the caller's REV address.

## Move native REV into the account

```sh
rshare-cli --key sender wallet fund --amount 10000000
```

`wallet fund` calls the r-cash account's deposit facet and ERTP `revFund`. It spends the
caller's own native vault; it does not mint. In the web app, **My bookings** shows the
native and account balances and the **Move native REV to account** control.

This is the funding step the guest ledger uses before it creates a Bond. The amount must
cover `rent + deposit` plus a margin.

## Withdraw

```sh
rshare-cli --key business wallet withdraw --amount 10000000 --to <REV address>
```

The path is account `withdraw` → ERTP purse → `revRedeem` to the named address. It is the
exit path for provider earnings and guest refunds. It is labelled devnet/operator because
r-share itself is devnet-only.

## Boundary

There is no fiat ramp, card rail or KYC. A browser cannot mint. Any production deployment
must supply its own regulated on-ramp and must not present `revFund` as a deposit product.

> **Where to go deeper.** [Escrow and settlement](../concepts/escrow-and-settlement.md) is
> the money path. [Platform constraints](../reference/platform-constraints.md) covers phlo.
> [Use the web app](use-the-web-app.md) is the UI path.
