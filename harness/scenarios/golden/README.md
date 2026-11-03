# Golden transcripts

Each `.rho` here is a fixed lifecycle run by `harness/unit_behavior.py` on the R0 and R7
deployments. The runner normalizes:

- heights to deltas from the first deploy;
- booking, listing and unit state projections;
- slot states, op maps, account balances and bond balances;
- refusal strings and terminal statuses.

The diff must be empty except for the deltas registered in `spec/behavior-delta.md`.

The suite needs a healthy r-cash devnet. It is `not run` when `acceptsDeploys` is false.
