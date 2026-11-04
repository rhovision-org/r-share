//! SDK errors and refusal classification.

use rcash_chain::ChainError;

use rshare_core::RshareError;

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
            "slot held" => Self::SlotHeld,
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
    Decode(String),
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
}

impl From<rcash_core::CoreError> for SdkError {
    fn from(error: rcash_core::CoreError) -> Self {
        Self::Core(RshareError::Rcash(error))
    }
}

/// The SDK result type.
pub type Result<T> = std::result::Result<T, SdkError>;
