//! Typed views of the chain's state records.
//!
//! Decoding is strict: a field the v2 contract always writes must be present and carry the
//! expected Rholang type. Fields that only v1 records omit are explicit `Option`s, never
//! fabricated zero values. Every failure carries a [`DecodeError`] with the record, field
//! and category.

use std::collections::BTreeMap;

use rcash_chain::rho_expr::RhoValue;
use rcash_core::AssetId;
use serde::Serialize;
use serde_json::Value;

use rshare_core::{
    Amount, BasisPoints, BlockCount, BookingStatus, Height, ListingStatus, Nonce, OpId, OwnerId,
    ProviderStatus, RegistryUri, ResourceKind, SlotSpec, SlotState, Terms, UnitStatus,
};

use crate::error::{DecodeKind, Result, SdkError};

type RhoMap = BTreeMap<String, RhoValue>;

/// A JSON object whose shape the SDK does not interpret (profile, unit attrs).
#[derive(Clone, Debug, PartialEq)]
pub struct OpaqueMetadata(Value);

impl OpaqueMetadata {
    /// Wrap a JSON value, requiring an object.
    pub fn new(value: Value) -> Result<Self> {
        Self::decode("opaque metadata", "value", value)
    }

    pub(crate) fn decode(context: &str, field: &str, value: Value) -> Result<Self> {
        if value.is_object() {
            Ok(Self(value))
        } else {
            Err(SdkError::decode(context, field, DecodeKind::WrongType))
        }
    }

    pub fn as_value(&self) -> &Value {
        &self.0
    }
}

impl Serialize for OpaqueMetadata {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

/// A provider's public profile and listing index.
#[derive(Clone, Debug, PartialEq)]
pub struct ProviderView {
    pub provider_id: OwnerId,
    pub profile: OpaqueMetadata,
    pub status: ProviderStatus,
    pub listings: BTreeMap<String, ProviderListingEntry>,
    pub admins: Vec<String>,
    pub threshold: i64,
    pub nonce: Nonce,
}

/// One entry in a provider's listing index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderListingEntry {
    pub uri: RegistryUri,
    pub kind: ResourceKind,
    pub terms_hash: String,
    /// `active` or `removed`; no shared status enum covers both, so the text is kept but
    /// must be present and must be a string.
    pub status: String,
}

/// A listing and its units. Slot records are never part of `info`.
#[derive(Clone, Debug, PartialEq)]
pub struct ListingView {
    pub listing_id: String,
    pub provider_uri: RegistryUri,
    pub provider_id: OwnerId,
    pub kind: ResourceKind,
    pub terms: Terms,
    pub terms_hash: String,
    pub status: ListingStatus,
    pub units: BTreeMap<String, UnitView>,
    pub admins: Vec<String>,
    pub threshold: i64,
    pub nonce: Nonce,
}

/// One unit's schedule and price.
#[derive(Clone, Debug, PartialEq)]
pub struct UnitView {
    pub unit_id: String,
    pub kind: ResourceKind,
    pub attrs: OpaqueMetadata,
    pub price_per_slot: Amount,
    pub max_hold_blocks: BlockCount,
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
    pub guest_id: OwnerId,
    pub provider_id: OwnerId,
    pub listing_id: String,
    pub listing_uri: RegistryUri,
    pub kind: ResourceKind,
    pub unit_slots: Vec<BookingUnit>,
    pub asset_id: AssetId,
    pub rent: Amount,
    pub deposit: Amount,
    pub fee_bps: BasisPoints,
    pub bond_uri: RegistryUri,
    pub hold_until_height: Height,
    pub check_in_height: Height,
    pub check_out_height: Height,
    pub status: BookingStatus,
    pub cancellation: Terms,
    pub admins: Vec<String>,
    pub threshold: i64,
    pub check_in_at_height: Option<Height>,
    pub check_out_at_height: Option<Height>,
    pub deposit_release_height: Option<Height>,
    pub dispute_at_height: Option<Height>,
    pub no_show_claim_at_height: Option<Height>,
    pub reject_reason: Option<String>,
    pub ticket: TicketView,
    pub claim: ClaimView,
    pub ops: BTreeMap<OpId, Value>,
    pub nonce: Nonce,
}

/// A booking's ticket record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TicketView {
    pub holder_id: OwnerId,
    pub redeemed: bool,
    pub redeemed_at_height: Height,
    pub transfer_cutoff_height: Height,
}

/// A deposit claim's evidence anchor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimView {
    pub amount: Amount,
    pub evidence_hash: String,
    pub at_height: Height,
}

/// Bond status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BondInfo {
    pub paid: bool,
    pub expected: Amount,
    pub remaining: Amount,
}

/// A quote or acquire summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuoteSummary {
    pub count: i64,
    pub rent: Amount,
    pub min_start: Height,
    pub max_end: Height,
    pub check_in: Height,
    pub check_out: Height,
}

/// The caller's r-cash account balance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountView {
    pub owner_id: OwnerId,
    pub rev_base_units: Amount,
}

