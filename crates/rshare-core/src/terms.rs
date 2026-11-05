//! The 20-field terms record.
//!
//! `kind` is the first canonical value but is passed beside the terms map on chain. The
//! map itself carries the other 19 fields. Local validation is advisory: the deployed
//! `Terms("validate")` arm is authoritative.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::error::{Result, RshareError};
use crate::primitives::{Amount, BasisPoints, BlockCount};
use crate::render::{quote_string, render_bool, render_i64, render_map};
use rcash_core::assets::AssetId;

/// The canonical field order, exactly as `Terms("fields")` returns it.
pub const FIELDS: [&str; 20] = [
    "kind",
    "assetId",
    "pricePerSlot",
    "deposit",
    "minSlots",
    "maxSlots",
    "instantBook",
    "contiguous",
    "holdBlocks",
    "depositReviewBlocks",
    "disputeWindowBlocks",
    "noShowBlocks",
    "fullRefundBeforeBlocks",
    "partialRefundBps",
    "partialRefundBeforeBlocks",
    "feeBps",
    "maxLeadBlocks",
    "maxSlotBlocks",
    "maxUnits",
    "eventTransferCutoffBlocks",
];

/// The resource classes the chain whitelists.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceKind {
    Lodging,
    Event,
    Bike,
    Car,
    Robot,
}

impl ResourceKind {
    pub const ALL: [ResourceKind; 5] = [
        Self::Lodging,
        Self::Event,
        Self::Bike,
        Self::Car,
        Self::Robot,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lodging => "lodging",
            Self::Event => "event",
            Self::Bike => "bike",
            Self::Car => "car",
            Self::Robot => "robot",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == value)
    }
}

impl std::fmt::Display for ResourceKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl std::str::FromStr for ResourceKind {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        Self::parse(value).ok_or_else(|| format!("unknown resource kind: {value}"))
    }
}

/// A terms validation failure. `Field` carries the contract's refusal key.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TermsError {
    #[error("terms field {0} is missing or out of range")]
    Field(&'static str),
    #[error("resource kind is not one of lodging|event|bike|car|robot")]
    Kind,
    #[error("assetId is empty")]
    Asset,
    #[error("minSlots must not exceed maxSlots")]
    Slots,
    #[error("partialRefundBeforeBlocks must not be below fullRefundBeforeBlocks")]
    RefundWindow,
}

/// The agreed price and policy for a listing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Terms {
    pub kind: ResourceKind,
    pub asset_id: AssetId,
    pub price_per_slot: Amount,
    pub deposit: Amount,
    pub min_slots: i64,
    pub max_slots: i64,
    pub instant_book: bool,
    pub contiguous: bool,
    pub hold_blocks: BlockCount,
    pub deposit_review_blocks: BlockCount,
    pub dispute_window_blocks: BlockCount,
    pub no_show_blocks: BlockCount,
    pub full_refund_before_blocks: BlockCount,
    pub partial_refund_bps: BasisPoints,
    pub partial_refund_before_blocks: BlockCount,
    pub fee_bps: BasisPoints,
    pub max_lead_blocks: BlockCount,
    pub max_slot_blocks: BlockCount,
    pub max_units: i64,
    pub event_transfer_cutoff_blocks: BlockCount,
}

impl Terms {
    /// A conservative starter policy. The values mirror the harness fixture and are
    /// real inputs, not mock prices: a provider must review them before publishing.
    pub fn default_for(kind: ResourceKind) -> Result<Self> {
        Ok(Self {
            kind,
            asset_id: AssetId::new("REV")?,
            price_per_slot: Amount::new(100)?,
            deposit: Amount::new(50)?,
            min_slots: 1,
            max_slots: 4,
            instant_book: true,
            contiguous: false,
            hold_blocks: BlockCount::new(1000)?,
            deposit_review_blocks: BlockCount::new(0)?,
            dispute_window_blocks: BlockCount::new(0)?,
            no_show_blocks: BlockCount::new(0)?,
            full_refund_before_blocks: BlockCount::new(0)?,
            partial_refund_bps: BasisPoints::ZERO,
            partial_refund_before_blocks: BlockCount::new(0)?,
            fee_bps: BasisPoints::ZERO,
            max_lead_blocks: BlockCount::new(1_000_000)?,
            max_slot_blocks: BlockCount::new(1000)?,
            max_units: 2,
            event_transfer_cutoff_blocks: BlockCount::new(0)?,
        })
    }

    /// Parse a terms map. `kind` is supplied separately; a `kind` in the map must agree.
    pub fn from_json(kind: ResourceKind, document: &str) -> Result<Self> {
        let value: Value = serde_json::from_str(document)?;
        Self::from_value(kind, &value)
    }

