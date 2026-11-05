//! Validated chain-domain scalars.
//!
//! These types exist so a raw `i64` from a JSON document or a CLI argument cannot reach a
//! term, a signature or a comparison without a constructor check. The JSON projection
//! boundary is controlled separately in `rshare-sdk`; this module only guarantees the
//! in-memory invariant.

use std::fmt;
use std::str::FromStr;

use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

use crate::error::{Result, RshareError};

macro_rules! non_negative {
    ($name:ident, $field:literal) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(i64);

        impl $name {
            pub const ZERO: Self = Self(0);

            pub fn new(value: i64) -> Result<Self> {
                if value < 0 {
                    return Err(RshareError::Value {
                        field: $field,
                        reason: "must not be negative",
                    });
                }
                Ok(Self(value))
            }

            pub fn get(self) -> i64 {
                self.0
            }

            pub fn checked_add(self, other: i64) -> Option<Self> {
                self.0
                    .checked_add(other)
                    .and_then(|value| Self::new(value).ok())
            }

            pub fn checked_sub(self, other: Self) -> Option<Self> {
                self.0
                    .checked_sub(other.0)
                    .and_then(|value| Self::new(value).ok())
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }

        impl FromStr for $name {
            type Err = RshareError;

            fn from_str(value: &str) -> Result<Self> {
                let parsed = value.parse::<i64>().map_err(|_| RshareError::Value {
                    field: $field,
                    reason: "not a decimal integer",
                })?;
                Self::new(parsed)
            }
        }

        impl From<$name> for i64 {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl TryFrom<i64> for $name {
            type Error = RshareError;

            fn try_from(value: i64) -> Result<Self> {
                Self::new(value)
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(
                &self,
                serializer: S,
            ) -> std::result::Result<S::Ok, S::Error> {
                serializer.serialize_i64(self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(
                deserializer: D,
            ) -> std::result::Result<Self, D::Error> {
                let value = i64::deserialize(deserializer)?;
                Self::new(value).map_err(de::Error::custom)
            }
        }
    };
}

non_negative!(Amount, "amount");
non_negative!(Height, "height");
non_negative!(Nonce, "nonce");
non_negative!(BlockCount, "block count");

impl Amount {
    pub fn checked_sum(self, other: Self) -> Option<Self> {
        self.checked_add(other.0)
    }
}

impl Height {
    /// The number of blocks from `self` to `end`, or `None` when `end` is earlier.
    pub fn until(self, end: Self) -> Option<BlockCount> {
        BlockCount::new(end.0.checked_sub(self.0)?).ok()
    }
}

impl Nonce {
    /// The next nonce, or `None` at `i64::MAX`.
    pub fn next(self) -> Option<Self> {
        self.checked_add(1)
    }
}

/// Basis points, bounded to `0..=10_000`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BasisPoints(u16);

impl BasisPoints {
    pub const ZERO: Self = Self(0);
    pub const MAX: u16 = 10_000;

    pub fn new(value: u16) -> Result<Self> {
        if value > Self::MAX {
            return Err(RshareError::Value {
                field: "basis points",
                reason: "must be at most 10000",
            });
        }
        Ok(Self(value))
    }

    pub fn get(self) -> u16 {
        self.0
    }

    /// Apply the rate to an amount, rounding down. `10_000` returns the amount.
    pub fn apply(self, amount: Amount) -> Result<Amount> {
        let product = i128::from(amount.get()) * i128::from(self.0);
        let part = product / 10_000;
        let part = i64::try_from(part).map_err(|_| RshareError::Value {
            field: "basis points",
            reason: "share does not fit the amount range",
        })?;
        Amount::new(part)
    }
}

impl From<BasisPoints> for i64 {
    fn from(value: BasisPoints) -> Self {
        i64::from(value.0)
    }
}

impl fmt::Display for BasisPoints {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl FromStr for BasisPoints {
    type Err = RshareError;

    fn from_str(value: &str) -> Result<Self> {
        let parsed = value.parse::<u16>().map_err(|_| RshareError::Value {
            field: "basis points",
            reason: "not a decimal integer",
        })?;
        Self::new(parsed)
    }
}

impl Serialize for BasisPoints {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_u16(self.0)
    }
}

impl<'de> Deserialize<'de> for BasisPoints {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value = u16::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

/// A REV owner identifier derived from a deploy key. The node emits lowercase hex; this
/// type rejects empty, overlong and non-alphanumeric values.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OwnerId(String);

impl OwnerId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.len() <= 200
            && value
                .chars()
                .all(|character| character.is_ascii_alphanumeric());
        if !valid {
            return Err(RshareError::Value {
                field: "owner id",
                reason: "must be 1..=200 ASCII alphanumerics",
            });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OwnerId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for OwnerId {
    type Err = RshareError;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl Serialize for OwnerId {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for OwnerId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amounts_reject_negative_and_check_overflow() {
        assert!(Amount::new(-1).is_err());
        assert_eq!(
            Amount::new(3)
                .unwrap()
                .checked_sum(Amount::new(4).unwrap())
                .unwrap()
                .get(),
            7
        );
        assert!(Amount::new(i64::MAX).unwrap().checked_add(1).is_none());
    }

    #[test]
    fn heights_and_nonces_are_monotonic() {
        let start = Height::new(10).unwrap();
        let end = Height::new(15).unwrap();
        assert_eq!(start.until(end).unwrap().get(), 5);
        assert!(end.until(start).is_none());
        let nonce = Nonce::new(41).unwrap();
        assert_eq!(nonce.next().unwrap().get(), 42);
        assert!(Nonce::new(i64::MAX).unwrap().next().is_none());
    }

    #[test]
    fn basis_points_are_bounded() {
        assert!(BasisPoints::new(10_001).is_err());
        let amount = Amount::new(1_000).unwrap();
        assert_eq!(
            BasisPoints::new(250).unwrap().apply(amount).unwrap().get(),
            25
        );
        assert_eq!(
            BasisPoints::new(10_000)
                .unwrap()
                .apply(amount)
                .unwrap()
                .get(),
            1_000
        );
    }

    #[test]
    fn owner_ids_reject_malformed_values() {
        assert!(OwnerId::new("").is_err());
        assert!(OwnerId::new("abc-def").is_err());
        assert_eq!(OwnerId::new("04f7aa").unwrap().as_str(), "04f7aa");
    }

    #[test]
    fn json_rejects_negative_values() {
        assert!(serde_json::from_str::<Amount>("-1").is_err());
        assert_eq!(serde_json::from_str::<Amount>("7").unwrap().get(), 7);
        assert!(serde_json::from_str::<BasisPoints>("10001").is_err());
    }
}
