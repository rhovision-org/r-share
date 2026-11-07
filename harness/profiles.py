"""Install and suite profiles.

Plain escrow is the safe default. The cover engine is an auditable experimental demo and is
only deployed by the explicit `cover-demo` profile.
"""

from __future__ import annotations

SAFE_TERMS = [
    "00_root.rho",
    "01_kernel.rho",
    "02_admin.rho",
    "40_payments.rho",
    "41_hold_deposit.rho",
    "03_terms.rho",
    "04_remedy.rho",
    "10_provider.rho",
    "13_unit.rho",
    "14_listing_contract.rho",
    "15_bond.rho",
    "34_booking_instance.rho",
    "22_booking_contract.rho",
    "23_guest_ledger.rho",
    "20_booking.rho",
]

COVER_DEMO_TERMS = [
    "00_root.rho",
    "01_kernel.rho",
    "02_admin.rho",
    "40_payments.rho",
    "41_hold_deposit.rho",
    "03_terms.rho",
    "04_remedy.rho",
    "30_cover_terms.rho",
    "31_cover_offer.rho",
    "33_cover_policy.rho",
    "32_cover_pool.rho",
    "10_provider.rho",
    "13_unit.rho",
    "14_listing_contract.rho",
    "15_bond.rho",
    "34_booking_instance.rho",
    "22_booking_contract.rho",
    "23_guest_ledger.rho",
    "24_guest_ledger_cover.rho",
    "20_booking.rho",
]

PROFILES = {
    "safe": SAFE_TERMS,
    "cover-demo": COVER_DEMO_TERMS,
}
DEFAULT_PROFILE = "safe"
EXPERIMENTAL_PROFILE = "cover-demo"

COVER_KEYS = [
    "component:cover-terms",
    "component:cover-book",
    "component:cover-pool",
    "component:cover-policy-factory",
    "component:guest-ledger-cover",
]