    /// Parse a terms map from an already-decoded JSON value (for chain replies).
    pub fn from_value(kind: ResourceKind, value: &Value) -> Result<Self> {
        let map = value
            .as_object()
            .ok_or_else(|| RshareError::Json("terms must be a JSON object".into()))?;
        if let Some(map_kind) = map.get("kind") {
            let map_kind = map_kind
                .as_str()
                .and_then(ResourceKind::parse)
                .ok_or(RshareError::Terms(TermsError::Kind))?;
            if map_kind != kind {
                return Err(RshareError::Terms(TermsError::Kind));
            }
        }
        let asset = text(map, "assetId")?;
        let asset_id = AssetId::new(asset).map_err(|_| RshareError::Terms(TermsError::Asset))?;
        Ok(Self {
            kind,
            asset_id,
            price_per_slot: amount(map, "pricePerSlot")?,
            deposit: amount(map, "deposit")?,
            min_slots: int(map, "minSlots")?,
            max_slots: int(map, "maxSlots")?,
            instant_book: boolean(map, "instantBook")?,
            contiguous: boolean(map, "contiguous")?,
            hold_blocks: blocks(map, "holdBlocks")?,
            deposit_review_blocks: blocks(map, "depositReviewBlocks")?,
            dispute_window_blocks: blocks(map, "disputeWindowBlocks")?,
            no_show_blocks: blocks(map, "noShowBlocks")?,
            full_refund_before_blocks: blocks(map, "fullRefundBeforeBlocks")?,
            partial_refund_bps: bps(map, "partialRefundBps")?,
            partial_refund_before_blocks: blocks(map, "partialRefundBeforeBlocks")?,
            fee_bps: bps(map, "feeBps")?,
            max_lead_blocks: blocks(map, "maxLeadBlocks")?,
            max_slot_blocks: blocks(map, "maxSlotBlocks")?,
            max_units: int(map, "maxUnits")?,
            event_transfer_cutoff_blocks: blocks(map, "eventTransferCutoffBlocks")?,
        })
    }

    /// Validate the numeric ranges and cross-field rules the contract enforces.
    ///
    /// This is advisory; always let `Terms("validate")` decide before publishing.
    pub fn validate(&self) -> std::result::Result<(), TermsError> {
        range("pricePerSlot", self.price_per_slot, 1, 1_000_000_000_000)?;
        range("deposit", self.deposit, 0, 1_000_000_000_000)?;
        range("minSlots", self.min_slots, 1, 14)?;
        range("maxSlots", self.max_slots, 1, 14)?;
        range("maxUnits", self.max_units, 1, 8)?;
        range("feeBps", self.fee_bps, 0, 10_000)?;
        range("partialRefundBps", self.partial_refund_bps, 0, 10_000)?;
        range("holdBlocks", self.hold_blocks, 1, 1000)?;
        range(
            "depositReviewBlocks",
            self.deposit_review_blocks,
            0,
            100_000,
        )?;
        range(
            "disputeWindowBlocks",
            self.dispute_window_blocks,
            0,
            100_000,
        )?;
        range("noShowBlocks", self.no_show_blocks, 0, 100_000)?;
        range(
            "fullRefundBeforeBlocks",
            self.full_refund_before_blocks,
            0,
            100_000,
        )?;
        range(
            "partialRefundBeforeBlocks",
            self.partial_refund_before_blocks,
            0,
            100_000,
        )?;
        range("maxLeadBlocks", self.max_lead_blocks, 1, 10_000_000)?;
        range("maxSlotBlocks", self.max_slot_blocks, 1, 10_000_000)?;
        range(
            "eventTransferCutoffBlocks",
            self.event_transfer_cutoff_blocks,
            0,
            100_000,
        )?;
        if self.min_slots > self.max_slots {
            return Err(TermsError::Slots);
        }
        if self.partial_refund_before_blocks.get() < self.full_refund_before_blocks.get() {
            return Err(TermsError::RefundWindow);
        }
        Ok(())
    }

    /// The 20 canonical values, `kind` first, exactly as `Terms("values")` builds them.
    pub fn canonical_values(&self) -> Vec<String> {
        vec![
            self.kind.to_string(),
            self.asset_id.to_string(),
            render_i64(self.price_per_slot),
            render_i64(self.deposit),
            render_i64(self.min_slots),
            render_i64(self.max_slots),
            render_bool(self.instant_book),
            render_bool(self.contiguous),
            render_i64(self.hold_blocks),
            render_i64(self.deposit_review_blocks),
            render_i64(self.dispute_window_blocks),
            render_i64(self.no_show_blocks),
            render_i64(self.full_refund_before_blocks),
            render_i64(self.partial_refund_bps),
            render_i64(self.partial_refund_before_blocks),
            render_i64(self.fee_bps),
            render_i64(self.max_lead_blocks),
            render_i64(self.max_slot_blocks),
            render_i64(self.max_units),
            render_i64(self.event_transfer_cutoff_blocks),
        ]
    }

