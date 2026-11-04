//! Typed views of the chain's state records.

use std::collections::BTreeMap;

use rcash_chain::rho_expr::RhoValue;
use serde_json::Value;

use rshare_core::{
    BookingStatus, ListingStatus, ProviderStatus, ResourceKind, SlotSpec, SlotState, Terms,
    UnitStatus,
};

use crate::error::{Result, SdkError};

type RhoMap = BTreeMap<String, RhoValue>;

/// A provider's public profile and listing index.
#[derive(Clone, Debug, PartialEq)]
pub struct ProviderView {
    pub provider_id: String,
    pub profile: Value,
    pub status: ProviderStatus,
    pub listings: BTreeMap<String, ProviderListingEntry>,
    pub admins: Vec<String>,
    pub threshold: i64,
    pub nonce: i64,
}

/// One entry in a provider's listing index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderListingEntry {
    pub uri: String,
    pub kind: ResourceKind,
    pub terms_hash: String,
    pub status: String,
}

/// A listing and its units. Slot records are never part of `info`.
#[derive(Clone, Debug, PartialEq)]
pub struct ListingView {
    pub listing_id: String,
    pub provider_uri: String,
    pub provider_id: String,
    pub kind: ResourceKind,
    pub terms: Terms,
    pub terms_hash: String,
    pub status: ListingStatus,
    pub units: BTreeMap<String, UnitView>,
    pub admins: Vec<String>,
    pub threshold: i64,
    pub nonce: i64,
}

/// One unit's schedule and price.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitView {
    pub unit_id: String,
    pub kind: ResourceKind,
    pub attrs: Value,
    pub price_per_slot: i64,
    pub max_hold_blocks: i64,
    pub status: UnitStatus,
    pub schedule: BTreeMap<String, SlotSpec>,
}

/// One unit's requested slots as stored on a booking.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookingUnit {
    pub unit_id: String,
    pub slots: Vec<String>,
}

/// A booking's full public state.
#[derive(Clone, Debug, PartialEq)]
pub struct BookingView {
    pub booking_id: String,
    pub guest_id: String,
    pub provider_id: String,
    pub listing_id: String,
    pub listing_uri: String,
    pub kind: ResourceKind,
    pub unit_slots: Vec<BookingUnit>,
    pub asset_id: String,
    pub rent: i64,
    pub deposit: i64,
    pub fee_bps: i64,
    pub bond_uri: String,
    pub hold_until_height: i64,
    pub check_in_height: i64,
    pub check_out_height: i64,
    pub status: BookingStatus,
    pub cancellation: Terms,
    pub admins: Vec<String>,
    pub threshold: i64,
    pub check_in_at_height: i64,
    pub check_out_at_height: i64,
    pub deposit_release_height: i64,
    pub dispute_at_height: i64,
    pub reject_reason: String,
    pub ticket: TicketView,
    pub claim: ClaimView,
    pub ops: BTreeMap<String, Value>,
    pub nonce: i64,
}

/// A booking's ticket record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TicketView {
    pub holder_id: String,
    pub redeemed: bool,
    pub redeemed_at_height: i64,
    pub transfer_cutoff_height: i64,
}

/// A deposit claim's evidence anchor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimView {
    pub amount: i64,
    pub evidence_hash: String,
    pub at_height: i64,
}

/// Bond status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BondInfo {
    pub paid: bool,
    pub expected: i64,
    pub remaining: i64,
}

/// A quote or acquire summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuoteSummary {
    pub count: i64,
    pub rent: i64,
    pub min_start: i64,
    pub max_end: i64,
    pub check_in: i64,
    pub check_out: i64,
}

/// The caller's r-cash account balance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountView {
    pub owner_id: String,
    pub rev_base_units: i64,
}

/// One guest-ledger operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuestOp {
    pub pending: bool,
    pub ok: bool,
    pub booking_id: Option<String>,
    pub uri: Option<String>,
}

