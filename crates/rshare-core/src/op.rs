//! Operation identifiers.
//!
//! A guest booking id is `guestId:opId`. The same opId must identify the same user
//! intent across retries; a new intent gets a new opId.

use std::fmt;

use crate::error::{Result, RshareError};

/// A validated operation id.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpId(String);

impl OpId {
    /// Validate and wrap an operation id.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.len() <= 96
            && value
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "-_.:".contains(character));
        if valid {
            Ok(Self(value))
        } else {
            Err(RshareError::OpId(format!(
                "opId must be 1..=96 ASCII alphanumerics or -_.:, got {value:?}"
            )))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The booking id the guest ledger derives from this operation.
    pub fn booking_id(&self, guest_id: &str) -> Result<String> {
        if guest_id.is_empty() || guest_id.contains(':') || guest_id.chars().any(char::is_control) {
            return Err(RshareError::OpId(format!(
                "guest id is not a printable owner id: {guest_id:?}"
            )));
        }
        Ok(format!("{guest_id}:{}", self.0))
    }
}

impl fmt::Display for OpId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::str::FromStr for OpId {
    type Err = RshareError;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_and_composes() {
        let op = OpId::new("book-1700000000000-aabb").unwrap();
        assert_eq!(
            op.booking_id("11112VYAt8rUGNRRZX3eJdgagaAhtWTK8Js7F7X5iqddMVqyDTtYau")
                .unwrap(),
            "11112VYAt8rUGNRRZX3eJdgagaAhtWTK8Js7F7X5iqddMVqyDTtYau:book-1700000000000-aabb"
        );
    }

    #[test]
    fn refuses_unrepresentable_ids() {
        assert!(OpId::new("").is_err());
        assert!(OpId::new("has space").is_err());
        assert!(OpId::new("quote\"").is_err());
        assert!(OpId::new("x".repeat(97)).is_err());
    }
}
