# Evidence

Each recorded run lives under `audit/evidence/<date>-<commit>/`:

```
manifest.json
logs/<command>.log
results/<suite>.json
```

`manifest.json` records the run id, date, commit, dirty state, command, exit code,
duration, node/rholo/r-cash/Lean versions, input and output hashes, artifacts, operator and
notes.

Evidence is published in full except credentials. Redaction is limited to private keys,
passwords, keystore ciphertext and node tokens; findings, reproduction commands and state
transitions are not removed. Never commit `.devnet/keys.json`, a keystore, a password or a
node token.

A finding is `confirmed` only when its evidence is E2 or higher (reproduced on a pinned
devnet or executable model). Static `file:line` evidence is E1 and remains `suspected` for
P0/P1.

The current baseline is `/home/patrick/RShare` at `dee7cff`; see
`2026-11-04-dee7cff/manifest.json` once the run is recorded.
