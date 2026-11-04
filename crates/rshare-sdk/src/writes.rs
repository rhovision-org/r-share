//! Typed writes.
//!
//! Every write is one signed deploy. Success requires a finalized block and a
//! `(true, ...)` reply; a `(false, reason)` is surfaced as a designed refusal.

use std::collections::BTreeMap;

use rcash_chain::rho_expr::RhoValue;
use serde_json::Value;

use rshare_core::render::{check_project_size, quote_string, render_json};
use rshare_core::slots::{render_slot_specs, render_unit_slots, validate_unit_slots};
use rshare_core::{
    ComponentKey, OpId, RegistryUri, ResourceKind, SlotSpec, Terms, UnitSlotRequest,
};

use crate::error::{Result, SdkError};
use crate::models::{BookingView, ListingView};
use crate::wire::{self, DeployReceipt, Read};
use crate::Client;

/// A successful write's typed value and receipt.
#[derive(Clone, Debug, PartialEq)]
pub struct Write<T> {
    pub value: T,
    pub receipt: DeployReceipt,
}

/// A booking created by the guest ledger.
#[derive(Clone, Debug, PartialEq)]
pub struct BookOutcome {
    pub booking_id: String,
    pub booking_uri: String,
    pub ledger_uri: String,
    pub guest_id: String,
    pub receipt: DeployReceipt,
}

/// A lifecycle action. `args()` renders the arm and its data arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BookingAction {
    Accept,
    Reject { reason: String },
    Cancel,
    CancelByProvider,
    CheckIn,
    CheckOut,
    GuestCheckOut,
    Settle,
    ClaimDeposit { amount: i64, evidence_hash: String },
    ResolveDeposit,
    Dispute { reason_hash: String },
    ResolveDispute,
    Expire,
    ClaimNoShow,
    Redeem,
    CloseEvent,
    TransferTicket { new_guest_id: String },
}

impl BookingAction {
    /// The arm and its arguments, without the reply channel.
    pub fn arm_and_args(&self, op: &OpId) -> Result<(&'static str, Vec<String>)> {
        let op = quote_string(op.as_str())?;
        Ok(match self {
            Self::Accept => ("accept", vec![owner(), op]),
            Self::Reject { reason } => ("reject", vec![owner(), op, quote_string(reason)?]),
            Self::Cancel => ("cancel", vec![owner(), op]),
            Self::CancelByProvider => ("cancelByProvider", vec![owner(), op]),
            Self::CheckIn => ("checkIn", vec![owner(), op]),
            Self::CheckOut => ("checkOut", vec![owner(), op]),
            Self::GuestCheckOut => ("guestCheckOut", vec![owner(), op]),
            Self::Settle => ("settle", vec![op]),
            Self::ClaimDeposit {
                amount,
                evidence_hash,
            } => (
                "claimDeposit",
                vec![
                    owner(),
                    op,
                    amount.to_string(),
                    quote_string(evidence_hash)?,
                ],
            ),
            Self::ResolveDeposit => ("resolveDeposit", vec![op]),
            Self::Dispute { reason_hash } => {
                ("dispute", vec![owner(), op, quote_string(reason_hash)?])
            }
            Self::ResolveDispute => ("resolveDispute", vec![op]),
            Self::Expire => ("expire", vec![op]),
            Self::ClaimNoShow => ("claimNoShow", vec![owner(), op]),
            Self::Redeem => ("redeem", vec![owner(), op]),
            Self::CloseEvent => ("closeEvent", vec![op]),
            Self::TransferTicket { new_guest_id } => (
                "transferTicket",
                vec![owner(), quote_string(new_guest_id)?, op],
            ),
        })
    }
}

/// A term preview; no signature, no phlo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preview {
    pub term: String,
    pub bytes: usize,
}

impl Preview {
    fn new(term: String) -> Result<Self> {
        check_project_size(&term)?;
        let bytes = term.len();
        Ok(Self { term, bytes })
    }
}