impl ProviderView {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "providerId": self.provider_id,
            "profile": self.profile,
            "status": self.status.as_str(),
            "listings": self.listings.iter().map(|(id, entry)| (id.clone(), entry.to_json())).collect::<serde_json::Map<_, _>>(),
            "admins": self.admins,
            "threshold": self.threshold,
            "nonce": self.nonce,
        })
    }

    pub fn from_rho(value: &RhoValue) -> Result<Self> {
        let map = map(value, "provider info")?;
        let listings = optional_map(map, "listings")
            .map(|listings| {
                listings
                    .iter()
                    .map(|(id, entry)| Ok((id.clone(), ProviderListingEntry::from_rho(entry)?)))
                    .collect::<Result<BTreeMap<_, _>>>()
            })
            .transpose()?
            .unwrap_or_default();
        Ok(Self {
            provider_id: text(map, "providerId")?,
            profile: json(map, "profile")?,
            status: ProviderStatus::parse(&text(map, "status")?),
            listings,
            admins: admins(map, "admins")?,
            threshold: int(map, "threshold")?,
            nonce: int(map, "nonce")?,
        })
    }
}

impl ProviderListingEntry {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "uri": self.uri,
            "kind": self.kind.as_str(),
            "termsHash": self.terms_hash,
            "status": self.status,
        })
    }

    pub fn from_rho(value: &RhoValue) -> Result<Self> {
        let map = map(value, "provider listing")?;
        let kind = text(map, "kind")?;
        Ok(Self {
            uri: text(map, "uri")?,
            kind: ResourceKind::parse(&kind)
                .ok_or_else(|| SdkError::Decode(format!("unknown resource kind: {kind}")))?,
            terms_hash: bytes_text(map, "termsHash")?,
            status: text(map, "status")?,
        })
    }
}

impl ListingView {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "listingId": self.listing_id,
            "providerUri": self.provider_uri,
            "providerId": self.provider_id,
            "kind": self.kind.as_str(),
            "terms": self.terms.to_json(),
            "termsHash": self.terms_hash,
            "status": self.status.as_str(),
            "units": self.units.iter().map(|(id, unit)| (id.clone(), unit.to_json())).collect::<serde_json::Map<_, _>>(),
            "admins": self.admins,
            "threshold": self.threshold,
            "nonce": self.nonce,
        })
    }

    pub fn from_rho(value: &RhoValue) -> Result<Self> {
        let map = map(value, "listing info")?;
        let kind_text = text(map, "kind")?;
        let kind = ResourceKind::parse(&kind_text)
            .ok_or_else(|| SdkError::Decode(format!("unknown resource kind: {kind_text}")))?;
        let terms = Terms::from_value(kind, &json(map, "terms")?)?;
        let units = optional_map(map, "units")
            .map(|units| {
                units
                    .iter()
                    .map(|(id, unit)| Ok((id.clone(), UnitView::from_rho(unit)?)))
                    .collect::<Result<BTreeMap<_, _>>>()
            })
            .transpose()?
            .unwrap_or_default();
        Ok(Self {
            listing_id: text(map, "listingId")?,
            provider_uri: text(map, "providerUri")?,
            provider_id: text(map, "providerId")?,
            kind,
            terms,
            terms_hash: bytes_text(map, "termsHash")?,
            status: ListingStatus::parse(&text(map, "status")?),
            units,
            admins: admins(map, "admins")?,
            threshold: int(map, "threshold")?,
            nonce: int(map, "nonce")?,
        })
    }
}

