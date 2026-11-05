//! Slot specifications and booking requests.

use serde::{Deserialize, Serialize};

use crate::error::{Result, RshareError};
use crate::primitives::Height;
use crate::render::{quote_string, render_i64, render_map};

/// One slot in a unit's schedule.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlotSpec {
    pub slot_id: String,
    pub start_height: Height,
    pub end_height: Height,
}

impl SlotSpec {
    pub fn validate(&self) -> Result<()> {
        validate_id("slot id", &self.slot_id, 64)?;
        if self.end_height <= self.start_height {
            return Err(RshareError::Slots(format!(
                "slot {} ends before it starts",
                self.slot_id
            )));
        }
        Ok(())
    }

    /// Render `{"slotId": "...", "startHeight": N, "endHeight": M}`.
    pub fn to_rho(&self) -> Result<String> {
        self.validate()?;
        render_map(&[
            ("slotId", quote_string(&self.slot_id)?),
            ("startHeight", render_i64(self.start_height)),
            ("endHeight", render_i64(self.end_height)),
        ])
    }
}

/// One unit's requested slots. Check-in/out heights are computed by the listing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitSlotRequest {
    pub unit_id: String,
    pub slots: Vec<String>,
}

impl UnitSlotRequest {
    /// Render `{"unitId": "...", "slots": ["s1", "s2"]}`.
    pub fn to_rho(&self) -> Result<String> {
        self.validate()?;
        let slots = self
            .slots
            .iter()
            .map(|slot| quote_string(slot))
            .collect::<Result<Vec<_>>>()?;
        render_map(&[
            ("unitId", quote_string(&self.unit_id)?),
            ("slots", format!("[{}]", slots.join(", "))),
        ])
    }

    pub fn validate(&self) -> Result<()> {
        validate_id("unit id", &self.unit_id, 96)?;
        if self.slots.is_empty() {
            return Err(RshareError::Slots(format!(
                "unit {} has no requested slots",
                self.unit_id
            )));
        }
        let mut seen = std::collections::BTreeSet::new();
        for slot in &self.slots {
            validate_id("slot id", slot, 64)?;
            if !seen.insert(slot) {
                return Err(RshareError::Slots(format!(
                    "duplicate slot {slot} in unit {}",
                    self.unit_id
                )));
            }
        }
        Ok(())
    }
}

/// Validate a whole booking request against the terms' unit cap.
pub fn validate_unit_slots(requests: &[UnitSlotRequest], max_units: i64) -> Result<()> {
    if requests.is_empty() {
        return Err(RshareError::Slots("no units requested".into()));
    }
    if requests.len() as i64 > max_units {
        return Err(RshareError::Slots(format!(
            "{} units requested; the listing allows {max_units}",
            requests.len()
        )));
    }
    let mut units = std::collections::BTreeSet::new();
    for request in requests {
        request.validate()?;
        if !units.insert(&request.unit_id) {
            return Err(RshareError::Slots(format!(
                "duplicate unit {}",
                request.unit_id
            )));
        }
    }
    Ok(())
}

/// Render a list of unit-slot requests for `quote`, `acquire`, `probe` and `book`.
pub fn render_unit_slots(requests: &[UnitSlotRequest]) -> Result<String> {
    let rendered = requests
        .iter()
        .map(UnitSlotRequest::to_rho)
        .collect::<Result<Vec<_>>>()?;
    Ok(format!("[{}]", rendered.join(", ")))
}

/// Render a unit schedule for `ensureUnit`.
pub fn render_slot_specs(specs: &[SlotSpec]) -> Result<String> {
    let mut seen = std::collections::BTreeSet::new();
    let rendered = specs
        .iter()
        .map(|spec| {
            spec.validate()?;
            if !seen.insert(&spec.slot_id) {
                return Err(RshareError::Slots(format!(
                    "duplicate slot id {}",
                    spec.slot_id
                )));
            }
            spec.to_rho()
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(format!("[{}]", rendered.join(", ")))
}

fn validate_id(label: &str, value: &str, max: usize) -> Result<()> {
    let valid = !value.is_empty()
        && value.len() <= max
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ":/_-.".contains(character));
    if valid {
        Ok(())
    } else {
        Err(RshareError::Slots(format!(
            "{label} must be 1..={max} ASCII alphanumerics or :/_-., got {value:?}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_the_expected_booking_shape() {
        let request = UnitSlotRequest {
            unit_id: "unit:1".into(),
            slots: vec!["s1".into(), "s2".into()],
        };
        assert_eq!(
            request.to_rho().unwrap(),
            "{\"unitId\": \"unit:1\", \"slots\": [\"s1\", \"s2\"]}"
        );
    }

    #[test]
    fn validates_the_unit_cap_and_duplicates() {
        let one = UnitSlotRequest {
            unit_id: "u1".into(),
            slots: vec!["s1".into()],
        };
        let two = UnitSlotRequest {
            unit_id: "u2".into(),
            slots: vec!["s2".into()],
        };
        assert!(validate_unit_slots(&[one.clone(), two], 2).is_ok());
        assert!(validate_unit_slots(std::slice::from_ref(&one), 0).is_err());
        assert!(validate_unit_slots(&[one.clone(), one.clone()], 2).is_err());

        let duplicate_slots = UnitSlotRequest {
            unit_id: "u1".into(),
            slots: vec!["s1".into(), "s1".into()],
        };
        assert!(duplicate_slots.validate().is_err());
    }

    #[test]
    fn refuses_unrepresentable_ids_and_bad_windows() {
        let bad = UnitSlotRequest {
            unit_id: "u\"1".into(),
            slots: vec!["s1".into()],
        };
        assert!(bad.validate().is_err());

        let bad = SlotSpec {
            slot_id: "s1".into(),
            start_height: Height::new(10).unwrap(),
            end_height: Height::new(10).unwrap(),
        };
        assert!(bad.validate().is_err());
    }
}