impl Client {
    async fn submit_uri(
        &self,
        uri: &RegistryUri,
        arm: &str,
        args: &[String],
    ) -> Result<(Vec<RhoValue>, DeployReceipt)> {
        let term = wire::call_term_signed(uri, arm, args)?;
        let receipt = wire::deploy_receipt(&self.inner, &term).await?;
        let reply = receipt
            .reply
            .clone()
            .ok_or_else(|| SdkError::Decode(format!("{arm} returned no value")))?;
        let values = wire::decode_reply(&reply)?;
        Ok((values, receipt))
    }

    /// `ProviderBook("register")`.
    pub async fn register_provider(&self, profile: &Value) -> Result<Write<(String, String)>> {
        let component = self.resolve(ComponentKey::ProviderBook).await?;
        let args = vec![owner(), render_json(profile)?];
        let (values, receipt) = self.submit_uri(&component.uri, "register", &args).await?;
        let provider_uri = text_at(&values, 0, "provider uri")?;
        let owner_id = text_at(&values, 1, "owner id")?;
        Ok(Write {
            value: (provider_uri, owner_id),
            receipt,
        })
    }

    /// `ProviderContract("updateProfile")`.
    pub async fn update_profile(
        &self,
        provider_uri: &RegistryUri,
        profile: &Value,
        op: &OpId,
    ) -> Result<Write<()>> {
        let args = vec![owner(), render_json(profile)?, quote_string(op.as_str())?];
        let (_values, receipt) = self
            .submit_uri(provider_uri, "updateProfile", &args)
            .await?;
        Ok(Write { value: (), receipt })
    }

    pub async fn provider_pause(&self, provider_uri: &RegistryUri) -> Result<Write<()>> {
        let (_values, receipt) = self.submit_uri(provider_uri, "pause", &[owner()]).await?;
        Ok(Write { value: (), receipt })
    }

    pub async fn provider_resume(&self, provider_uri: &RegistryUri) -> Result<Write<()>> {
        let (_values, receipt) = self.submit_uri(provider_uri, "resume", &[owner()]).await?;
        Ok(Write { value: (), receipt })
    }

    /// `ListingFactory("create")`.
    pub async fn create_listing(
        &self,
        provider_uri: &RegistryUri,
        listing_id: &str,
        kind: ResourceKind,
        terms: &Terms,
    ) -> Result<Write<String>> {
        let component = self.resolve(ComponentKey::ListingFactory).await?;
        let args = vec![
            owner(),
            quote_string(provider_uri.as_str())?,
            quote_string(listing_id)?,
            quote_string(kind.as_str())?,
            terms.to_rho_map()?,
        ];
        let (values, receipt) = self.submit_uri(&component.uri, "create", &args).await?;
        let listing_uri = text_at(&values, 0, "listing uri")?;
        Ok(Write {
            value: listing_uri,
            receipt,
        })
    }

    /// A preview of `ListingFactory("create")`.
    pub async fn preview_create_listing(
        &self,
        provider_uri: &RegistryUri,
        listing_id: &str,
        kind: ResourceKind,
        terms: &Terms,
    ) -> Result<Preview> {
        let component = self.resolve(ComponentKey::ListingFactory).await?;
        let args = vec![
            owner(),
            quote_string(provider_uri.as_str())?,
            quote_string(listing_id)?,
            quote_string(kind.as_str())?,
            terms.to_rho_map()?,
        ];
        Preview::new(wire::call_term_signed(&component.uri, "create", &args)?)
    }

    /// `ListingContract("ensureUnit")`.
    pub async fn ensure_unit(
        &self,
        listing_uri: &RegistryUri,
        unit_id: &str,
        attrs: &Value,
        slots: &[SlotSpec],
        price_per_slot: i64,
    ) -> Result<Write<String>> {
        let args = vec![
            owner(),
            quote_string(unit_id)?,
            render_json(attrs)?,
            render_slot_specs(slots)?,
            price_per_slot.to_string(),
        ];
        let (values, receipt) = self.submit_uri(listing_uri, "ensureUnit", &args).await?;
        let unit = text_at(&values, 0, "unit id")?;
        Ok(Write {
            value: unit,
            receipt,
        })
    }

