PY ?= .venv/bin/python
MDBOOK ?= mdbook
RCASH_DIR ?= ../Rholphin
RHOLO_DIR ?= ../RNodeRust
NODE ?= http://localhost:40403

.PHONY: venv build build-check measure measure-check check-arch check-captures \
        check-select check-placeholders check-money spec-check parse-check \
        docs docs-serve docs-check devnet down status \
        rcash-install keys install setup unit-root unit-payments unit-provider \
        unit-listing unit-booking unit-concurrency unit-lifecycle unit-events \
        unit-admin unit-generality unit-all ci clean

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

spec-check: build-check measure-check check-arch check-captures check-select \
            check-placeholders check-money

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

down:
	$(MAKE) -C $(RCASH_DIR) down RHOLO_DIR=$(abspath $(RHOLO_DIR))

status:
	@curl -s $(NODE)/api/v1/status | $(PY) -m json.tool || echo "devnet not running"

rcash-install:
	$(MAKE) -C $(RCASH_DIR) install RHOLO_DIR=$(abspath $(RHOLO_DIR))

keys:
	$(PY) tools/keygen.py --out .devnet/keys.json

install:
	$(PY) harness/install.py --node $(NODE)

setup: devnet keys rcash-install install

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
	$(PY) harness/unit_pending.py lifecycle
unit-events:
	$(PY) harness/unit_pending.py events
unit-admin:
	$(PY) harness/unit_pending.py admin
unit-generality:
	$(PY) harness/unit_pending.py generality

# Only the suites implemented so far run in the campaign. The pending targets exit 2.
unit-all: unit-root unit-payments

ci: spec-check unit-all

clean:
	rm -rf .venv __pycache__ harness/__pycache__ tools/__pycache__ contracts/build/* docs/book
