# Audit charter

## Purpose

Produce a reviewable desk audit of the r-share on-chain primitives and Lean model, a HAZOP
that converts credible deviations into formal obligations, and a prioritised Rholang
improvement plan. The audit records evidence; it does not apply contract fixes.

## Scope

In scope:

- all 13 deploy terms and 21 components in `contracts/src/deploy/`;
- all state cells, public and internal arms, factories, admin and pause surfaces;
- terms, remedy, guest-ledger booking, booking lifecycle, Bond and Payments adapter;
- the Lean project, flow manifest, vectors, axiom audit and formal gates;
- behavioural harness, fixtures, recorded evidence and docs claims that affect a contract
  guarantee;
- off-chain boundaries only where they create an on-chain assumption: `deployerId`,
  `rho:block:data`, registry lookup, r-cash account/purse/facet, deploy atomicity and
  finality.

Out of scope:

- rholo node internals, r-cash internals, cryptographic primitive proofs and consensus;
- legal, regulatory, insurance, KYC, privacy and consumer-law adjudication;
- production-readiness claims, economic modelling beyond bounded loss/liveness and UI/UX;
- applying P0 contract fixes in this change.

## Authority order

1. `contracts/src` and the built/deployed terms: what the code does.
2. `spec/lean` and the formal gates: what is machine-checked.
3. `spec/flows`, `spec/invariants.md`, `spec/coverage.yaml`: claimed traceability.
4. `docs/`: claims to be audited, not authority.

## Roles

| Role | Responsibility |
|---|---|
| Lead auditor | scope, arm inventory, HAZOP, findings, report |
| Contracts reviewer | independent read of source-level findings and dispositions |
| Lean reviewer | theorem registry, axiom audit, model fidelity and proof status |
| Devnet operator | fresh devnet, suite runs, evidence capture |
| Independent reviewer | confirms material claims have evidence; records dissent |

If an independent reviewer is unavailable, the report is labelled a self-audit and no
severity is presented as externally validated.

## Evidence standard

| Level | Meaning | Required for |
|---|---|---|
| E0 | Unverified observation | Nothing above P3 |
| E1 | Source `file:line` plus command, or a reproducible source-level trace | All findings |
| E2 | Reproduced on a pinned devnet or executable model | P0/P1 confirmed |
| E3 | Machine-checked theorem plus conformance corpus | Formal claims |
| E4 | `proved-tied` observational conformance | Only after the `proved-tied` checklist |

A P0/P1 finding without E2 is `suspected`, not `confirmed`. Evidence is published in full
except credentials; redaction is limited to private keys, passwords, keystore ciphertext and
node tokens.

## Exit criteria

- Every public arm is inventoried and classified.
- Every high or critical hazard has a finding or a justified disposition.
- Every P0 has a regression-test specification and a proposed change.
- Every `proved-model` flow rule names a theorem whose statement carries the obligation.
- Every proposed change has a hazard, test and Lean property where applicable.
- `make check`, `make audit-check` and `make docs-check` pass.
- The report states what was run, what was not run and what is not claimed.
- The independent-review gate is complete or the report is labelled self-audit.

## Disclosure

The repository is public and AGPL-3.0-or-later. The full findings, HAZOP rows and
reproductions are published. Revisit this policy if a production or third-party deployment
creates users at risk.
