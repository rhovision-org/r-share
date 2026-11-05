//! On-chain status values.
//!
//! Unknown values are preserved rather than rejected so an older client can display a
//! newer contract state without pretending it understands it.

use std::str::FromStr;

macro_rules! status_enum {
    ($name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub enum $name {
            $($variant),+,
            Unknown(String),
        }

        impl $name {
            pub fn as_str(&self) -> &str {
                match self {
                    $(Self::$variant => $text),+,
                    Self::Unknown(value) => value,
                }
            }

            pub fn parse(value: &str) -> Self {
                match value {
                    $($text => Self::$variant),+,
                    other => Self::Unknown(other.to_string()),
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = std::convert::Infallible;

            fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
                Ok(Self::parse(value))
            }
        }
    };
}

status_enum!(ListingStatus {
    Draft => "draft",
    Active => "active",
    Paused => "paused",
    Delisted => "delisted",
});

status_enum!(UnitStatus {
    Active => "active",
    Paused => "paused",
});

status_enum!(ProviderStatus {
    Active => "active",
    Paused => "paused",
});

status_enum!(BookingStatus {
    Requested => "requested",
    Confirmed => "confirmed",
    CheckedIn => "checkedIn",
    CheckedOut => "checkedOut",
    Redeemed => "redeemed",
    Settled => "settled",
    Rejected => "rejected",
    Cancelled => "cancelled",
    Expired => "expired",
    NoShow => "noShow",
    NoShowClaimed => "noShowClaimed",
    NoShowDisputed => "noShowDisputed",
    NoShowRefunded => "noShowRefunded",
    Disputed => "disputed",
});

impl BookingStatus {
    /// Terminal statuses have no further lifecycle action.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Settled
                | Self::Rejected
                | Self::Cancelled
                | Self::Expired
                | Self::NoShow
                | Self::NoShowRefunded
        )
    }
}

/// The public state of one slot, as `probe` returns it (release tokens redacted).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SlotState {
    Free {
        generation: i64,
    },
    Held {
        booking_id: String,
        until_height: i64,
    },
    Booked {
        booking_id: String,
    },
    /// A record the client does not understand. Never rendered as free.
    Invalid,
}

impl SlotState {
    /// A display/JSON projection. Release tokens are never included.
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::Free { generation } => serde_json::json!(["free", generation]),
            Self::Held {
                booking_id,
                until_height,
            } => serde_json::json!(["held", booking_id, until_height]),
            Self::Booked { booking_id } => serde_json::json!(["booked", booking_id]),
            Self::Invalid => serde_json::json!(["invalid"]),
        }
    }

    pub fn is_free(&self) -> bool {
        matches!(self, Self::Free { .. })
    }

    pub fn is_held(&self) -> bool {
        matches!(self, Self::Held { .. })
    }

    pub fn is_booked(&self) -> bool {
        matches!(self, Self::Booked { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_round_trip_and_preserve_unknown_values() {
        assert_eq!(ListingStatus::parse("active"), ListingStatus::Active);
        assert_eq!(BookingStatus::parse("noShow").to_string(), "noShow");
        assert_eq!(
            BookingStatus::parse("futureState"),
            BookingStatus::Unknown("futureState".into())
        );
        assert_eq!(
            "futureState".parse::<BookingStatus>().unwrap().as_str(),
            "futureState"
        );
    }

    #[test]
    fn terminality_matches_the_lifecycle() {
        assert!(BookingStatus::Settled.is_terminal());
        assert!(BookingStatus::Cancelled.is_terminal());
        assert!(BookingStatus::NoShowRefunded.is_terminal());
        assert!(!BookingStatus::Redeemed.is_terminal());
        assert!(!BookingStatus::NoShowClaimed.is_terminal());
        assert!(!BookingStatus::NoShowDisputed.is_terminal());
        assert!(!BookingStatus::CheckedOut.is_terminal());
        assert!(!BookingStatus::Disputed.is_terminal());
        assert!(!BookingStatus::Requested.is_terminal());
    }
}