impl UnitView {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "unitId": self.unit_id,
            "kind": self.kind.as_str(),
            "attrs": self.attrs,
            "pricePerSlot": self.price_per_slot,
            "maxHoldBlocks": self.max_hold_blocks,
            "status": self.status.as_str(),
            "schedule": self.schedule,
        })
    }

    pub fn from_rho(value: &RhoValue) -> Result<Self> {
        let map = map(value, "unit")?;
        let kind_text = text(map, "kind")?;
        let kind = ResourceKind::parse(&kind_text)
            .ok_or_else(|| SdkError::Decode(format!("unknown resource kind: {kind_text}")))?;
        let schedule = optional_map(map, "schedule")
            .map(|schedule| {
                schedule
                    .iter()
                    .map(|(id, spec)| Ok((id.clone(), slot_spec(spec)?)))
                    .collect::<Result<BTreeMap<_, _>>>()
            })
            .transpose()?
            .unwrap_or_default();
        Ok(Self {
            unit_id: text(map, "unitId")?,
            kind,
            attrs: json(map, "attrs")?,
            price_per_slot: int(map, "pricePerSlot")?,
            max_hold_blocks: int(map, "maxHoldBlocks")?,
            status: UnitStatus::parse(&text(map, "status")?),
            schedule,
        })
    }
}

impl BookingView {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "bookingId": self.booking_id,
            "guestId": self.guest_id,
            "providerId": self.provider_id,
            "listingId": self.listing_id,
            "listingUri": self.listing_uri,
            "kind": self.kind.as_str(),
            "unitSlots": self.unit_slots.iter().map(|unit| serde_json::json!({
                "unitId": unit.unit_id,
                "slots": unit.slots,
            })).collect::<Vec<_>>(),
            "assetId": self.asset_id,
            "rent": self.rent,
            "deposit": self.deposit,
            "feeBps": self.fee_bps,
            "bondUri": self.bond_uri,
            "holdUntilHeight": self.hold_until_height,
            "checkInHeight": self.check_in_height,
            "checkOutHeight": self.check_out_height,
            "status": self.status.as_str(),
            "cancellation": self.cancellation.to_json(),
            "admins": self.admins,
            "threshold": self.threshold,
            "checkInAtHeight": self.check_in_at_height,
            "checkOutAtHeight": self.check_out_at_height,
            "depositReleaseHeight": self.deposit_release_height,
            "disputeAtHeight": self.dispute_at_height,
            "rejectReason": self.reject_reason,
            "ticket": self.ticket.to_json(),
            "claim": self.claim.to_json(),
            "ops": self.ops,
            "nonce": self.nonce,
        })
    }

    pub fn from_rho(value: &RhoValue) -> Result<Self> {
        let map = map(value, "booking info")?;
        let kind_text = text(map, "kind")?;
        let kind = ResourceKind::parse(&kind_text)
            .ok_or_else(|| SdkError::Decode(format!("unknown resource kind: {kind_text}")))?;
        let unit_slots = list(map, "unitSlots")?
            .iter()
            .map(booking_unit)
            .collect::<Result<Vec<_>>>()?;
        let ops = optional_map(map, "ops")
            .map(|ops| {
                ops.iter()
                    .map(|(id, value)| (id.clone(), value.to_json()))
                    .collect()
            })
            .unwrap_or_default();
        Ok(Self {
            booking_id: text(map, "bookingId")?,
            guest_id: text(map, "guestId")?,
            provider_id: text(map, "providerId")?,
            listing_id: text(map, "listingId")?,
            listing_uri: text(map, "listingUri")?,
            kind,
            unit_slots,
            asset_id: text(map, "assetId")?,
            rent: int(map, "rent")?,
            deposit: int(map, "deposit")?,
            fee_bps: int(map, "feeBps")?,
            bond_uri: text(map, "bondUri")?,
            hold_until_height: int(map, "holdUntilHeight")?,
            check_in_height: int(map, "checkInHeight")?,
            check_out_height: int(map, "checkOutHeight")?,
            status: BookingStatus::parse(&text(map, "status")?),
            cancellation: Terms::from_value(kind, &json(map, "cancellation")?)?,
            admins: admins(map, "admins")?,
            threshold: int(map, "threshold")?,
            check_in_at_height: int_or(map, "checkInAtHeight", 0),
            check_out_at_height: int_or(map, "checkOutAtHeight", 0),
            deposit_release_height: int_or(map, "depositReleaseHeight", 0),
            dispute_at_height: int_or(map, "disputeAtHeight", 0),
            reject_reason: text_or(map, "rejectReason"),
            ticket: TicketView::from_rho(field(map, "ticket")?)?,
            claim: ClaimView::from_rho(field(map, "claim")?)?,
            ops,
            nonce: int_or(map, "nonce", 0),
        })
    }
}

