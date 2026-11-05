# Register schemas

The register fields are defined here; `tools/check_audit.py` is the executable schema.
Do not add a second JSON Schema definition until a second consumer exists.

## Finding

Required: `id`, `title`, `severity` (`P0`–`P3`), `class`, `status`
(`confirmed|suspected|disputed|fixed|deferred|accepted`), `confidence`, `components`,
`arms`, `hazards`, `invariants`, `evidence_level` (`E0`–`E4`), `evidence`, `description`,
`preconditions`, `impact`, `proposed_fix`, `reviewer`, `verified_at`.

`evidence` entries are objects with `kind` (`source|test|command|model|docs`) and `ref`.

## Hazard

Required: `id`, `node`, `guide_word`, `parameter`, `deviation`, `causes`,
`consequences`, `severity`, `likelihood`, `risk`, `controls`, `findings`, `tests`,
`disposition` (`avoid|mitigate|transfer|accept|open`), `owner`, `evidence`.

`consequences` has `on_chain`, `operational` and optional `safety`.

## Theorem

Required: `id`, `name`, `module`, `kind` (`theorem|alias|assumption|counterexample`),
`status`, `statement`, `axioms`, `scope`, `evidence`, `reviewer`.

`proved-model` requires a theorem or alias target. `proved-tied` requires `corpus`.
`modeled` and `assumption` are not proofs.

## Improvement

Required: `id`, `priority`, `findings`, `hazards`, `change`, `type`, `breaking`,
`migration`, `compatibility_window`, `tests`, `rollout_stage`, `rollback`, `status`.

## Test

Required: `id`, `name`, `file`, `status` (`planned|implemented|passing|failing|not_run`),
`focus`.

## Cross-links

`tools/check_audit.py` checks that every reference resolves, every P0/P1 finding has at
least one hazard and evidence entry, every hazard has a disposition, and every
`proved-tied` theorem has a corpus. `tools/audit_trace.py --check` verifies the generated
matrix is current.
