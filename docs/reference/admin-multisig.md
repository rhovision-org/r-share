# Admin multisig

> **Status:** Implemented in the CLI and the web console. The devnet journey exercises the
> one-signature refusal and the two-signature pause/resume.

## Payload

Every admin and arbiter action verifies a canonical payload:

```text
rshare:admin:v1|<domain>|<nonce>|<action>|<arg0>,<arg1>,...
```

The digest is BLAKE2b-256 over the UTF-8 payload. Signatures are DER secp256k1 over the
digest. The deployed `Codec("adminPayload")` produces the payload; the client never
reconstructs it in TypeScript.

## Domains

| Action | Domain | Action | Args |
|---|---|---|---|
| Global pause/resume | `admin` | `pauseAll` / `resumeAll` | `[]` |
| Listing admin pause | `listing:<listingId>` | `pause` / `delist` | `[]` |
| Provider admin pause | `provider:<ownerId>` | `pause` | `[]` |
| Booking arbitration | `arbiter` | `split` | `[bookingId, splitBps]` |

`ShareAdmin("admin")` uses the global admin set; listing, provider and booking use their
own `admins` and `threshold`.

## The `setAdmins` quirk

The call carries the new admin byte arrays and threshold, but the signed payload carries
the comma-joined public-key text and threshold:

- payload args: `[adminsText, threshold]`
- call args: `[byte-array list, threshold]`

The SDK exposes `set_admins_payload_args` and the typed `setAdmins` helpers so the two are
not confused.

## Nonce

A nonce must be strictly greater than the target's stored nonce; a replay refuses `replay`.
Listing, provider and booking expose their nonce, so the client uses `current + 1`. The
global `ShareAdmin` has no nonce read: use the finalized height plus a margin (the web
console and CLI default to height + 1000) and treat a refusal as a signal to read state.

## Ceremony

1. Pick the target and action.
2. Build the payload; display the exact text and digest.
3. Sign with each saved profile until the threshold is reached. A profile signs at most
   once per payload.
4. Submit with any funded profile; signers need not be the submitter.

The web console stores pending actions and signatures in IndexedDB. The CLI accepts
comma-separated hex signatures:

```sh
rshare-cli admin payload --domain admin --nonce 12345 --action pauseAll
rshare-cli --key admin1 admin sign --payload '<text>'
rshare-cli --key admin1 admin pause-all --nonce 12345 --sig <hex>,<hex>
```

## Limits

There is no global nonce arm, no time-locked governance, no root rotation UI and no
automatic signature relay. Those are out of scope for this client.

> **Where to go deeper.** [Governance and arbitration](../concepts/governance-and-arbitration.md)
> is the model. [Arm reference](arms.md) lists the admin arms. [Client
> architecture](client-architecture.md) is the layer map.