impl TicketView {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "holderId": self.holder_id,
            "redeemed": self.redeemed,
            "redeemedAtHeight": self.redeemed_at_height,
            "transferCutoffHeight": self.transfer_cutoff_height,
        })
    }

    pub fn from_rho(value: &RhoValue) -> Result<Self> {
        let map = map(value, "ticket")?;
        Ok(Self {
            holder_id: text(map, "holderId")?,
            redeemed: boolean(map, "redeemed")?,
            redeemed_at_height: int(map, "redeemedAtHeight")?,
            transfer_cutoff_height: int(map, "transferCutoffHeight")?,
        })
    }
}

impl ClaimView {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "amount": self.amount,
            "evidenceHash": self.evidence_hash,
            "atHeight": self.at_height,
        })
    }

    pub fn from_rho(value: &RhoValue) -> Result<Self> {
        let map = map(value, "claim")?;
        Ok(Self {
            amount: int(map, "amount")?,
            evidence_hash: text_or(map, "evidenceHash"),
            at_height: int(map, "atHeight")?,
        })
    }
}

impl BondInfo {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "paid": self.paid,
            "expected": self.expected,
            "remaining": self.remaining,
        })
    }

    pub fn from_rho(value: &RhoValue) -> Result<Self> {
        let map = map(value, "bond info")?;
        Ok(Self {
            paid: boolean(map, "paid")?,
            expected: int(map, "expected")?,
            remaining: int(map, "remaining")?,
        })
    }
}

impl QuoteSummary {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "count": self.count,
            "rent": self.rent,
            "minStart": self.min_start,
            "maxEnd": self.max_end,
            "checkIn": self.check_in,
            "checkOut": self.check_out,
        })
    }

    pub fn from_rho(value: &RhoValue) -> Result<Self> {
        let map = map(value, "quote")?;
        Ok(Self {
            count: int(map, "count")?,
            rent: int(map, "rent")?,
            min_start: int(map, "minStart")?,
            max_end: int(map, "maxEnd")?,
            check_in: int(map, "checkIn")?,
            check_out: int(map, "checkOut")?,
        })
    }
}

impl GuestOp {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "pending": self.pending,
            "ok": self.ok,
            "bookingId": self.booking_id,
            "uri": self.uri,
        })
    }

    pub fn from_rho(value: &RhoValue) -> Result<Self> {
        let map = map(value, "ledger op")?;
        Ok(Self {
            pending: boolean(map, "pending")?,
            ok: boolean(map, "ok").unwrap_or(false),
            booking_id: optional_text(map, "bookingId"),
            uri: optional_text(map, "uri"),
        })
    }
}

impl AccountView {
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "ownerId": self.owner_id,
            "revBaseUnits": self.rev_base_units,
        })
    }
}

/// Decode one slot record. Release tokens are already redacted by `probe`; a four-field
/// held record (from a direct caller) is still treated as held and the token is discarded.
pub fn slot_state(value: &RhoValue) -> SlotState {
    let Some(items) = value.as_list() else {
        return SlotState::Invalid;
    };
    match items.first().and_then(RhoValue::as_str) {
        Some("free") => items
            .get(1)
            .and_then(RhoValue::as_int)
            .map(|generation| SlotState::Free { generation })
            .unwrap_or(SlotState::Invalid),
        Some("held") => match (
            items.get(1).and_then(RhoValue::as_str),
            items.get(2).and_then(RhoValue::as_int),
        ) {
            (Some(booking_id), Some(until_height)) => SlotState::Held {
                booking_id: booking_id.to_string(),
                until_height,
            },
            _ => SlotState::Invalid,
        },
        Some("booked") => items
            .get(1)
            .and_then(RhoValue::as_str)
            .map(|booking_id| SlotState::Booked {
                booking_id: booking_id.to_string(),
            })
            .unwrap_or(SlotState::Invalid),
        _ => SlotState::Invalid,
    }
}

