# Lifecycle and obligations

**Status:** Phase 1 states are implemented; the Phase 2 claim/resolve generalization and
provider bond are design proposals.

## 1. States and derived phases

```
requested -> confirmed -> checkedIn -> checkedOut -> settled
    |            |            |
    +-> rejected +-> cancelled +-> disputed -> settled
    +-> expired  +-> noShow
                 +-> redeemed (events) -> settled
```

Derived phases:

- **pre-service:** `requested`, `confirmed`
- **in-service:** `checkedIn` (or `redeemed` for events)
- **post-service:** `checkedOut` (or `redeemed`)
- **terminal:** `settled`, `rejected`, `cancelled`, `expired`, `noShow`

## 2. Transitions

| From | Arm | Actor | To | Guards |
|---|---|---|---|---|
| requested | accept | provider | confirmed | now < holdUntil; slots confirm |
| requested | reject | provider | rejected | releases slots; full refund |
| requested | expire | anyone | expired | now >= holdUntil |
| requested/confirmed | cancel | buyer | cancelled | policy window |
| requested/confirmed | cancelByProvider | provider | cancelled | full refund |
| confirmed | checkIn | provider | checkedIn | checkInHeight <= now <= checkOutHeight |
| confirmed | claimNoShow | provider | noShow | now >= checkInHeight + noShowBlocks |
| confirmed | redeem | provider | redeemed | event; exactly once |
| checkedIn | checkOut | provider | checkedOut | sets depositReleaseHeight |
| checkedIn | guestCheckOut | buyer | checkedOut | now >= checkOutHeight |
| checkedOut | settle | anyone | settled | now >= depositReleaseHeight; no pending claim |
| checkedOut | claimDeposit | provider | checkedOut | claim <= deposit; before release |
| checkedOut | resolveDeposit | anyone | settled | claim exists; dispute window elapsed |
| redeemed | settle/closeEvent | anyone | settled | event over |
| any non-terminal | dispute | party | disputed | not terminal |
| disputed | arbitrate | 2-of-3 | settled | class-bounded split |
| any | setAdmins | 2-of-3 | same | current threshold |

## 3. Guards and actors

- **Buyer:** `guestId`, derived from the deploy key. Cancels, disputes, guest-checks-out,
  transfers a ticket.
- **Provider:** `providerId`, derived from the deploy key. Accepts, rejects, checks in/out,
  claims damage or no-show, redeems.
- **Anyone:** `settle`, `expire`, `resolveDeposit`, `closeEvent`. These are the liveness
  paths.
- **2-of-3:** `arbitrate`, `setAdmins`, pause/delist, Root admin.

## 4. Obligation discharge

| Obligation | Discharged when | Evidence |
|---|---|---|
| buyer pays | booking creation; bond funded | bond state |
| provider delivers | check-in and the service interval completes | check-in height, evidence hash |
| buyer uses responsibly | checkout, no damage claim or a resolved claim | checkout, evidence hash |
| buyer returns | checkout or guest checkout | checkout height |
| provider cures a failed delivery | before `checkInHeight + cureBlocks` | provider action, evidence hash |

A discharge is a state transition or a claim resolution; it is never a discretionary
on-chain judgment of facts.

## 5. Deadlines and lazy expiry

- Hold: `holdUntilHeight = now + holdBlocks`.
- Cancellation cutoffs: `start - fullRefundBeforeBlocks`, `start - partialRefundBeforeBlocks`.
- No-show: `checkInHeight + noShowBlocks`.
- Deposit review: `checkOutAtHeight + depositReviewBlocks`.
- Dispute: `claim.atHeight + disputeWindowBlocks`.
- Event transfer: `checkOutHeight - eventTransferCutoffBlocks`.

No timer fires. Each deadline is evaluated by the next caller; the terminal arms are
permissionless so a stalled booking can always be finished.

## 6. Terminal states

A terminal state is permanent. A replay of a mutating arm returns the stored result and moves
no money. The Bond is single-use; a second `pay` refuses `"closed"`.

## 7. Event specialization

Events reuse the lifecycle with three arms:

- `redeem`: provider admits the holder, exactly once.
- `transferTicket`: buyer moves admission before the cutoff, with a zero deposit.
- `closeEvent`: anyone settles an unredeemed ticket after the event.

The ticket is a projection of the booking, not a second state machine. This is the only
kind-specific arm set and should not be copied as a template for new classes.

## 8. Race semantics

- One slot, two buyers: exactly one hold; the other `"slot held"`.
- Disjoint slots: both succeed.
- Overlapping multi-slot: one booking holds all its slots or none.
- Accept vs cancel: one terminal transition; the other `"closed"`.
- Expiry race: a later holder reclaims; the old booking refunds and its release is
  `"not holder"`.
- Settle vs settle: one payout; the Bond is `"closed"` to the loser.
- Claim vs settle: one terminal path; no double payout.
- Arbitrate vs default resolve: one resolution.
- Pause vs book: the deploy order decides; a committed booking stands, a later one refuses
  `"paused"`.

## 9. Phase 2 claim generalization

`claim(class, amount, evidenceHash, opId)` and `resolveClaim(opId)` replace
`claimDeposit`, `claimNoShow` and `resolveDeposit`; `arbitrate` remains. The class selects
the remedy row and the claimant/window; the chain applies the deterministic default or a
class-bounded signed override. The lifecycle adds no new state; a claim is a pending field on
the existing booking state.