    /// A preview of `ListingContract("ensureUnit")`.
    pub async fn preview_ensure_unit(
        &self,
        listing_uri: &RegistryUri,
        unit_id: &str,
        attrs: &Value,
        slots: &[SlotSpec],
        price_per_slot: i64,
    ) -> Result<Preview> {
        let args = vec![
            owner(),
            quote_string(unit_id)?,
            render_json(attrs)?,
            render_slot_specs(slots)?,
            price_per_slot.to_string(),
        ];
        Preview::new(wire::call_term_signed(listing_uri, "ensureUnit", &args)?)
    }

    pub async fn activate_listing(&self, listing_uri: &RegistryUri) -> Result<Write<()>> {
        let (_values, receipt) = self.submit_uri(listing_uri, "activate", &[owner()]).await?;
        Ok(Write { value: (), receipt })
    }

    pub async fn pause_listing(&self, listing_uri: &RegistryUri) -> Result<Write<()>> {
        let (_values, receipt) = self.submit_uri(listing_uri, "pause", &[owner()]).await?;
        Ok(Write { value: (), receipt })
    }

    pub async fn resume_listing(&self, listing_uri: &RegistryUri) -> Result<Write<()>> {
        let (_values, receipt) = self.submit_uri(listing_uri, "resume", &[owner()]).await?;
        Ok(Write { value: (), receipt })
    }

    pub async fn delist_listing(&self, listing_uri: &RegistryUri) -> Result<Write<()>> {
        let (_values, receipt) = self.submit_uri(listing_uri, "delist", &[owner()]).await?;
        Ok(Write { value: (), receipt })
    }

    pub async fn pause_unit(&self, listing_uri: &RegistryUri, unit_id: &str) -> Result<Write<()>> {
        let args = vec![owner(), quote_string(unit_id)?];
        let (_values, receipt) = self.submit_uri(listing_uri, "pauseUnit", &args).await?;
        Ok(Write { value: (), receipt })
    }

    pub async fn resume_unit(&self, listing_uri: &RegistryUri, unit_id: &str) -> Result<Write<()>> {
        let args = vec![owner(), quote_string(unit_id)?];
        let (_values, receipt) = self.submit_uri(listing_uri, "resumeUnit", &args).await?;
        Ok(Write { value: (), receipt })
    }

    pub async fn reset_unit(
        &self,
        listing_uri: &RegistryUri,
        unit_id: &str,
        slot_id: &str,
    ) -> Result<Write<()>> {
        let args = vec![owner(), quote_string(unit_id)?, quote_string(slot_id)?];
        let (_values, receipt) = self.submit_uri(listing_uri, "reset", &args).await?;
        Ok(Write { value: (), receipt })
    }

    /// Read the listing, preflight the request locally, and book through the guest ledger.
    pub async fn book(
        &self,
        listing_uri: &RegistryUri,
        requests: &[UnitSlotRequest],
        op: &OpId,
        expected_terms_hash: Option<&str>,
    ) -> Result<BookOutcome> {
        let listing = self.listing_info(listing_uri).await?;
        self.preflight_booking(&listing.value, requests, expected_terms_hash)?;
        let ledger = self.guest_ledger().await?;
        let expected = expected_terms_hash.unwrap_or(&listing.value.terms_hash);
        let args = vec![
            owner(),
            quote_string(listing_uri.as_str())?,
            render_unit_slots(requests)?,
            quote_string(op.as_str())?,
            rcash_core::rho::hex_bytes(expected)?,
        ];
        let (values, receipt) = self
            .submit_uri(&RegistryUri::new(&ledger.value.uri)?, "book", &args)
            .await?;
        let booking_id = text_at(&values, 0, "booking id")?;
        let booking_uri = text_at(&values, 1, "booking uri")?;
        Ok(BookOutcome {
            booking_id,
            booking_uri,
            ledger_uri: ledger.value.uri,
            guest_id: ledger.value.guest_id,
            receipt,
        })
    }

