#!/usr/bin/env bash
# One-shot remediation acceptance campaign.
#
# Starts a fresh isolated devnet, installs r-cash and r-share once, runs the full
# suite set, the CLI journey and the browser journeys, and writes one evidence
# attempt directory. It refuses the default devnet and never overwrites evidence.
#
# Required environment:
#   RSHARE_NODE=http://localhost:41403
#   RSHARE_ADMIN=http://localhost:41405
#   RSHARE_CONTAINER=rshare-audit-bootstrap
#   RCASH_DIR=/tmp/rcash-audit
# Optional:
#   AUDIT_DEVNET_SCRIPT=/tmp/devnet-rshare-audit.sh
#   RCASH_CHECKOUT=/home/patrick/Rholphin
#   AUDIT_ATTEMPT=attempt-1
#   KEEP_DEVNET=1
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

NODE="${RSHARE_NODE:-}"
ADMIN="${RSHARE_ADMIN:-}"
CONTAINER="${RSHARE_CONTAINER:-}"
RCASH_DIR="${RCASH_DIR:-}"
DEVNET_SCRIPT="${AUDIT_DEVNET_SCRIPT:-/tmp/devnet-rshare-audit.sh}"
RCASH_CHECKOUT="${RCASH_CHECKOUT:-/home/patrick/Rholphin}"
PREFIX="rshare-audit"
ATTEMPT="${AUDIT_ATTEMPT:-attempt-1}"
EVIDENCE="$ROOT/audit/evidence/2026-11-04-remediation/$ATTEMPT"

fail() { echo "audit-campaign: $*" >&2; exit 1; }

[[ -n "$NODE" && -n "$ADMIN" && -n "$CONTAINER" && -n "$RCASH_DIR" ]] \
  || fail "set RSHARE_NODE, RSHARE_ADMIN, RSHARE_CONTAINER and RCASH_DIR"
[[ "$NODE" != *":40403"* ]] || fail "refusing the default devnet at $NODE"
[[ -x "$DEVNET_SCRIPT" ]] || fail "missing devnet script: $DEVNET_SCRIPT"
[[ -x "$RCASH_CHECKOUT/.venv/bin/python" ]] || fail "missing r-cash checkout: $RCASH_CHECKOUT"
[[ ! -e "$EVIDENCE" ]] || fail "evidence attempt already exists: $EVIDENCE"

export RSHARE_NODE="$NODE" RSHARE_ADMIN="$ADMIN" RSHARE_CONTAINER="$CONTAINER"
export RCASH_DIR DEVNET_PREFIX="$PREFIX" DEVNET_NETWORK="$PREFIX"
export RCASH_NODE="$NODE" RCASH_NO_HOSTLIST=true
export RSHARE_KEYS="$ROOT/.devnet/keys.json"

mkdir -p "$EVIDENCE/logs" "$EVIDENCE/results"

cleanup() {
  if [[ "${KEEP_DEVNET:-0}" != "1" ]]; then
    "$DEVNET_SCRIPT" down -v >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

echo "audit-campaign: fresh $PREFIX devnet on $NODE"
"$DEVNET_SCRIPT" down -v >/dev/null 2>&1 || true
"$DEVNET_SCRIPT" up

echo "audit-campaign: wait for proposals and a finalized signed deploy"
"$ROOT/.venv/bin/python" tools/wait_devnet.py --node "$NODE" --blocks 3 --timeout 300

echo "audit-campaign: install r-cash core"
rm -rf "$RCASH_DIR/.devnet/ops" "$RCASH_DIR/.devnet/v3-install.json" \
  "$RCASH_DIR/deployments/devnet-v4.json"
set +e
"$RCASH_CHECKOUT/.venv/bin/python" "$RCASH_CHECKOUT/harness/install_testnet.py" \
  --node "$NODE" \
  --keys "$RCASH_CHECKOUT/.devnet/keys.json" \
  --admin-keys "$RCASH_CHECKOUT/.devnet/keys.json" \
  --ops "$RCASH_DIR/.devnet/ops" \
  --state "$RCASH_DIR/.devnet/v3-install.json" \
  --manifest "$RCASH_DIR/deployments/devnet-v4.json" \
  --min-lock-blocks 10 --fresh 2>&1 | tee "$EVIDENCE/logs/install-rcash.log"
rcash_rc=${PIPESTATUS[0]}
set -e
if [[ "$rcash_rc" -ne 0 ]]; then
  echo "audit-campaign: v4 installer stopped at $rcash_rc; checking core steps"
fi
"$ROOT/.venv/bin/python" tools/rcash_core.py \
  --rcash-checkout "$RCASH_CHECKOUT" \
  --state "$RCASH_DIR/.devnet/v3-install.json" \
  --manifest "$RCASH_DIR/deployments/devnet-v4.json" 2>&1 \
  | tee "$EVIDENCE/logs/install-rcash-core.log"

echo "audit-campaign: install r-share"
make keys install NODE="$NODE" | tee "$EVIDENCE/logs/install.log"
make client-fixtures | tee "$EVIDENCE/logs/fixtures.log"

height="$("$ROOT/.venv/bin/python" -c '
import json, sys, urllib.request
with urllib.request.urlopen(sys.argv[1] + "/api/v1/status", timeout=30) as r:
    print(json.load(r)["latestBlockNumber"])
' "$NODE")"
echo "audit-campaign: height after install $height"
[[ "$height" -lt 10000 ]] || fail "expected a fresh chain, height is $height"

echo "audit-campaign: suites"
"$ROOT/.venv/bin/python" harness/run_suites.py 2>&1 | tee "$EVIDENCE/logs/unit-all.log"
cp deployments/suite-results.json "$EVIDENCE/results/suite-results.json"

echo "audit-campaign: CLI journey"
cargo run -q -p rshare-cli -- devnet journey --json \
  | tee "$EVIDENCE/results/client-journey.json"

echo "audit-campaign: browser journeys"
(
  cd web
  VITE_RSHARE_NODE="$NODE" npm run build
  RSHARE_NODE="$NODE" npx playwright test 2>&1 | tee "$EVIDENCE/logs/web-e2e.log"
)

echo "audit-campaign: offline gates"
make check 2>&1 | tee "$EVIDENCE/logs/check.log"
make client-check 2>&1 | tee "$EVIDENCE/logs/client-check.log"

git rev-parse HEAD > "$EVIDENCE/git-head.txt"
git status --short > "$EVIDENCE/git-status.txt"
sha256sum contracts/src/deploy/*.rho contracts/src/lib/*.rho \
  harness/run_suites.py crates/rshare-cli/src/main.rs > "$EVIDENCE/frozen-source.sha256"
sha256sum deployments/devnet.json deployments/catalogue.json > "$EVIDENCE/manifests.sha256"

echo "audit-campaign: pass; evidence at $EVIDENCE"
