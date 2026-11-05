# Audit and HAZOP method

## 1. Baseline

Freeze the commit, toolchain, rholo and r-cash pins, devnet manifest and hashes of the
build, Lean, flow and invariant inputs. Reproduce the static, formal and behavioural gates;
record pass, fail and not-run verbatim. Do not reuse an older suite-results file as
evidence.

## 2. Inventory and reconciliation

Extract every public arm from `contracts/src/deploy/` and compare it with
`contracts/components.json`, `spec/flows/core.yaml`, `docs/reference/arms.md` and
`harness/run_suites.py`. Every arm is classified as `cornerstone`, `unmapped` or `internal`
with a rationale. Every `proved-model` flow rule must resolve to a theorem or alias whose
statement carries the arm's obligation; a definition or constructor is `modeled`, not
proved.

W1 is a hard gate. HAZOP starts only after the arm inventory and theorem status are
reconciled, otherwise the register inherits the current arm and status confusion.

## 3. HAZOP

### Nodes

The node list is `hazop/nodes.yaml` (N1–N20). A node is a contract, a group of arms that
share a state cell, or an external boundary that changes contract behaviour.

### Guide words

Parameters: amount, authority, signature count, nonce/opId, state/status, unit, slot,
booking, asset, deadline/height, cell, release token, admin key set and provider index.

Guide words: `NO/NOT`, `MORE`, `LESS`, `AS WELL AS`, `PART OF`, `REVERSE`, `OTHER THAN`,
`EARLY`, `LATE`, `BEFORE`, `AFTER`, `REPEAT`, `CONCURRENT`, `UNAUTHORIZED`, `STALE`, `LOST`,
`DUPLICATE`, `UNKNOWN`, `UNAVAILABLE`.

For each credible node × guide-word × parameter combination record the deviation, causes,
on-chain consequence, operational consequence, safety consequence where relevant, existing
controls, severity, likelihood, risk, finding, test, Lean property, disposition and owner.
Screen combinations; do not record every pairing.

### Risk scale

| Severity | Meaning |
|---|---|
| S4 Critical | irrecoverable loss or lock of funds/inventory, unauthorized transfer, double sale, governance takeover, safety-relevant off-chain harm |
| S3 High | temporary loss or lock, bounded misallocation, privacy leak, platform-wide liveness outage |
| S2 Medium | griefing, degraded liveness, incorrect status/projection |
| S1 Low | docs/UX/refusal mismatch, no value or authority impact |

Likelihood is L4 (likely) to L1 (remote). Risk is the matrix product. An unresolved S4/L3+
hazard blocks an implementation wave unless the disposition is an explicit accepted risk
with a named owner.

### Seed hazards

The first pass checks the seed hazards in `registers/hazards.yaml`. They are reading-level
observations; each is reproduced or downgraded to `suspected` before it becomes a confirmed
finding.

## 4. Contract audit

Checklist per arm:

- authority derived from `deployerId`, never from an argument;
- the state cell is reposted on every normal branch; no take-without-put;
- refusal before value moves; `1/0` only after a value move;
- release token custody and projection redaction;
- opId/nonce replay and idempotency;
- integer arithmetic, bounds and bucket conservation;
- deadline and height ordering, including equality boundaries;
- state-machine terminality and permissionless paths;
- pause scope and emergency actions;
- factory origin and input authentication;
- dependency-failure behaviour and unmatched `for` paths;
- docs claims versus source.

## 5. Dynamic verification

Dynamic evidence uses a fresh devnet and the harness in `harness/`:

- fault injection against a test Root/registry and test adapters;
- boundary matrix around heights, amounts, bps, slots, units and admin thresholds;
- adversarial paths: reset abuse, direct factory, pause bypass, resume delisted, early
  settle, no-show without check-in, duplicate keys, replay and wrong token;
- cell liveness: after every success/refusal, read the state cell with a timeout and scan
  `GuestLedger("ops")` for pending entries;
- deterministic property/soak sequences across listings and bookings.

Every fault test needs a negative control showing it can fail. A test that cannot fail is
not evidence.

## 6. Lean audit

- Inventory every theorem and every `#print axioms` line. `check_lean.py` requires exactly
  the registered theorem set, not a minimum line count.
- Check every flow rule: theorem or alias, statement, scope, axioms and tie.
- Check model fidelity: allocator schedule/window checks, booking guards, reset, admin,
  ERTP and ledger failure paths.
- Record `proved-model`, `proved-model-bounded`, `modeled`, `assumption`, `owed` and
  `proved-tied` honestly.
- `proved-tied` requires a generated corpus, a harness run and a pinned evidence manifest.

## 7. Reporting

Findings are ranked by severity and class. The report states counts, the P0/P1 table, the
theorem coverage, what was not run, residual risk and non-claims. A finding is not closed by
a proposed fix; it is closed by evidence that the fix is deployed and verified.
