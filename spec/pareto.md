# Pareto analysis of universal primitive cores

**Status:** Analysis and proposal. The "current" column is measured; the other columns are
estimates and are marked as such.

## 1. Purpose

Choose the next primitive set that increases use-case coverage without adding a trusted
party or a global bottleneck. The comparison is between:

- the current buyer-deposit core;
- a bilateral-bond core with a generalized claim and deterministic remedy table;
- an oracle/SLA core that settles on attested quality or metering;
- a bilateral core plus evidence anchors.

## 2. Axes and hard constraints

Primary axes, chosen with the maintainer:

1. **Coverage** — a class × claim cell counts only when a scenario passes. Mapping-only and
   design-only cells do not count.
2. **Trust assumptions** — the number of parties whose misbehaviour or unavailability can
   change the outcome, excluding the chain's consensus and the parties' own keys.

Secondary axes: on-chain bytes, state cells, dispute steps, liveness.

Hard constraints:

- no on-chain oracle or fact adjudication;
- evidence is a hash plus metadata;
- one private cell per slot and per booking;
- no global hot cell, timer or `select`;
- heights only; no trusted wall clock.

## 3. Dominance rule

A dominates B iff:

- coverage(A) ≥ coverage(B);
- trust(A) ≤ trust(B);
- bytes, cells, dispute steps and liveness are no worse;
- at least one dimension is strictly better.

A candidate that violates a hard constraint is not on the frontier.

## 4. Candidate cores

| Candidate | Coverage | Trust | Bytes | Cells / booking | Dispute steps | Liveness |
|---|---|---|---|---|---|---|
| Current buyer-deposit core | 0 passing; 20 mapping; 10 design | 3 | 22,987 B max measured; 13 terms | 2 + N slots | 2 / 3 | permissionless settle/expire/resolve; pause now enforced |
| **Bilateral bond + generalized claim** | target 30/30; all cells design until suites pass | 3 | +4–6 KiB est. | 2 bond legs + booking + N slots | 2 / 3 | permissionless resolveClaim/expire/settle |
| Oracle/SLA | + quality and metering | 4 (adds oracle) | +10–15 KiB est. | 2 + attestations | 3 / 5 | depends on oracle quorum/freshness |
| Bilateral + evidence anchors | + evidence classes | 3 | +1–2 KiB over bilateral | 2 + evidence log | 2 / 3 | same as bilateral |

The estimates are design targets, not measurements. `spec/pareto-scorecard.json` records the
measured current metrics and the target coverage; `tools/pareto.py` regenerates it.

## 5. Measurement and tooling

- `tools/metrics.py` measures terms, bytes, depth, contract count, components, state-cell
  types, arm declarations and resolve/registry sites.
- `spec/coverage.yaml` is the class × claim manifest. `tools/check_coverage.py` validates it;
  only `passing` cells count.
- `spec/trust.md` lists the assumptions; `tools/check_no_oracle.py` fails an undeclared
  component key.
- `tools/check_remedy_table.py` validates the remedy classes and bounds.
- `tools/pareto.py` regenerates `spec/pareto-scorecard.json`; CI fails on drift.

## 6. Scorecard

The committed scorecard records:

```
coverage: {total, passing, mapping, design}
trust_assumptions: [root/installer bootstrap, 2-of-3 admin/arbiter, r-cash rail]
metrics: {terms, components, state_cell_types, bytes_max, arm_declarations}
candidates: [current, bilateral, oracle, bilateral+anchor]
```

The current core is measured. The bilateral and oracle rows are estimates. A row becomes
measured only when its scenarios pass and its metrics are built.

## 7. Sensitivity

The choice of bilateral core is robust to:

- a higher or lower provider bond (the leg is collateral, not a new party);
- a longer or shorter cure window (a policy parameter);
- a new class (kind/attrs/policy only);
- a stricter evidence schema (hash-only).

It changes if:

- the law or a regulator requires an on-chain attestor (then the trust axis changes);
- metering must settle continuously (a new primitive and a new trust boundary);
- provider bonds are commercially impossible (fall back to buyer-only deposit and a
  reputation layer);
- a class needs a booking arm (the primitive is wrong and the change belongs in a fork).

## 8. Chosen core

The **bilateral bond + generalized claim + deterministic remedy table + evidence hashes** is
the candidate with the best coverage/trust trade-off in this analysis. It is a hypothesis,
not a measured frontier: `coverage_passing` is 0 until the claim-class scenarios run, and
the alternative rows are estimates.

- It covers both obligations with no new trusted party; the 2-of-3 arbiter already exists.
- It adds one bond contract and one claim class parameter, not a new state machine.
- It keeps the per-slot, per-booking concurrency model.
- It excludes the oracle core because that adds a trust assumption the maintainer explicitly
  rejected.

The Phase 3 evidence anchor is the next Pareto improvement. The Phase 4 provider bond/cure
is the implementation of the bilateral leg. Phase 5 metering is a separate, bounded
extension and is not on the core frontier.

## 9. Phase roadmap and monotonicity

| Phase | Adds | Must not add |
|---|---|---|
| 0 Current | baseline metrics, golden transcripts | contract changes |
| 1 Verified behavior | modular base layer, Terms/Remedy/Bond, pause/contiguous | claim classes, provider bond, oracle, global hot cell |
| 2 Pure-policy generalization | Terms v2, obligation objects, remedy table, generalized claim | oracle, second writer, arbitrary policy code, global registry |
| 3 Evidence anchors | per-booking anchors, challenge windows | fact adjudication, live-name registry, global append-only hot cell |
| 4 Provider bond/cure | bilateral legs, provider funding, cure window | shared bond pool on hot path, timers, unbounded liability |
| 5 Metering | signed readings, buyer challenge, partial-performance remedy, bond caps | on-chain oracle, per-block billing, uncapped exposure |

Each phase must be non-dominated by the previous on the scorecard. A phase that adds a trust
party, disproportionately raises bytes, adds dispute steps or reduces liveness is rejected or
moved to a fork.

## 10. What would change the choice

- A passing end-to-end suite that shows the bilateral model cannot settle without an oracle.
- A regulator requiring on-chain identity or quality attestation.
- A metering product whose price cannot be expressed by a bounded pro-rata rule.
- A provider-bond market that makes the leg unfundable.
- A new class that genuinely needs a new lifecycle arm.
