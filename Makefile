PY ?= .venv/bin/python
MDBOOK ?= mdbook
RCASH_DIR ?= ../Rholphin
RHOLO_DIR ?= ../RNodeRust
NODE ?= http://localhost:40403

.PHONY: venv build build-check measure measure-check metrics metrics-compare \
        metrics-baseline check-arch check-captures check-select check-placeholders \
        check-money spec-check parse-check docs docs-serve docs-check devnet devnet-up \
        devnet-wait down status rcash-install rcash-bootstrap keys install setup unit-root unit-payments \
        unit-provider unit-listing unit-booking unit-concurrency unit-lifecycle \
        unit-claims unit-pricing unit-guestbook unit-events unit-admin unit-generality \
        unit-conservation unit-behavior unit-all \
        check ci clean

venv:
	python3 -m venv .venv
	$(PY) -m pip -q install --upgrade pip

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
            check-no-oracle pareto-check

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
	$(MAKE) -C $(RCASH_DIR) install RHOLO_DIR=$(abspath $(RHOLO_DIR))

rcash-bootstrap:
	$(PY) harness/rcash_bootstrap.py --rcash $(RCASH_DIR) --node $(NODE)

keys:
	$(PY) tools/keygen.py --out .devnet/keys.json

install:
	$(PY) harness/install.py --node $(NODE)

setup: devnet-up keys rcash-bootstrap install

unit-root:
	$(PY) harness/unit_root.py
unit-payments:
	$(PY) harness/unit_payments.py
unit-provider:
	$(PY) harness/unit_pending.py provider
unit-listing:
	$(PY) harness/unit_pending.py listing
unit-booking:
	$(PY) harness/unit_pending.py booking
unit-concurrency:
	$(PY) harness/unit_pending.py concurrency
unit-lifecycle:
	$(PY) harness/unit_lifecycle.py
unit-claims:
	$(PY) harness/unit_claims.py
unit-pricing:
	$(PY) harness/unit_pricing.py
unit-guestbook:
	$(PY) harness/unit_guestbook.py
unit-events:
	$(PY) harness/unit_pending.py events
unit-admin:
	$(PY) harness/unit_pending.py admin
unit-generality:
	$(PY) harness/unit_pending.py generality
unit-conservation:
	$(PY) harness/conservation.py
unit-behavior:
	$(PY) harness/unit_behavior.py

# The runner reports pass / fail / not-run and writes deployments/suite-results.json.
unit-all:
	$(PY) harness/run_suites.py

check: spec-check parse-check docs-check

ci: check unit-all

clean:
	rm -rf .venv __pycache__ harness/__pycache__ tools/__pycache__ contracts/build/* docs/book
