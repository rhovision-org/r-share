//! SDK errors and refusal classification.

use std::fmt;

use rcash_chain::ChainError;

use rshare_core::RshareError;

/// Why a projection record could not be decoded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeKind {
    /// The record is missing a field the contract always writes.
    Missing,
    /// A field is present but is not the expected Rholang type.
    WrongType,
    /// A field has the right type but violates the domain bounds.
    OutOfRange,
    /// A tagged value carries a tag this client does not know.
    UnknownTag,
    /// The record declares a schema version this client does not support.
    UnsupportedSchema,
}

impl fmt::Display for DecodeKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Missing => "missing",
            Self::WrongType => "wrong type",
            Self::OutOfRange => "out of range",
            Self::UnknownTag => "unknown tag",
            Self::UnsupportedSchema => "unsupported schema",
        })
    }
}

/// A structured projection-decode failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodeError {
    /// The record being decoded, for example `provider info`.
    pub context: String,
    /// The field (or nested field path) that failed.
    pub field: String,
    /// The failure category.
    pub kind: DecodeKind,
}

impl DecodeError {
    pub fn new(context: impl Into<String>, field: impl Into<String>, kind: DecodeKind) -> Self {
        Self {
            context: context.into(),
            field: field.into(),
            kind,
        }
    }

    pub fn missing(context: impl Into<String>, field: impl Into<String>) -> Self {
        Self::new(context, field, DecodeKind::Missing)
    }

    pub fn wrong_type(context: impl Into<String>, field: impl Into<String>) -> Self {
        Self::new(context, field, DecodeKind::WrongType)
    }

    pub fn out_of_range(context: impl Into<String>, field: impl Into<String>) -> Self {
        Self::new(context, field, DecodeKind::OutOfRange)
    }

    pub fn unknown_tag(context: impl Into<String>, field: impl Into<String>) -> Self {
        Self::new(context, field, DecodeKind::UnknownTag)
    }

    pub fn unsupported_schema(context: impl Into<String>, field: impl Into<String>) -> Self {
        Self::new(context, field, DecodeKind::UnsupportedSchema)
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: field `{}` is {}",
            self.context, self.field, self.kind
        )
    }
}

impl std::error::Error for DecodeError {}

/// The published refusal vocabulary, grouped for the UI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefusalCode {
    NotOwner,
    Closed,
    Replay,
    Pending,
    SlotHeld,
    SlotLost,
    UnknownSlot,
    UnknownUnit,
    UnknownListing,
    Removed,
    NotActive,
    Paused,
    ProviderPaused,
    UnitPaused,
    Terms,
    Slots,
    Schedule,
    Window,
    Price,
    Bps,
    Threshold,
    Amount,
    Kind,
    Asset,
    PaymentsUnavailable,
    Account,
    BondFactoryUnavailable,
    UnitFactoryUnavailable,
    Codec,
    Clock,
    TooEarly,
    TooLate,
    BelowThreshold,
    Reservation,
    AdminKey,
    Other(String),
}

impl RefusalCode {
    /// Classify a contract refusal string.
    pub fn classify(reason: &str) -> Self {
        match reason {
            "not owner" | "not provider" | "not guest" | "not party" => Self::NotOwner,
            "closed" => Self::Closed,
            "replay" => Self::Replay,
            "pending" => Self::Pending,
            "slot held" | "slot booked" => Self::SlotHeld,
            "slot lost" => Self::SlotLost,
            "unknown slot" => Self::UnknownSlot,
            "unknown unit" | "duplicate unit" => Self::UnknownUnit,
            "unknown listing" | "no provider" | "no ledger" | "no account" => Self::UnknownListing,
            "removed" => Self::Removed,
            "not active" | "listing inactive" => Self::NotActive,
            "paused" => Self::Paused,
            "provider paused" => Self::ProviderPaused,
            "unit paused" => Self::UnitPaused,
            "terms" | "refund window" | "instant" | "contiguous" => Self::Terms,
            "slots" | "units" | "duplicate slot" | "slot id" | "slot order" => Self::Slots,
            "schedule" | "slot past" | "slot lead" | "slot duration" | "slot gap" => Self::Schedule,
            "window" => Self::Window,
            "price" => Self::Price,
            "bps" => Self::Bps,
            "threshold" => Self::Threshold,
            "duplicate key" | "key" => Self::AdminKey,
            "reservation" => Self::Reservation,
            "listing" => Self::UnknownListing,
            "amount" => Self::Amount,
            "kind" => Self::Kind,
            "asset" => Self::Asset,
            "payments unavailable" | "rcash unavailable" => Self::PaymentsUnavailable,
            "account" => Self::Account,
            "bond factory unavailable" => Self::BondFactoryUnavailable,
            "unit factory unavailable" | "listing factory unavailable" => {
                Self::UnitFactoryUnavailable
            }
            "codec" => Self::Codec,
            "clock" => Self::Clock,
            "too early" => Self::TooEarly,
            "too late" => Self::TooLate,
            "below threshold" => Self::BelowThreshold,
            other => Self::Other(other.to_string()),
        }
    }
}

/// An SDK error.
#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    #[error(transparent)]
    Chain(#[from] ChainError),
    #[error(transparent)]
    Core(#[from] RshareError),
    #[error("decode error: {0}")]
    Decode(#[from] DecodeError),
    #[error("random source failed: {0}")]
    Entropy(String),
    #[error("refused: {reason}")]
    Refused { reason: String, code: RefusalCode },
    #[error("deploy failed: {error}")]
    DeployFailed {
        error: String,
        block_hash: Option<String>,
    },
    #[error("deploy {deploy_id} did not settle: {last}")]
    Unknown { deploy_id: String, last: String },
    #[error("wallet is locked")]
    NoKey,
    #[error("term is {bytes} bytes; the budget is {budget}")]
    TermTooLarge { bytes: usize, budget: usize },
}

impl SdkError {
    pub fn refused(reason: impl Into<String>) -> Self {
        let reason = reason.into();
        let code = RefusalCode::classify(&reason);
        Self::Refused { reason, code }
    }

    /// Build a structured decode error.
    pub fn decode(context: impl Into<String>, field: impl Into<String>, kind: DecodeKind) -> Self {
        Self::Decode(DecodeError::new(context, field, kind))
    }
}

impl From<rcash_core::CoreError> for SdkError {
    fn from(error: rcash_core::CoreError) -> Self {
        Self::Core(RshareError::Rcash(error))
    }
}

/// The SDK result type.
pub type Result<T> = std::result::Result<T, SdkError>;