    /// A preview of the book term. Requires the guest ledger URI from `guest_ledger`.
    pub fn preview_book(
        &self,
        ledger_uri: &RegistryUri,
        listing_uri: &RegistryUri,
        requests: &[UnitSlotRequest],
        op: &OpId,
        expected_terms_hash: &str,
    ) -> Result<Preview> {
        let args = vec![
            owner(),
            quote_string(listing_uri.as_str())?,
            render_unit_slots(requests)?,
            quote_string(op.as_str())?,
            rcash_core::rho::hex_bytes(expected_terms_hash)?,
        ];
        Preview::new(wire::call_term_signed(ledger_uri, "book", &args)?)
    }

    fn preflight_booking(
        &self,
        listing: &ListingView,
        requests: &[UnitSlotRequest],
        expected_terms_hash: Option<&str>,
    ) -> Result<()> {
        if listing.status != rshare_core::ListingStatus::Active {
            return Err(SdkError::refused("listing inactive"));
        }
        validate_unit_slots(requests, listing.terms.max_units)?;
        if let Some(expected) = expected_terms_hash {
            if expected != listing.terms_hash {
                return Err(SdkError::refused("terms"));
            }
        }
        Ok(())
    }

    /// One lifecycle action on a booking.
    pub async fn booking_action(
        &self,
        booking_uri: &RegistryUri,
        action: &BookingAction,
        op: &OpId,
    ) -> Result<Write<()>> {
        let (arm, args) = action.arm_and_args(op)?;
        let (_values, receipt) = self.submit_uri(booking_uri, arm, &args).await?;
        Ok(Write { value: (), receipt })
    }

    /// Fetch a booking's current view (convenience for callers that then act).
    pub async fn booking_or_err(&self, booking_uri: &RegistryUri) -> Result<BookingView> {
        Ok(self.booking_info(booking_uri).await?.value)
    }
}

fn owner() -> String {
    "*deployerId".to_string()
}

fn text_at(values: &[RhoValue], index: usize, label: &str) -> Result<String> {
    values
        .get(index)
        .and_then(RhoValue::as_str)
        .map(str::to_string)
        .ok_or_else(|| SdkError::Decode(format!("{label} is missing or not a string")))
}

/// Render a list of hex public keys as a Rholang byte-array list.
pub fn admins_rho(admins: &[String]) -> Result<String> {
    let rendered = admins
        .iter()
        .map(|admin| rcash_core::rho::hex_bytes(admin).map_err(SdkError::from))
        .collect::<Result<Vec<_>>>()?;
    Ok(format!("[{}]", rendered.join(", ")))
}

/// The comma-joined public-key text `setAdmins` uses for its signed payload.
pub fn admins_text(admins: &[String]) -> String {
    admins.join(",")
}

/// Render a list of hex signatures as a Rholang byte-array list.
pub fn signatures_rho(signatures: &[String]) -> Result<String> {
    let rendered = signatures
        .iter()
        .map(|signature| rcash_core::rho::hex_bytes(signature).map_err(SdkError::from))
        .collect::<Result<Vec<_>>>()?;
    Ok(format!("[{}]", rendered.join(", ")))
}

/// A read from `Terms` that returns a bare value.
pub async fn terms_read(client: &Client, arm: &str, args: &[String]) -> Result<Read<Value>> {
    let component = client.resolve(ComponentKey::Terms).await?;
    let term = wire::call_term(&component.uri, arm, args)?;
    let (value, meta) = wire::explore_value(&client.inner, &term).await?;
    Ok(Read {
        value: value.to_json(),
        meta,
    })
}

/// A listing's map as JSON, for display.
pub fn listing_json(listing: &ListingView) -> Value {
    listing.to_json()
}

/// A booking map as JSON, for display.
pub fn booking_json(booking: &BookingView) -> Value {
    booking.to_json()
}

/// A map of provider listings, for display.
pub fn provider_listings_json(
    listings: &BTreeMap<String, crate::models::ProviderListingEntry>,
) -> Value {
    Value::Object(
        listings
            .iter()
            .map(|(id, entry)| (id.clone(), entry.to_json()))
            .collect(),
    )
}

/// A map of units, for display.
pub fn units_json(units: &BTreeMap<String, crate::models::UnitView>) -> Value {
    Value::Object(
        units
            .iter()
            .map(|(id, unit)| (id.clone(), unit.to_json()))
            .collect(),
    )
}
