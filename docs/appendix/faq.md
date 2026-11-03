# FAQ

> **Status:** Reference. No promises.

## Is r-share an Airbnb?

No. It is a primitive layer that an Airbnb-like product could use. It has no UI, no
marketplace, no ranking, no customer relationship and no off-chain database.

## Is it production-ready?

No. It is devnet-only and unaudited, pinned to r-cash v0.5 at `49b441d`. Do not put real
value on it.

## Has it been tested?

The static and parser gates pass. Two behavioural suites (`unit_root`, `unit_payments`) are
implemented but have not run on a healthy devnet here; the rest are placeholders. Read
[Status, scope and honest limits](../orientation/status-and-scope.md).

## Why not use a normal database and a payment processor?

A database plus a processor works for one operator. It does not give the guest a verifiable
escrow, a deterministic cancellation policy, or a settlement that neither party controls.
r-share is for products that need those properties.

## Why r-cash?

r-cash is the payment rail: multi-currency accounts, ERTP payments, deposit facets and a
permissionless transfer layer. r-share assumes it rather than minting its own asset.

## Why not r-cash's TransferBook for escrow?

`TransferBook` is a secret-hash, all-or-nothing claim escrow with a sender-only refund. A
rental needs provider accept/reject, rent/fee/deposit splitting, partial cancellations,
damage claims, disputes and event redemption. r-share holds a per-booking purse and pays
exact buckets.

## Can I use it for bikes, cars or robots?

The shared path is designed for it. The bike, car and robot chapters are mappings, not
verified implementations; `unit_generality` is planned (M7). The RaaS-specific pieces are
design intent.

## Why are slot IDs strings instead of dates?

Because the chain cannot trust a wall clock. The provider maps dates to block heights; the
chain enforces heights. The slot ID is the human-facing key.

## What happens if nobody calls `expire` or `settle`?

The escrow stays in the booking's purse. `expire`, `settle` and `closeEvent` are
permissionless, so anyone can finish a stalled booking. Guests are refund-incentivized.

## Who arbitrates a dispute?

A 2-of-3 admin set captured at booking creation. It splits the deposit; it does not decide
the facts. Inspection, evidence and communication are off-chain.

## Is the data private?

No. Listings, bookings, dates, prices and addresses are on-chain. Do not put personal data
on-chain.

## Can I change the cancellation policy?

Yes. `Remedy` is a pure contract; add a branch, keep the bucket sum invariant, and test the
boundaries. See [Extend a policy](../guides/extend-a-policy.md).

## Can I add a new kind of inventory?

Yes, if it fits capacity over an interval. Change `kind`, `attrs`, slot IDs, terms and a
pure policy branch. Do not add a booking arm. See [Adding a new resource
class](../case-studies/new-resource-class.md).

## Can I run this as a business?

Only with your own legal, insurance, consumer-protection, data-protection and safety work.
The AGPL network obligation applies to a modified version. See [Operate
safely](../guides/operate-safely.md) and [Provenance and license](provenance-and-license.md).

## Why is the book stock mdBook with ASCII diagrams?

So it builds everywhere and survives every fork. No plugin toolchain, no diagram renderer,
no broken math.

> **Where to go deeper.** [Fork and run](../guides/fork-and-run.md) is the setup. [Contract
> map](../reference/contract-map.md) is the code. [Roadmap](roadmap.md) is the plan.
