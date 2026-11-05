# Audit method

> **Status:** Baseline desk audit at `dee7cff`. Five P0/P1 findings are reproduced on an
> isolated devnet; the rest are static or suspected. The registers live under `audit/`.

## Why this exists

The recorded 16-suite campaign shows that selected paths work on one devnet. It is not an
audit. This chapter describes the method used to inventory every arm, reconcile the Lean
claims, run a HAZOP and produce a prioritised Rholang backlog.

## Authority order

1. `contracts/src` and the built/deployed terms: what the code does.
2. `spec/lean` and the formal gates: what is machine-checked.
3. `spec/flows`, `spec/invariants.md`, `spec/coverage.yaml`: claimed traceability.
4. `docs/`: claims to be audited, not authority.

## Evidence levels

| Level | Meaning |
|---|---|
| E0 | Unverified observation |
| E1 | Source `file:line` plus command, or a reproducible source trace |
| E2 | Reproduced on a pinned devnet or executable model |
| E3 | Machine-checked theorem plus conformance corpus |
| E4 | `proved-tied` observational conformance |

A P0/P1 finding is `confirmed` only at E2 or above. Evidence is published in full except
credentials; only private keys, passwords, keystore ciphertext and node tokens are removed.

## Work packages

| WP | Work | Output |
|---|---|---|
| W0 | Freeze baseline and toolchain | `audit/evidence/<date>-<commit>/manifest.json` |
| W1 | Arm and theorem reconciliation | `audit/registers/arms.yaml`, `theorems.yaml` |
| W2 | HAZOP N1–N20 | `audit/registers/hazards.yaml` |
| W3 | Desk audit | `audit/registers/findings.yaml` |
| W4 | Lean L0–L1 | complete audit, rent/reset/event/ledger theorems |
| W5 | Lean L2 | concurrency and deploy atomicity |
| W6 | Lean L3 | admin threshold and ERTP conservation |
| W7 | Dynamic verification | green regression suites, `harness/faults.py`, boundary suites |
| W8 | Improvement plan | `audit/registers/improvements.yaml` |
| W9 | Docs and report | this chapter, `audit/report.md` |
| W10 | Independent review | review record; otherwise label self-audit |

## Gates

```sh
make check          # static, parser, docs and formal gates plus audit-check
make audit-check    # register schemas, arm inventory, traceability
make audit-dynamic  # fault/factory regression; needs a fresh devnet
```

`make audit-check` validates the registers; it does not decide whether a finding is a real
vulnerability, and it does not fail because a finding is open. `audit-dynamic` runs the
fault-injection and factory-origin regressions; it is a green gate on a fresh devnet.

## What the method does not do

It does not prove a refinement of Rholang, certify rholo/r-cash, adjudicate legal or
factual questions, or replace an external security review. The limits are repeated in
[Formal model](formal-model.md) and [Verification record](verification.md).
