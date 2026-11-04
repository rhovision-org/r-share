PY ?= .venv/bin/python
MDBOOK ?= mdbook
RCASH_DIR ?= ../Rholphin
RHOLO_DIR ?= ../RNodeRust
NODE ?= http://localhost:40403

# Make every recipe and its Python children use the same absolute checkout. CI sets
# RCASH_DIR to its `rholo` checkout, so no sibling named Rholphin exists there.
export RCASH_DIR := $(abspath $(RCASH_DIR))
export RHOLO_DIR := $(abspath $(RHOLO_DIR))

.PHONY: venv build build-check measure measure-check metrics metrics-compare \
        metrics-baseline check-arch check-captures check-select check-placeholders \
        check-money check-suites check-allocator spec-check parse-check docs docs-serve docs-check devnet devnet-up \
        devnet-wait down status rcash-install rcash-keys rcash-bootstrap keys install setup unit-root unit-payments \
        unit-provider unit-listing unit-booking unit-allocator unit-lifecycle \
        unit-claims unit-pricing unit-guestbook unit-events unit-admin unit-pause \
        unit-generality unit-conservation unit-behavior unit-all \
        check ci clean lean-build lean-check vectors vectors-write check-flows formal-check \
        client-lint client-test client-wasm client-wasm-smoke client-check \
        client-fixtures client-journey client-keystores \
        web-install web-test web-build web-e2e

venv:
	python3 -m venv .venv
	$(PY) -m pip -q install --upgrade pip
	$(PY) -m pip -q install -r harness/requirements.txt

build:
	$(PY) tools/build_contracts.py

build-check:
	$(PY) tools/build_contracts.py --check

measure:
	$(PY) tools/measure_contracts.py --write

measure-check:
	$(PY) tools/measure_contracts.py --check

metrics:
	$(PY) tools/metrics.py --write contracts/metrics-current.json

metrics-compare:
	$(PY) tools/metrics.py --compare contracts/metrics-baseline.json

metrics-baseline:
	$(PY) tools/metrics.py --write-baseline

check-arch:
	$(PY) tools/check_arch.py

check-captures:
	$(PY) tools/check_captures.py

check-select:
	$(PY) tools/check_select.py

check-placeholders:
	$(PY) tools/check_placeholders.py

check-money:
	$(PY) tools/check_money.py

check-coverage:
	$(PY) tools/check_coverage.py

check-suites:
	$(PY) tools/check_suites.py

check-allocator:
	$(PY) tools/check_allocator.py

check-remedy:
	$(PY) tools/check_remedy_table.py

check-no-oracle:
	$(PY) tools/check_no_oracle.py

pareto:
	$(PY) tools/pareto.py --write

pareto-check:
	$(PY) tools/pareto.py --check

spec-check: build-check measure-check metrics-compare check-arch check-captures \
            check-select check-placeholders check-money check-coverage check-remedy \
            check-no-oracle check-suites check-allocator pareto-check

parse-check:
	$(PY) harness/install.py --dry-run
	tools/parse_check.sh contracts/build/*.rho .devnet/rendered/*.rho

docs:
	$(MDBOOK) build

docs-serve:
	$(MDBOOK) serve --open

docs-check:
	$(PY) tools/check_docs.py
	$(MDBOOK) build

devnet:
	$(MAKE) -C $(RCASH_DIR) devnet RHOLO_DIR=$(abspath $(RHOLO_DIR))

devnet-up:
	-$(MAKE) -C $(RCASH_DIR) down RHOLO_DIR=$(abspath $(RHOLO_DIR))
	$(MAKE) -C $(RCASH_DIR) devnet RHOLO_DIR=$(abspath $(RHOLO_DIR))
	$(PY) harness/preflight.py --wait 300

devnet-wait:
	$(PY) harness/preflight.py --wait 300

down:
	$(MAKE) -C $(RCASH_DIR) down RHOLO_DIR=$(abspath $(RHOLO_DIR))

status:
	@curl -s $(NODE)/api/v1/status | $(PY) -m json.tool || echo "devnet not running"

rcash-install:
	$(MAKE) -C $(RCASH_DIR) install RHOLO_DIR=$(RHOLO_DIR)

rcash-keys:
	$(MAKE) -C $(RCASH_DIR) keys RHOLO_DIR=$(RHOLO_DIR)

rcash-bootstrap:
	$(PY) harness/rcash_bootstrap.py --rcash $(RCASH_DIR) --node $(NODE)

keys:
	$(PY) tools/keygen.py --out .devnet/keys.json \
		--from-path $(RCASH_DIR)/.devnet/keys.json

install:
	$(PY) harness/install.py --node $(NODE)

setup: devnet-up rcash-keys rcash-bootstrap keys install

unit-root:
	$(PY) harness/unit_root.py
unit-payments:
	$(PY) harness/unit_payments.py
unit-provider:
	$(PY) harness/unit_provider.py
unit-listing:
	$(PY) harness/unit_listing.py
unit-booking:
	$(PY) harness/unit_booking.py
unit-allocator:
	$(PY) harness/unit_allocator.py
unit-lifecycle:
	$(PY) harness/unit_lifecycle.py
unit-claims:
	$(PY) harness/unit_claims.py
unit-pricing:
	$(PY) harness/unit_pricing.py
unit-guestbook:
	$(PY) harness/unit_guestbook.py
unit-pause:
	$(PY) harness/unit_pause.py
unit-events:
	$(PY) harness/unit_events.py
unit-admin:
	$(PY) harness/unit_admin.py
unit-generality:
	$(PY) harness/unit_generality.py
unit-conservation:
	$(PY) harness/unit_conservation.py
unit-behavior:
	$(PY) harness/unit_behavior.py

# The runner reports pass / fail / not-run and writes deployments/suite-results.json.
unit-all:
	$(PY) harness/run_suites.py

lean-build:
	cd spec/lean && lake build

lean-check:
	$(PY) tools/check_lean.py

vectors-write:
	$(PY) tools/gen_vectors.py --write

vectors:
	$(PY) tools/check_vectors.py

check-flows:
	$(PY) tools/check_flows.py

formal-check: lean-check vectors check-flows

# ---- reference client ---------------------------------------------------------------

client-lint:
	cargo fmt --all -- --check
	cargo clippy -p rshare-core -p rshare-keystore -p rshare-sdk -p rshare-cli --all-targets -- -D warnings

client-test:
	cargo test -p rshare-core -p rshare-keystore -p rshare-sdk

client-wasm:
	cargo check -p rshare-wasm --target wasm32-unknown-unknown

web-install:
	cd web && npm ci --no-audit --no-fund

client-wasm-smoke: web-install
	cd web && npm run test:wasm

client-check: client-lint client-test client-wasm client-wasm-smoke
	cd web && npm run typecheck && npm test && npm run build

client-fixtures:
	cargo run -q -p rshare-cli -- devnet fixtures

client-journey:
	cargo run -q -p rshare-cli -- devnet journey --json

client-keystores:
	mkdir -p .devnet/keystores
	cargo run -q -p rshare-cli -- keystore encrypt --name admin1 --password e2e-admin-password --out .devnet/keystores/admin1.json
	cargo run -q -p rshare-cli -- keystore encrypt --name admin2 --password e2e-admin-password --out .devnet/keystores/admin2.json

web-test:
	cd web && npm test

web-build: web-install
	cd web && npm run build

web-e2e: web-install client-keystores
	cd web && npm run test:e2e

check: spec-check parse-check docs-check formal-check

ci: check unit-all

clean:
	rm -rf .venv __pycache__ harness/__pycache__ tools/__pycache__ contracts/build/* docs/book