    /// Render the 19-field terms map passed as the `@terms` argument.
    pub fn to_rho_map(&self) -> Result<String> {
        self.validate()?;
        render_map(&[
            ("assetId", quote_string(self.asset_id.as_str())?),
            ("pricePerSlot", render_i64(self.price_per_slot)),
            ("deposit", render_i64(self.deposit)),
            ("minSlots", render_i64(self.min_slots)),
            ("maxSlots", render_i64(self.max_slots)),
            ("instantBook", render_bool(self.instant_book)),
            ("contiguous", render_bool(self.contiguous)),
            ("holdBlocks", render_i64(self.hold_blocks)),
            (
                "depositReviewBlocks",
                render_i64(self.deposit_review_blocks),
            ),
            (
                "disputeWindowBlocks",
                render_i64(self.dispute_window_blocks),
            ),
            ("noShowBlocks", render_i64(self.no_show_blocks)),
            (
                "fullRefundBeforeBlocks",
                render_i64(self.full_refund_before_blocks),
            ),
            ("partialRefundBps", render_i64(self.partial_refund_bps)),
            (
                "partialRefundBeforeBlocks",
                render_i64(self.partial_refund_before_blocks),
            ),
            ("feeBps", render_i64(self.fee_bps)),
            ("maxLeadBlocks", render_i64(self.max_lead_blocks)),
            ("maxSlotBlocks", render_i64(self.max_slot_blocks)),
            ("maxUnits", render_i64(self.max_units)),
            (
                "eventTransferCutoffBlocks",
                render_i64(self.event_transfer_cutoff_blocks),
            ),
        ])
    }

    /// The 19-field JSON map used by CLI `--terms` and the web form.
    pub fn to_map_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("assetId".into(), Value::from(self.asset_id.as_str()));
        map.insert(
            "pricePerSlot".into(),
            Value::from(self.price_per_slot.get()),
        );
        map.insert("deposit".into(), Value::from(self.deposit.get()));
        map.insert("minSlots".into(), Value::from(self.min_slots));
        map.insert("maxSlots".into(), Value::from(self.max_slots));
        map.insert("instantBook".into(), Value::from(self.instant_book));
        map.insert("contiguous".into(), Value::from(self.contiguous));
        map.insert("holdBlocks".into(), Value::from(self.hold_blocks.get()));
        map.insert(
            "depositReviewBlocks".into(),
            Value::from(self.deposit_review_blocks.get()),
        );
        map.insert(
            "disputeWindowBlocks".into(),
            Value::from(self.dispute_window_blocks.get()),
        );
        map.insert(
            "noShowBlocks".into(),
            Value::from(self.no_show_blocks.get()),
        );
        map.insert(
            "fullRefundBeforeBlocks".into(),
            Value::from(self.full_refund_before_blocks.get()),
        );
        map.insert(
            "partialRefundBps".into(),
            Value::from(self.partial_refund_bps.get()),
        );
        map.insert(
            "partialRefundBeforeBlocks".into(),
            Value::from(self.partial_refund_before_blocks.get()),
        );
        map.insert("feeBps".into(), Value::from(self.fee_bps.get()));
        map.insert(
            "maxLeadBlocks".into(),
            Value::from(self.max_lead_blocks.get()),
        );
        map.insert(
            "maxSlotBlocks".into(),
            Value::from(self.max_slot_blocks.get()),
        );
        map.insert("maxUnits".into(), Value::from(self.max_units));
        map.insert(
            "eventTransferCutoffBlocks".into(),
            Value::from(self.event_transfer_cutoff_blocks.get()),
        );
        Value::Object(map)
    }

    /// The display form, including `kind`.
    pub fn to_json(&self) -> Value {
        let mut value = self.to_map_json();
        if let Value::Object(map) = &mut value {
            map.insert("kind".into(), Value::from(self.kind.as_str()));
        }
        value
    }
}

fn range(
    field: &'static str,
    value: impl Into<i64>,
    min: i64,
    max: i64,
) -> std::result::Result<(), TermsError> {
    let value = value.into();
    if value < min || value > max {
        Err(TermsError::Field(field))
    } else {
        Ok(())
    }
}

fn int(map: &Map<String, Value>, field: &'static str) -> Result<i64> {
    map.get(field)
        .and_then(Value::as_i64)
        .ok_or(RshareError::Terms(TermsError::Field(field)))
}

