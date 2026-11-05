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
own `admins` and `threshold`. A v2 admin set requires at least two distinct public keys and
a threshold of at least 2; duplicate keys, threshold 1 and threshold greater than the key
count are refused. A v1 object that already has duplicates or threshold 1 fails closed under
the v2 guard and must be re-seeded or rotated.

## The `setAdmins` quirk

The call carries the new admin byte arrays and threshold, but the signed payload carries
the comma-joined public-key text and threshold:

- payload args: `[adminsText, threshold]`
- call args: `[byte-array list, threshold]`

The SDK exposes `set_admins_payload_args` and the typed `setAdmins` helpers so the two are
not confused.

## Nonce

A nonce must be strictly greater than the target's stored nonce; a replay refuses `replay`.
Every admin domain now exposes its current nonce: `ShareAdmin("nonce")`,
`ShareRoot("nonce")`, and the `nonce` field in listing, provider and booking `info`.
Clients read the current value and sign `current + 1`; they never derive a nonce from block
height or wall-clock time.

Two clients can read the same `N` and build `N+1`; the chain accepts one. Signatures bind
the nonce, so a replay is a conflict, not an invitation to resubmit: re-read the nonce,
invalidate the pending action, rebuild the payload and sign again.

## Ceremony

1. Pick the target and action.
2. Build the payload; display the exact text and digest.
3. Sign with each saved profile until the threshold is reached. A profile signs at most
   once per payload.
4. Submit with any funded profile; signers need not be the submitter.

The web console stores pending actions and signatures in IndexedDB. The CLI accepts
comma-separated hex signatures:

```sh
rshare-cli admin nonce                     # current and next
rshare-cli admin nonce --root              # governance nonce
rshare-cli admin payload --domain admin --nonce 12345 --action pauseAll
rshare-cli --key admin1 admin sign --payload '<text>'
rshare-cli --key admin1 admin pause-all --nonce 12345 --sig <hex>,<hex>
```

## Limits

There is no time-locked governance, no root rotation UI and no automatic signature relay.
Those are out of scope for this client.

> **Where to go deeper.** [Governance and arbitration](../concepts/governance-and-arbitration.md)
> is the model. [Arm reference](arms.md) lists the admin arms. [Client
> architecture](client-architecture.md) is the layer map.
