//! The r-share Root's component keys.

use std::fmt;
use std::str::FromStr;

/// A component registered in the r-share Root.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ComponentKey {
    Root,
    Clock,
    Verifier,
    Codec,
    AdminGuard,
    ShareAdmin,
    Payments,
    Terms,
    Remedy,
    ProviderBook,
    UnitFactory,
    ListingFactory,
    BondFactory,
    BookingFactory,
    GuestLedgerFactory,
    GuestBook,
}

impl ComponentKey {
    /// Every key the installer registers. `doctor` resolves all of them.
    pub const ALL: [ComponentKey; 16] = [
        Self::Root,
        Self::Clock,
        Self::Verifier,
        Self::Codec,
        Self::AdminGuard,
        Self::ShareAdmin,
        Self::Payments,
        Self::Terms,
        Self::Remedy,
        Self::ProviderBook,
        Self::UnitFactory,
        Self::ListingFactory,
        Self::BondFactory,
        Self::BookingFactory,
        Self::GuestLedgerFactory,
        Self::GuestBook,
    ];

    /// The full registry key, including the `component:` prefix.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Root => "component:root",
            Self::Clock => "component:clock",
            Self::Verifier => "component:verifier",
            Self::Codec => "component:codec",
            Self::AdminGuard => "component:admin-guard",
            Self::ShareAdmin => "component:admin",
            Self::Payments => "component:payments",
            Self::Terms => "component:terms",
            Self::Remedy => "component:remedy",
            Self::ProviderBook => "component:provider-book",
            Self::UnitFactory => "component:unit-factory",
            Self::ListingFactory => "component:listing-factory",
            Self::BondFactory => "component:bond-factory",
            Self::BookingFactory => "component:booking-factory",
            Self::GuestLedgerFactory => "component:guest-ledger-factory",
            Self::GuestBook => "component:guest-book",
        }
    }

    /// Parse a full key or its short suffix (`component:terms` or `terms`).
    pub fn parse(value: &str) -> Option<Self> {
        let short = value.strip_prefix("component:").unwrap_or(value);
        Self::ALL
            .into_iter()
            .find(|key| key.as_str() == format!("component:{short}"))
    }
}

impl fmt::Display for ComponentKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ComponentKey {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        Self::parse(value).ok_or_else(|| format!("unknown component key: {value}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_round_trips() {
        for key in ComponentKey::ALL {
            assert_eq!(ComponentKey::parse(key.as_str()), Some(key));
            assert_eq!(key.to_string().parse::<ComponentKey>().unwrap(), key);
        }
    }

    #[test]
    fn short_names_are_accepted() {
        assert_eq!(ComponentKey::parse("terms"), Some(ComponentKey::Terms));
        assert_eq!(ComponentKey::parse("nope"), None);
    }
}