fn amount(map: &Map<String, Value>, field: &'static str) -> Result<Amount> {
    Amount::new(int(map, field)?).map_err(|_| RshareError::Terms(TermsError::Field(field)))
}

fn blocks(map: &Map<String, Value>, field: &'static str) -> Result<BlockCount> {
    BlockCount::new(int(map, field)?).map_err(|_| RshareError::Terms(TermsError::Field(field)))
}

fn bps(map: &Map<String, Value>, field: &'static str) -> Result<BasisPoints> {
    let raw = u16::try_from(int(map, field)?)
        .map_err(|_| RshareError::Terms(TermsError::Field(field)))?;
    BasisPoints::new(raw).map_err(|_| RshareError::Terms(TermsError::Field(field)))
}

fn boolean(map: &Map<String, Value>, field: &'static str) -> Result<bool> {
    map.get(field)
        .and_then(Value::as_bool)
        .ok_or(RshareError::Terms(TermsError::Field(field)))
}

fn text<'a>(map: &'a Map<String, Value>, field: &'static str) -> Result<&'a str> {
    map.get(field)
        .and_then(Value::as_str)
        .ok_or(RshareError::Terms(TermsError::Field(field)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn robot() -> Terms {
        Terms::default_for(ResourceKind::Robot).unwrap()
    }

    #[test]
    fn defaults_are_valid_and_render_the_full_map() {
        let terms = robot();
        terms.validate().unwrap();
        let map = terms.to_rho_map().unwrap();
        for field in [
            "assetId",
            "pricePerSlot",
            "deposit",
            "minSlots",
            "maxSlots",
            "instantBook",
            "contiguous",
            "holdBlocks",
            "depositReviewBlocks",
            "disputeWindowBlocks",
            "noShowBlocks",
            "fullRefundBeforeBlocks",
            "partialRefundBps",
            "partialRefundBeforeBlocks",
            "feeBps",
            "maxLeadBlocks",
            "maxSlotBlocks",
            "maxUnits",
            "eventTransferCutoffBlocks",
        ] {
            assert!(map.contains(&format!("\"{field}\":")), "missing {field}");
        }
        assert!(!map.contains("\"kind\":"));
    }

    #[test]
    fn canonical_values_start_with_kind_and_are_twenty_long() {
        let values = robot().canonical_values();
        assert_eq!(values.len(), 20);
        assert_eq!(values[0], "robot");
        assert_eq!(values[1], "REV");
        assert_eq!(values[6], "true");
    }

    #[test]
    fn parses_the_fixture_shape_and_keeps_kind_separate() {
        let terms = Terms::from_json(
            ResourceKind::Lodging,
            r#"{"assetId":"REV","pricePerSlot":100,"deposit":50,"minSlots":1,
                "maxSlots":4,"instantBook":true,"contiguous":false,"holdBlocks":1000,
                "depositReviewBlocks":0,"disputeWindowBlocks":0,"noShowBlocks":0,
                "fullRefundBeforeBlocks":0,"partialRefundBps":0,
                "partialRefundBeforeBlocks":0,"feeBps":0,"maxLeadBlocks":1000000,
                "maxSlotBlocks":1000,"maxUnits":2,"eventTransferCutoffBlocks":0}"#,
        )
        .unwrap();
        assert_eq!(terms.kind, ResourceKind::Lodging);
        assert_eq!(terms.price_per_slot.get(), 100);
    }

    #[test]
    fn rejects_bad_ranges_and_cross_field_rules() {
        let mut terms = robot();
        terms.min_slots = 5;
        terms.max_slots = 4;
        assert_eq!(terms.validate(), Err(TermsError::Slots));

        let mut terms = robot();
        terms.price_per_slot = Amount::new(0).unwrap();
        assert_eq!(terms.validate(), Err(TermsError::Field("pricePerSlot")));

        let mut terms = robot();
        terms.full_refund_before_blocks = BlockCount::new(10).unwrap();
        terms.partial_refund_before_blocks = BlockCount::new(5).unwrap();
        assert_eq!(terms.validate(), Err(TermsError::RefundWindow));
    }

    #[test]
    fn refuses_a_mismatched_kind_in_the_map() {
        let document = robot().to_json().to_string();
        assert!(Terms::from_json(ResourceKind::Lodging, &document).is_err());
    }

    #[test]
    fn rejects_missing_and_non_integer_fields() {
        let mut value = json!({});
        value["assetId"] = json!("REV");
        assert!(Terms::from_json(ResourceKind::Robot, &value.to_string()).is_err());

        let mut value = robot().to_map_json();
        value["deposit"] = json!("50");
        assert!(Terms::from_json(ResourceKind::Robot, &value.to_string()).is_err());
    }
}
