//! Pure r-share client types.
//!
//! This crate has no I/O, keys, network or clock. It turns user intent into validated
//! values and Rholang source fragments; every chain call and every signature lives in
//! `rshare-sdk`.

pub mod component;
pub mod error;
pub mod manifest;
pub mod op;
pub mod primitives;
pub mod render;
pub mod slots;
pub mod status;
pub mod terms;

pub use component::ComponentKey;
pub use error::{Result, RshareError};
pub use manifest::Manifest;
pub use op::OpId;
pub use primitives::{Amount, BasisPoints, BlockCount, Height, Nonce, OwnerId};
pub use rcash_core::assets::RegistryUri;
pub use slots::{SlotSpec, UnitSlotRequest};
pub use status::{BookingStatus, ListingStatus, ProviderStatus, SlotState, UnitStatus};
pub use terms::{ResourceKind, Terms, TermsError};

/// The project's per-term budget. The node accepts 64 KiB inclusive; r-share keeps a
/// 8 KiB margin for future template and quoting growth.
pub const PROJECT_TERM_BUDGET: usize = 56 * 1024;
