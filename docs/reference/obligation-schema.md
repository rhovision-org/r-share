# Obligation schema

> **Status:** Design proposal (Phase 2). Phase 1 encodes the same information implicitly in
> the terms and the booking lifecycle.

An obligation is named data attached to a booking. It does not create a new state cell.

| Field | Type | Meaning |
|---|---|---|
| `obligor` | `"provider"` or `"buyer"` | who owes the duty |
| `kind` | `"deliver"`, `"use"`, `"pay"`, `"return"` | the duty |
| `trigger` | string | the height or event that starts the window |
| `deadlineHeight` | Int | the last height for performance |
| `evidenceSchema` | Hash | the schema of the evidence hash |
| `claimClass` | string | the remedy row a breach selects |
| `cureBlocks` | Int | the provider's cure window, 0 for buyer duties |
| `severity` | `"minor"` or `"major"` | a hint for the arbiter, not a money rule |

Example:

```text
{"obligor": "provider",
 "kind": "deliver",
 "trigger": "checkInHeight",
 "deadlineHeight": 12000,
 "evidenceSchema": "<hash of the delivery/inspection schema>",
 "claimClass": "providerDeliveryBreach",
 "cureBlocks": 12,
 "severity": "major"}
```

## Where it lives

- The listing's terms carry an `obligations` hash (Phase 2 `rshare:terms:v2`).
- The booking copies the obligation set into its state at creation.
- A claim names one `claimClass`; the remedy table resolves it.

No obligation cell, no per-obligation contract and no second writer. The booking state stays
the single authoritative cell.

## Validation

- `obligor` and `kind` are from the fixed vocabulary.
- `deadlineHeight` is a height, never a timestamp.
- `claimClass` is a member of the versioned remedy table.
- `cureBlocks` is bounded by the terms and the slot window.
- The obligation set hash is part of the terms hash; a guest can prove what they accepted.

> **Where to go deeper.** [Obligations and bonds](../concepts/obligations-and-bonds.md) is the
> narrative. [Claim classes](claim-classes.md) is the remedy vocabulary. [Space-time
> spec](https://github.com/PatrickMockridge/r-share/blob/main/spec/space-time.md) is the
> normative proposal.