/// One guest-ledger operation.
///
/// A reserved operation is `pending` with no outcome yet. A finished operation has
/// `pending == false` and an `ok` outcome; it never defaults to `false`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuestOp {
    pub pending: bool,
    pub ok: Option<bool>,
    pub booking_id: Option<String>,
    pub uri: Option<RegistryUri>,
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
        let decoder = Decoder::new(value, "provider info")?;
        let listings = decoder
            .map("listings")?
            .iter()
            .map(|(id, entry)| Ok((id.clone(), ProviderListingEntry::from_rho(entry)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        Ok(Self {
            provider_id: decoder.owner_id("providerId")?,
            profile: decoder.metadata("profile")?,
            status: ProviderStatus::parse(&decoder.string("status")?),
            listings,
            admins: decoder.admins("admins")?,
            threshold: decoder.int("threshold")?,
            nonce: decoder.nonce("nonce")?,
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
        let decoder = Decoder::new(value, "provider listing")?;
        Ok(Self {
            uri: decoder.registry_uri("uri")?,
            kind: decoder.kind("kind")?,
            terms_hash: decoder.bytes_text("termsHash")?,
            status: decoder.string("status")?,
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
        let decoder = Decoder::new(value, "listing info")?;
        let kind = decoder.kind("kind")?;
        let terms = Terms::from_value(kind, &decoder.json("terms")?)?;
        let units = decoder
            .map("units")?
            .iter()
            .map(|(id, unit)| Ok((id.clone(), UnitView::from_rho(unit)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        Ok(Self {
            listing_id: decoder.string("listingId")?,
            provider_uri: decoder.registry_uri("providerUri")?,
            provider_id: decoder.owner_id("providerId")?,
            kind,
            terms,
            terms_hash: decoder.bytes_text("termsHash")?,
            status: ListingStatus::parse(&decoder.string("status")?),
            units,
            admins: decoder.admins("admins")?,
            threshold: decoder.int("threshold")?,
            nonce: decoder.nonce("nonce")?,
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
        let decoder = Decoder::new(value, "unit")?;
        let schedule = decoder
            .map("schedule")?
            .iter()
            .map(|(id, spec)| Ok((id.clone(), slot_spec(spec)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        Ok(Self {
            unit_id: decoder.string("unitId")?,
            kind: decoder.kind("kind")?,
            attrs: decoder.metadata("attrs")?,
            price_per_slot: decoder.amount("pricePerSlot")?,
            max_hold_blocks: decoder.block_count("maxHoldBlocks")?,
            status: UnitStatus::parse(&decoder.string("status")?),
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
            "noShowClaimAtHeight": self.no_show_claim_at_height,
            "rejectReason": self.reject_reason,
            "ticket": self.ticket.to_json(),
            "claim": self.claim.to_json(),
            "ops": self.ops.iter().map(|(id, value)| (id.as_str().to_string(), value.clone())).collect::<serde_json::Map<_, _>>(),
            "nonce": self.nonce,
        })
    }

    pub fn from_rho(value: &RhoValue) -> Result<Self> {
        let decoder = Decoder::new(value, "booking info")?;
        decoder.check_schema("schemaVersion", 2)?;
        let kind = decoder.kind("kind")?;
        let unit_slots = decoder
            .list("unitSlots")?
            .iter()
            .map(booking_unit)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            booking_id: decoder.string("bookingId")?,
            guest_id: decoder.owner_id("guestId")?,
            provider_id: decoder.owner_id("providerId")?,
            listing_id: decoder.string("listingId")?,
            listing_uri: decoder.registry_uri("listingUri")?,
            kind,
            unit_slots,
            asset_id: decoder.asset_id("assetId")?,
            rent: decoder.amount("rent")?,
            deposit: decoder.amount("deposit")?,
            fee_bps: decoder.basis_points("feeBps")?,
            bond_uri: decoder.registry_uri("bondUri")?,
            hold_until_height: decoder.height("holdUntilHeight")?,
            check_in_height: decoder.height("checkInHeight")?,
            check_out_height: decoder.height("checkOutHeight")?,
            status: BookingStatus::parse(&decoder.string("status")?),
            cancellation: Terms::from_value(kind, &decoder.json("cancellation")?)?,
            admins: decoder.admins("admins")?,
            threshold: decoder.int("threshold")?,
            check_in_at_height: decoder.optional_height("checkInAtHeight")?,
            check_out_at_height: decoder.optional_height("checkOutAtHeight")?,
            deposit_release_height: decoder.optional_height("depositReleaseHeight")?,
            dispute_at_height: decoder.optional_height("disputeAtHeight")?,
            no_show_claim_at_height: decoder.optional_height("noShowClaimAtHeight")?,
            reject_reason: decoder.optional_string("rejectReason")?,
            ticket: TicketView::from_rho(decoder.raw("ticket")?)?,
            claim: ClaimView::from_rho(decoder.raw("claim")?)?,
            ops: decoder.op_map("ops")?,
            nonce: decoder.nonce("nonce")?,
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
        let decoder = Decoder::new(value, "ticket")?;
        Ok(Self {
            holder_id: decoder.owner_id("holderId")?,
            redeemed: decoder.boolean("redeemed")?,
            redeemed_at_height: decoder.height("redeemedAtHeight")?,
            transfer_cutoff_height: decoder.height("transferCutoffHeight")?,
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
        let decoder = Decoder::new(value, "claim")?;
        Ok(Self {
            amount: decoder.amount("amount")?,
            evidence_hash: decoder.string("evidenceHash")?,
            at_height: decoder.height("atHeight")?,
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
        let decoder = Decoder::new(value, "bond info")?;
        Ok(Self {
            paid: decoder.boolean("paid")?,
            expected: decoder.amount("expected")?,
            remaining: decoder.amount("remaining")?,
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
        let decoder = Decoder::new(value, "quote")?;
        Ok(Self {
            count: decoder.int("count")?,
            rent: decoder.amount("rent")?,
            min_start: decoder.height("minStart")?,
            max_end: decoder.height("maxEnd")?,
            check_in: decoder.height("checkIn")?,
            check_out: decoder.height("checkOut")?,
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
        let decoder = Decoder::new(value, "ledger op")?;
        let pending = decoder.boolean("pending")?;
        let ok = decoder.optional_bool("ok")?;
        if !pending && ok.is_none() {
            return Err(SdkError::decode("ledger op", "ok", DecodeKind::Missing));
        }
        Ok(Self {
            pending,
            ok,
            booking_id: decoder.optional_string("bookingId")?,
            uri: decoder.optional_registry_uri("uri")?,
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

    pub fn from_rho(value: &RhoValue) -> Result<Self> {
        let decoder = Decoder::new(value, "account")?;
        Ok(Self {
            owner_id: decoder.owner_id("ownerId")?,
            rev_base_units: decoder.amount("revBaseUnits")?,
        })
    }
}

/// Decode one slot record. Release tokens are already redacted by `probe`; a four-field
/// held record (from a direct caller) is still treated as held and the token is discarded.
///
/// A malformed record is a decode error, never `SlotState::Invalid`.
pub fn slot_state(value: &RhoValue) -> Result<SlotState> {
    const CONTEXT: &str = "slot state";
    let items = value
        .as_list()
        .ok_or_else(|| SdkError::decode(CONTEXT, "record", DecodeKind::WrongType))?;
    let tag = match items.first() {
        None => return Err(SdkError::decode(CONTEXT, "tag", DecodeKind::Missing)),
        Some(tag) => tag
            .as_str()
            .ok_or_else(|| SdkError::decode(CONTEXT, "tag", DecodeKind::WrongType))?,
    };
    match tag {
        "free" => Ok(SlotState::Free {
            generation: item_int(items, 1, CONTEXT, "generation")?,
        }),
        "held" => Ok(SlotState::Held {
            booking_id: item_str(items, 1, CONTEXT, "bookingId")?.to_string(),
            until_height: item_int(items, 2, CONTEXT, "untilHeight")?,
        }),
        "booked" => Ok(SlotState::Booked {
            booking_id: item_str(items, 1, CONTEXT, "bookingId")?.to_string(),
        }),
        _ => Err(SdkError::decode(CONTEXT, "tag", DecodeKind::UnknownTag)),
    }
}

fn booking_unit(value: &RhoValue) -> Result<BookingUnit> {
    let decoder = Decoder::new(value, "booking unit")?;
    let slots = decoder
        .list("slots")?
        .iter()
        .map(|slot| {
            slot.as_str()
                .map(str::to_string)
                .ok_or_else(|| SdkError::decode("booking unit", "slots", DecodeKind::WrongType))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(BookingUnit {
        unit_id: decoder.string("unitId")?,
        slots,
    })
}

pub(crate) fn slot_spec(value: &RhoValue) -> Result<SlotSpec> {
    let decoder = Decoder::new(value, "slot spec")?;
    Ok(SlotSpec {
        slot_id: decoder.string("slotId")?,
        start_height: decoder.height("startHeight")?,
        end_height: decoder.height("endHeight")?,
    })
}

fn item_int(items: &[RhoValue], index: usize, context: &str, field: &str) -> Result<i64> {
    item(items, index, context, field)?
        .as_int()
        .ok_or_else(|| SdkError::decode(context, field, DecodeKind::WrongType))
}

fn item_str<'a>(
    items: &'a [RhoValue],
    index: usize,
    context: &str,
    field: &str,
) -> Result<&'a str> {
    item(items, index, context, field)?
        .as_str()
        .ok_or_else(|| SdkError::decode(context, field, DecodeKind::WrongType))
}

fn item<'a>(
    items: &'a [RhoValue],
    index: usize,
    context: &str,
    field: &str,
) -> Result<&'a RhoValue> {
    items
        .get(index)
        .ok_or_else(|| SdkError::decode(context, field, DecodeKind::Missing))
}

/// Decode a list of byte-array or string public keys. Any other entry is an error.
pub(crate) fn admin_list(context: &str, field: &str, value: &RhoValue) -> Result<Vec<String>> {
    let items = value
        .as_list()
        .ok_or_else(|| SdkError::decode(context, field, DecodeKind::WrongType))?;
    items
        .iter()
        .map(|item| match item {
            RhoValue::Bytes(hex) => Ok(hex.clone()),
            other => other
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| SdkError::decode(context, field, DecodeKind::WrongType)),
        })
        .collect()
}

/// A record decoder. Every failure is annotated with the record's context.
struct Decoder<'a> {
    context: &'a str,
    map: &'a RhoMap,
}

impl<'a> Decoder<'a> {
    fn new(value: &'a RhoValue, context: &'a str) -> Result<Self> {
        let map = value
            .as_map()
            .ok_or_else(|| SdkError::decode(context, "record", DecodeKind::WrongType))?;
        Ok(Self { context, map })
    }

    fn raw(&self, field: &str) -> Result<&'a RhoValue> {
        self.map
            .get(field)
            .ok_or_else(|| SdkError::decode(self.context, field, DecodeKind::Missing))
    }

    fn wrong(&self, field: &str) -> SdkError {
        SdkError::decode(self.context, field, DecodeKind::WrongType)
    }

    fn out_of_range(&self, field: &str) -> SdkError {
        SdkError::decode(self.context, field, DecodeKind::OutOfRange)
    }

    fn int(&self, field: &str) -> Result<i64> {
        self.raw(field)?.as_int().ok_or_else(|| self.wrong(field))
    }

    fn boolean(&self, field: &str) -> Result<bool> {
        self.raw(field)?.as_bool().ok_or_else(|| self.wrong(field))
    }

    fn string(&self, field: &str) -> Result<String> {
        self.raw(field)?
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| self.wrong(field))
    }

    fn optional_string(&self, field: &str) -> Result<Option<String>> {
        match self.map.get(field) {
            None => Ok(None),
            Some(value) => value
                .as_str()
                .map(str::to_string)
                .map(Some)
                .ok_or_else(|| self.wrong(field)),
        }
    }

    fn optional_bool(&self, field: &str) -> Result<Option<bool>> {
        match self.map.get(field) {
            None => Ok(None),
            Some(value) => value.as_bool().map(Some).ok_or_else(|| self.wrong(field)),
        }
    }

    fn height(&self, field: &str) -> Result<Height> {
        Height::new(self.int(field)?).map_err(|_| self.out_of_range(field))
    }

    fn optional_height(&self, field: &str) -> Result<Option<Height>> {
        match self.map.get(field) {
            None => Ok(None),
            Some(value) => {
                let raw = value.as_int().ok_or_else(|| self.wrong(field))?;
                Ok(Some(
                    Height::new(raw).map_err(|_| self.out_of_range(field))?,
                ))
            }
        }
    }

    fn amount(&self, field: &str) -> Result<Amount> {
        Amount::new(self.int(field)?).map_err(|_| self.out_of_range(field))
    }

    fn block_count(&self, field: &str) -> Result<BlockCount> {
        BlockCount::new(self.int(field)?).map_err(|_| self.out_of_range(field))
    }

    fn nonce(&self, field: &str) -> Result<Nonce> {
        Nonce::new(self.int(field)?).map_err(|_| self.out_of_range(field))
    }

    fn basis_points(&self, field: &str) -> Result<BasisPoints> {
        let raw = u16::try_from(self.int(field)?).map_err(|_| self.out_of_range(field))?;
        BasisPoints::new(raw).map_err(|_| self.out_of_range(field))
    }

    fn owner_id(&self, field: &str) -> Result<OwnerId> {
        OwnerId::new(self.string(field)?).map_err(|_| self.out_of_range(field))
    }

    fn registry_uri(&self, field: &str) -> Result<RegistryUri> {
        RegistryUri::new(self.string(field)?).map_err(|_| self.out_of_range(field))
    }

    fn optional_registry_uri(&self, field: &str) -> Result<Option<RegistryUri>> {
        match self.map.get(field) {
            None => Ok(None),
            Some(value) => {
                let text = value.as_str().ok_or_else(|| self.wrong(field))?;
                Ok(Some(
                    RegistryUri::new(text).map_err(|_| self.out_of_range(field))?,
                ))
            }
        }
    }

    fn asset_id(&self, field: &str) -> Result<AssetId> {
        AssetId::new(self.string(field)?).map_err(|_| self.out_of_range(field))
    }

    fn kind(&self, field: &str) -> Result<ResourceKind> {
        let text = self.string(field)?;
        ResourceKind::parse(&text)
            .ok_or_else(|| SdkError::decode(self.context, field, DecodeKind::UnknownTag))
    }

    fn metadata(&self, field: &str) -> Result<OpaqueMetadata> {
        OpaqueMetadata::decode(self.context, field, self.raw(field)?.to_json())
    }

    /// A hash field arrives as a byte array from `rho:crypto:blake2b256Hash`; accept a hex
    /// string too so records written by a different client decode.
    fn bytes_text(&self, field: &str) -> Result<String> {
        match self.raw(field)? {
            RhoValue::Bytes(hex) => Ok(hex.clone()),
            other => other
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| self.wrong(field)),
        }
    }

    fn map(&self, field: &str) -> Result<&'a RhoMap> {
        self.raw(field)?.as_map().ok_or_else(|| self.wrong(field))
    }

    fn list(&self, field: &str) -> Result<&'a [RhoValue]> {
        self.raw(field)?.as_list().ok_or_else(|| self.wrong(field))
    }

    fn json(&self, field: &str) -> Result<Value> {
        Ok(self.raw(field)?.to_json())
    }

    fn admins(&self, field: &str) -> Result<Vec<String>> {
        admin_list(self.context, field, self.raw(field)?)
    }

    fn op_map(&self, field: &str) -> Result<BTreeMap<OpId, Value>> {
        self.map(field)?
            .iter()
            .map(|(id, value)| {
                let op = OpId::new(id.clone()).map_err(|_| {
                    SdkError::decode(
                        self.context,
                        format!("{field}.{id}"),
                        DecodeKind::OutOfRange,
                    )
                })?;
                Ok((op, value.to_json()))
            })
            .collect()
    }

    fn check_schema(&self, field: &str, supported: i64) -> Result<()> {
        let Some(value) = self.map.get(field) else {
            return Ok(());
        };
        let version = value.as_int().ok_or_else(|| self.wrong(field))?;
        if version > supported {
            return Err(SdkError::decode(
                self.context,
                field,
                DecodeKind::UnsupportedSchema,
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used, dead_code)]

    use super::*;

    fn s(value: &str) -> RhoValue {
        RhoValue::Str(value.to_string())
    }

    fn i(value: i64) -> RhoValue {
        RhoValue::Int(value)
    }

    fn b(value: bool) -> RhoValue {
        RhoValue::Bool(value)
    }

    fn list(items: Vec<RhoValue>) -> RhoValue {
        RhoValue::List(items)
    }

    fn bytes(value: &str) -> RhoValue {
        RhoValue::Bytes(value.to_string())
    }

    fn record(entries: Vec<(&str, RhoValue)>) -> RhoValue {
        RhoValue::Map(
            entries
                .into_iter()
                .map(|(key, value)| (key.to_string(), value))
                .collect(),
        )
    }

    fn drop_field(value: &RhoValue, key: &str) -> RhoValue {
        match value {
            RhoValue::Map(map) => RhoValue::Map(
                map.iter()
                    .filter(|(field, _)| field.as_str() != key)
                    .map(|(field, value)| (field.clone(), value.clone()))
                    .collect(),
            ),
            other => other.clone(),
        }
    }

    fn set_field(value: &RhoValue, key: &str, replacement: RhoValue) -> RhoValue {
        match value {
            RhoValue::Map(map) => {
                let mut map = map.clone();
                map.insert(key.to_string(), replacement);
                RhoValue::Map(map)
            }
            other => other.clone(),
        }
    }

    fn terms_rho() -> RhoValue {
        let terms = Terms::default_for(ResourceKind::Robot).unwrap();
        json_to_rho(&terms.to_json())
    }

    fn json_to_rho(value: &Value) -> RhoValue {
        match value {
            Value::Bool(value) => b(*value),
            Value::Number(value) => i(value.as_i64().expect("terms use integers")),
            Value::String(value) => s(value),
            Value::Array(items) => list(items.iter().map(json_to_rho).collect()),
            Value::Object(map) => record(
                map.iter()
                    .map(|(key, value)| (key.as_str(), json_to_rho(value)))
                    .collect(),
            ),
            Value::Null => panic!("the fixture has no null"),
        }
    }

    fn error_kind<T: std::fmt::Debug>(result: Result<T>) -> DecodeKind {
        match result {
            Ok(value) => panic!("expected a decode error, got {value:?}"),
            Err(SdkError::Decode(error)) => error.kind,
            Err(other) => panic!("expected a decode error, got {other:?}"),
        }
    }

    fn provider() -> RhoValue {
        record(vec![
            ("providerId", s("provider1")),
            ("profile", record(vec![("name", s("robot"))])),
            ("status", s("active")),
            ("listings", record(vec![])),
            ("admins", list(vec![bytes("aa"), s("bb")])),
            ("threshold", i(1)),
            ("nonce", i(0)),
        ])
    }

    fn listing_entry() -> RhoValue {
        record(vec![
            ("uri", s("rho:id:listing1")),
            ("kind", s("robot")),
            ("termsHash", bytes("ab")),
            ("status", s("active")),
        ])
    }

    fn unit() -> RhoValue {
        record(vec![
            ("unitId", s("unit1")),
            ("kind", s("robot")),
            ("attrs", record(vec![("model", s("r1"))])),
            ("pricePerSlot", i(100)),
            ("maxHoldBlocks", i(1000)),
            ("status", s("active")),
            (
                "schedule",
                record(vec![(
                    "slot1",
                    record(vec![
                        ("slotId", s("slot1")),
                        ("startHeight", i(10)),
                        ("endHeight", i(20)),
                    ]),
                )]),
            ),
        ])
    }

    fn listing() -> RhoValue {
        record(vec![
            ("listingId", s("listing1")),
            ("providerUri", s("rho:id:provider1")),
            ("providerId", s("provider1")),
            ("kind", s("robot")),
            ("terms", terms_rho()),
            ("termsHash", bytes("ab")),
            ("status", s("active")),
            ("units", record(vec![("unit1", unit())])),
            ("admins", list(vec![bytes("aa")])),
            ("threshold", i(1)),
            ("nonce", i(0)),
        ])
    }

    fn ticket() -> RhoValue {
        record(vec![
            ("holderId", s("guest1")),
            ("redeemed", b(false)),
            ("redeemedAtHeight", i(0)),
            ("transferCutoffHeight", i(20)),
        ])
    }

    fn claim() -> RhoValue {
        record(vec![
            ("amount", i(0)),
            ("evidenceHash", s("")),
            ("atHeight", i(0)),
        ])
    }

    fn booking() -> RhoValue {
        record(vec![
            ("bookingId", s("booking1")),
            ("guestId", s("guest1")),
            ("providerId", s("provider1")),
            ("listingId", s("listing1")),
            ("listingUri", s("rho:id:listing1")),
            ("kind", s("robot")),
            (
                "unitSlots",
                list(vec![record(vec![
                    ("unitId", s("unit1")),
                    ("slots", list(vec![s("slot1")])),
                ])]),
            ),
            ("assetId", s("REV")),
            ("rent", i(100)),
            ("deposit", i(50)),
            ("feeBps", i(250)),
            ("bondUri", s("rho:id:bond1")),
            ("holdUntilHeight", i(100)),
            ("checkInHeight", i(10)),
            ("checkOutHeight", i(20)),
            ("status", s("confirmed")),
            ("cancellation", terms_rho()),
            ("admins", list(vec![bytes("aa")])),
            ("threshold", i(1)),
            ("ticket", ticket()),
            ("claim", claim()),
            (
                "ops",
                record(vec![("book-1", RhoValue::Tuple(vec![b(true), i(0)]))]),
            ),
            ("nonce", i(0)),
            ("schemaVersion", i(2)),
        ])
    }

    #[test]
    fn provider_view_is_strict() {
        let view = ProviderView::from_rho(&provider()).unwrap();
        assert_eq!(view.provider_id.as_str(), "provider1");
        assert_eq!(view.status, ProviderStatus::Active);
        assert_eq!(view.threshold, 1);
        assert_eq!(view.nonce.get(), 0);
        assert_eq!(view.profile.as_value()["name"], "robot");

        assert_eq!(
            error_kind(ProviderView::from_rho(&drop_field(&provider(), "admins"))),
            DecodeKind::Missing
        );
        assert_eq!(
            error_kind(ProviderView::from_rho(&set_field(
                &provider(),
                "providerId",
                i(7)
            ))),
            DecodeKind::WrongType
        );

        let unknown = set_field(&provider(), "status", s("futureState"));
        assert_eq!(
            ProviderView::from_rho(&unknown).unwrap().status,
            ProviderStatus::Unknown("futureState".into())
        );
    }

    #[test]
    fn provider_listing_entry_is_strict() {
        let entry = ProviderListingEntry::from_rho(&listing_entry()).unwrap();
        assert_eq!(entry.uri.as_str(), "rho:id:listing1");
        assert_eq!(entry.kind, ResourceKind::Robot);
        assert_eq!(entry.status, "active");

        assert_eq!(
            error_kind(ProviderListingEntry::from_rho(&drop_field(
                &listing_entry(),
                "uri"
            ))),
            DecodeKind::Missing
        );
        assert_eq!(
            error_kind(ProviderListingEntry::from_rho(&set_field(
                &listing_entry(),
                "status",
                i(1)
            ))),
            DecodeKind::WrongType
        );
    }

    #[test]
    fn listing_view_is_strict() {
        let view = ListingView::from_rho(&listing()).unwrap();
        assert_eq!(view.listing_id, "listing1");
        assert_eq!(view.provider_uri.as_str(), "rho:id:provider1");
        assert_eq!(view.provider_id.as_str(), "provider1");
        assert_eq!(view.units.len(), 1);
        assert_eq!(view.nonce.get(), 0);

        assert_eq!(
            error_kind(ListingView::from_rho(&drop_field(&listing(), "nonce"))),
            DecodeKind::Missing
        );
        assert_eq!(
            error_kind(ListingView::from_rho(&set_field(
                &listing(),
                "units",
                list(vec![])
            ))),
            DecodeKind::WrongType
        );

        let unknown = set_field(&listing(), "status", s("futureState"));
        assert_eq!(
            ListingView::from_rho(&unknown).unwrap().status,
            ListingStatus::Unknown("futureState".into())
        );
    }

    #[test]
    fn unit_view_is_strict() {
        let view = UnitView::from_rho(&unit()).unwrap();
        assert_eq!(view.unit_id, "unit1");
        assert_eq!(view.price_per_slot.get(), 100);
        assert_eq!(view.max_hold_blocks.get(), 1000);
        assert_eq!(view.attrs.as_value()["model"], "r1");
        assert_eq!(view.schedule.len(), 1);

        assert_eq!(
            error_kind(UnitView::from_rho(&drop_field(&unit(), "schedule"))),
            DecodeKind::Missing
        );
        assert_eq!(
            error_kind(UnitView::from_rho(&set_field(&unit(), "attrs", s("nope")))),
            DecodeKind::WrongType
        );
        assert_eq!(
            error_kind(UnitView::from_rho(&set_field(
                &unit(),
                "pricePerSlot",
                i(-1)
            ))),
            DecodeKind::OutOfRange
        );
    }

    #[test]
    fn booking_view_is_strict_and_keeps_optional_fields_absent() {
        let view = BookingView::from_rho(&booking()).unwrap();
        assert_eq!(view.booking_id, "booking1");
        assert_eq!(view.guest_id.as_str(), "guest1");
        assert_eq!(view.provider_id.as_str(), "provider1");
        assert_eq!(view.listing_uri.as_str(), "rho:id:listing1");
        assert_eq!(view.asset_id.as_str(), "REV");
        assert_eq!(view.rent.get(), 100);
        assert_eq!(view.deposit.get(), 50);
        assert_eq!(view.fee_bps.get(), 250);
        assert_eq!(view.hold_until_height.get(), 100);
        assert_eq!(view.check_in_at_height, None);
        assert_eq!(view.check_out_at_height, None);
        assert_eq!(view.deposit_release_height, None);
        assert_eq!(view.dispute_at_height, None);
        assert_eq!(view.no_show_claim_at_height, None);
        assert_eq!(view.reject_reason, None);
        assert!(view.ops.contains_key(&OpId::new("book-1").unwrap()));
        assert_eq!(view.nonce.get(), 0);

        assert_eq!(
            error_kind(BookingView::from_rho(&drop_field(&booking(), "nonce"))),
            DecodeKind::Missing
        );
        assert_eq!(
            error_kind(BookingView::from_rho(&set_field(
                &booking(),
                "rent",
                s("100")
            ))),
            DecodeKind::WrongType
        );
        assert_eq!(
            error_kind(BookingView::from_rho(&set_field(
                &booking(),
                "schemaVersion",
                i(3)
            ))),
            DecodeKind::UnsupportedSchema
        );

        let with_heights = set_field(&booking(), "checkInAtHeight", i(0));
        let with_heights = set_field(&with_heights, "rejectReason", s(""));
        let view = BookingView::from_rho(&with_heights).unwrap();
        assert_eq!(view.check_in_at_height.unwrap().get(), 0);
        assert_eq!(view.reject_reason.as_deref(), Some(""));

        let unknown = set_field(&booking(), "status", s("futureState"));
        assert_eq!(
            BookingView::from_rho(&unknown).unwrap().status,
            BookingStatus::Unknown("futureState".into())
        );
    }

    #[test]
    fn ticket_view_is_strict() {
        let view = TicketView::from_rho(&ticket()).unwrap();
        assert_eq!(view.holder_id.as_str(), "guest1");
        assert!(!view.redeemed);
        assert_eq!(view.transfer_cutoff_height.get(), 20);

        assert_eq!(
            error_kind(TicketView::from_rho(&drop_field(&ticket(), "redeemed"))),
            DecodeKind::Missing
        );
        assert_eq!(
            error_kind(TicketView::from_rho(&set_field(
                &ticket(),
                "redeemedAtHeight",
                s("0")
            ))),
            DecodeKind::WrongType
        );
    }

    #[test]
    fn claim_view_is_strict() {
        let view = ClaimView::from_rho(&claim()).unwrap();
        assert_eq!(view.amount.get(), 0);
        assert_eq!(view.evidence_hash, "");
        assert_eq!(view.at_height.get(), 0);

        assert_eq!(
            error_kind(ClaimView::from_rho(&drop_field(&claim(), "evidenceHash"))),
            DecodeKind::Missing
        );
        assert_eq!(
            error_kind(ClaimView::from_rho(&set_field(&claim(), "amount", s("0")))),
            DecodeKind::WrongType
        );
    }

    #[test]
    fn bond_info_is_strict() {
        let record = record(vec![
            ("paid", b(true)),
            ("expected", i(50)),
            ("remaining", i(20)),
        ]);
        let view = BondInfo::from_rho(&record).unwrap();
        assert!(view.paid);
        assert_eq!(view.expected.get(), 50);
        assert_eq!(view.remaining.get(), 20);

        assert_eq!(
            error_kind(BondInfo::from_rho(&drop_field(&record, "paid"))),
            DecodeKind::Missing
        );
        assert_eq!(
            error_kind(BondInfo::from_rho(&set_field(&record, "expected", s("50")))),
            DecodeKind::WrongType
        );
    }

    #[test]
    fn quote_summary_is_strict() {
        let record = record(vec![
            ("count", i(2)),
            ("rent", i(200)),
            ("minStart", i(10)),
            ("maxEnd", i(20)),
            ("checkIn", i(10)),
            ("checkOut", i(20)),
        ]);
        let view = QuoteSummary::from_rho(&record).unwrap();
        assert_eq!(view.count, 2);
        assert_eq!(view.rent.get(), 200);
        assert_eq!(view.min_start.get(), 10);
        assert_eq!(view.max_end.get(), 20);

        assert_eq!(
            error_kind(QuoteSummary::from_rho(&drop_field(&record, "count"))),
            DecodeKind::Missing
        );
        assert_eq!(
            error_kind(QuoteSummary::from_rho(&set_field(
                &record,
                "rent",
                s("200")
            ))),
            DecodeKind::WrongType
        );
    }

    #[test]
    fn account_view_is_strict() {
        let record = record(vec![("ownerId", s("guest1")), ("revBaseUnits", i(500))]);
        let view = AccountView::from_rho(&record).unwrap();
        assert_eq!(view.owner_id.as_str(), "guest1");
        assert_eq!(view.rev_base_units.get(), 500);

        assert_eq!(
            error_kind(AccountView::from_rho(&drop_field(&record, "ownerId"))),
            DecodeKind::Missing
        );
        assert_eq!(
            error_kind(AccountView::from_rho(&set_field(
                &record,
                "revBaseUnits",
                s("500")
            ))),
            DecodeKind::WrongType
        );
    }

    #[test]
    fn guest_op_distinguishes_pending_from_finished() {
        let pending = GuestOp::from_rho(&record(vec![("pending", b(true))])).unwrap();
        assert!(pending.pending);
        assert_eq!(pending.ok, None);
        assert_eq!(pending.booking_id, None);
        assert_eq!(pending.uri, None);

        let finished = GuestOp::from_rho(&record(vec![
            ("pending", b(false)),
            ("ok", b(true)),
            ("bookingId", s("guest1:book-1")),
            ("uri", s("rho:id:booking1")),
        ]))
        .unwrap();
        assert!(!finished.pending);
        assert_eq!(finished.ok, Some(true));
        assert_eq!(finished.booking_id.as_deref(), Some("guest1:book-1"));
        assert_eq!(finished.uri.as_ref().unwrap().as_str(), "rho:id:booking1");

        let missing_ok = record(vec![("pending", b(false))]);
        assert_eq!(
            error_kind(GuestOp::from_rho(&missing_ok)),
            DecodeKind::Missing
        );
        let wrong_ok = record(vec![("pending", b(false)), ("ok", s("yes"))]);
        assert_eq!(
            error_kind(GuestOp::from_rho(&wrong_ok)),
            DecodeKind::WrongType
        );
        assert_eq!(
            error_kind(GuestOp::from_rho(&record(vec![]))),
            DecodeKind::Missing
        );
    }

    #[test]
    fn slot_states_decode_strictly() {
        assert_eq!(
            slot_state(&list(vec![s("free"), i(3)])).unwrap(),
            SlotState::Free { generation: 3 }
        );
        assert_eq!(
            slot_state(&list(vec![s("held"), s("booking1"), i(50)])).unwrap(),
            SlotState::Held {
                booking_id: "booking1".into(),
                until_height: 50,
            }
        );
        assert_eq!(
            slot_state(&list(vec![s("booked"), s("booking1")])).unwrap(),
            SlotState::Booked {
                booking_id: "booking1".into(),
            }
        );

        assert_eq!(
            error_kind(slot_state(&s("not a list"))),
            DecodeKind::WrongType
        );
        assert_eq!(
            error_kind(slot_state(&list(vec![s("free")]))),
            DecodeKind::Missing
        );
        assert_eq!(
            error_kind(slot_state(&list(vec![s("future"), i(1)]))),
            DecodeKind::UnknownTag
        );
    }

    #[test]
    fn to_json_preserves_the_wire_key_names() {
        let json = BookingView::from_rho(&booking()).unwrap().to_json();
        assert_eq!(json["guestId"], "guest1");
        assert_eq!(json["listingUri"], "rho:id:listing1");
        assert_eq!(json["assetId"], "REV");
        assert_eq!(json["rent"], 100);
        assert_eq!(json["feeBps"], 250);
        assert_eq!(json["checkInAtHeight"], Value::Null);
        assert_eq!(json["rejectReason"], Value::Null);
        assert!(json["ops"].get("book-1").is_some());
    }
}