fn booking_unit(value: &RhoValue) -> Result<BookingUnit> {
    let map = map(value, "booking unit")?;
    let slots = list(map, "slots")?
        .iter()
        .map(|slot| {
            slot.as_str()
                .map(str::to_string)
                .ok_or_else(|| SdkError::Decode("slot id is not a string".into()))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(BookingUnit {
        unit_id: text(map, "unitId")?,
        slots,
    })
}

pub(crate) fn slot_spec(value: &RhoValue) -> Result<SlotSpec> {
    let map = map(value, "slot spec")?;
    Ok(SlotSpec {
        slot_id: text_or(map, "slotId"),
        start_height: int(map, "startHeight")?,
        end_height: int(map, "endHeight")?,
    })
}

fn map<'a>(value: &'a RhoValue, label: &str) -> Result<&'a RhoMap> {
    value
        .as_map()
        .ok_or_else(|| SdkError::Decode(format!("{label} is not a map")))
}

fn field<'a>(map: &'a RhoMap, key: &str) -> Result<&'a RhoValue> {
    map.get(key)
        .ok_or_else(|| SdkError::Decode(format!("missing {key}")))
}

fn optional_map<'a>(map: &'a RhoMap, key: &str) -> Option<&'a RhoMap> {
    map.get(key).and_then(RhoValue::as_map)
}

fn int(map: &RhoMap, key: &str) -> Result<i64> {
    field(map, key)?
        .as_int()
        .ok_or_else(|| SdkError::Decode(format!("{key} is not an integer")))
}

fn int_or(map: &RhoMap, key: &str, default: i64) -> i64 {
    map.get(key).and_then(RhoValue::as_int).unwrap_or(default)
}

fn boolean(map: &RhoMap, key: &str) -> Result<bool> {
    field(map, key)?
        .as_bool()
        .ok_or_else(|| SdkError::Decode(format!("{key} is not a boolean")))
}

fn text(map: &RhoMap, key: &str) -> Result<String> {
    field(map, key)?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| SdkError::Decode(format!("{key} is not a string")))
}

/// A hash field arrives as a byte array from `rho:crypto:blake2b256Hash`; accept a hex
/// string too so records written by a different client decode.
fn bytes_text(map: &RhoMap, key: &str) -> Result<String> {
    match field(map, key)? {
        RhoValue::Bytes(hex) => Ok(hex.clone()),
        other => other
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| SdkError::Decode(format!("{key} is not a byte array or string"))),
    }
}

fn text_or(map: &RhoMap, key: &str) -> String {
    map.get(key)
        .and_then(RhoValue::as_str)
        .unwrap_or_default()
        .to_string()
}

fn optional_text(map: &RhoMap, key: &str) -> Option<String> {
    map.get(key).and_then(RhoValue::as_str).map(str::to_string)
}

fn list<'a>(map: &'a RhoMap, key: &str) -> Result<&'a [RhoValue]> {
    field(map, key)?
        .as_list()
        .ok_or_else(|| SdkError::Decode(format!("{key} is not a list")))
}

fn json(map: &RhoMap, key: &str) -> Result<Value> {
    Ok(field(map, key)?.to_json())
}

fn admins(map: &RhoMap, key: &str) -> Result<Vec<String>> {
    Ok(map.get(key).map(admin_list).unwrap_or_default())
}

/// Decode a list of byte-array or string public keys.
pub(crate) fn admin_list(value: &RhoValue) -> Vec<String> {
    let Some(items) = value.as_list() else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| match item {
            RhoValue::Bytes(hex) => Some(hex.clone()),
            other => other.as_str().map(str::to_string),
        })
        .collect()
}
