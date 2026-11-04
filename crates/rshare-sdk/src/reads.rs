//! Typed reads.
//!
//! State peeks use `explore-deploy`. Arms that take a `deployerId` or call `Clock` use a
//! signed deploy and therefore cost phlo.

use std::collections::BTreeMap;

use rcash_chain::rho_expr::RhoValue;
use rshare_core::render::quote_string;
use rshare_core::slots::render_unit_slots;
use rshare_core::{ComponentKey, RegistryUri, ResourceKind, SlotSpec, Terms, UnitSlotRequest};

use crate::error::{Result, SdkError};
use crate::models::{
    slot_state, AccountView, BondInfo, BookingView, ClaimView, GuestOp, ListingView,
    ProviderListingEntry, ProviderView, QuoteSummary, TicketView, UnitView,
};
use crate::wire::{self, Read, ReadMeta, ReadSource};
use crate::Client;

/// A guest ledger reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuestLedgerRef {
    pub uri: String,
    pub guest_id: String,
}

impl Client {
    async fn read_uri(
        &self,
        uri: &RegistryUri,
        arm: &str,
        args: &[String],
    ) -> Result<(Vec<RhoValue>, ReadMeta)> {
        let term = wire::call_term(uri, arm, args)?;
        let (value, meta) = wire::explore_value(&self.inner, &term).await?;
        let values = wire::decode_reply(&value)?;
        Ok((values, meta))
    }

    /// Some arms (`Terms("fields"/"booking"/"remedy")`) reply with a bare value, not the
    /// `(true, ...)` convention. Read those without `decode_reply`.
    async fn read_raw_uri(
        &self,
        uri: &RegistryUri,
        arm: &str,
        args: &[String],
    ) -> Result<(RhoValue, ReadMeta)> {
        let term = wire::call_term(uri, arm, args)?;
        wire::explore_value(&self.inner, &term).await
    }

    async fn signed_uri(
        &self,
        uri: &RegistryUri,
        arm: &str,
        args: &[String],
    ) -> Result<(Vec<RhoValue>, ReadMeta)> {
        let term = wire::call_term_signed(uri, arm, args)?;
        let receipt = wire::deploy_receipt(&self.inner, &term).await?;
        let reply = receipt
            .reply
            .ok_or_else(|| SdkError::Decode("signed call returned no value".into()))?;
        let values = wire::decode_reply(&reply)?;
        let meta = ReadMeta {
            at_height: receipt.block_number,
            block_hash: receipt.block_hash.unwrap_or_default(),
            source: ReadSource::SignedDeploy,
            deploy_id: Some(receipt.deploy_id),
        };
        Ok((values, meta))
    }

    /// `Terms("fields")`.
    pub async fn terms_fields(&self) -> Result<Read<Vec<String>>> {
        let component = self.resolve(ComponentKey::Terms).await?;
        let (value, meta) = self.read_raw_uri(&component.uri, "fields", &[]).await?;
        let fields = value
            .as_list()
            .ok_or_else(|| SdkError::Decode("fields is not a list".into()))?
            .iter()
            .filter_map(RhoValue::as_str)
            .map(str::to_string)
            .collect();
        Ok(Read {
            value: fields,
            meta,
        })
    }

    /// `Terms("validate")`. The chain is the authority; local validation is advisory.
    pub async fn terms_validate(&self, terms: &Terms) -> Result<Read<()>> {
        let component = self.resolve(ComponentKey::Terms).await?;
        let args = vec![quote_string(terms.kind.as_str())?, terms.to_rho_map()?];
        let (_values, meta) = self.read_uri(&component.uri, "validate", &args).await?;
        Ok(Read { value: (), meta })
    }

    /// `Terms("hash")`.
    pub async fn terms_hash(&self, terms: &Terms) -> Result<Read<String>> {
        let component = self.resolve(ComponentKey::Terms).await?;
        let args = vec![quote_string(terms.kind.as_str())?, terms.to_rho_map()?];
        let (values, meta) = self.read_uri(&component.uri, "hash", &args).await?;
        let hash = hash_text(first(&values, "hash")?)?;
        Ok(Read { value: hash, meta })
    }

    /// `Terms("booking")` projection.
    pub async fn terms_booking(&self, terms: &Terms) -> Result<Read<serde_json::Value>> {
        let component = self.resolve(ComponentKey::Terms).await?;
        let args = vec![terms.to_rho_map()?];
        let (value, meta) = self.read_raw_uri(&component.uri, "booking", &args).await?;
        Ok(Read {
            value: value.to_json(),
            meta,
        })
    }

    /// `Terms("remedy")` projection.
    pub async fn terms_remedy(&self, terms: &Terms) -> Result<Read<serde_json::Value>> {
        let component = self.resolve(ComponentKey::Terms).await?;
        let args = vec![terms.to_rho_map()?];
        let (value, meta) = self.read_raw_uri(&component.uri, "remedy", &args).await?;
        Ok(Read {
            value: value.to_json(),
            meta,
        })
    }

    /// A provider's public state.
    pub async fn provider_info(&self, uri: &RegistryUri) -> Result<Read<ProviderView>> {
        let (values, meta) = self.read_uri(uri, "info", &[]).await?;
        Ok(Read {
            value: ProviderView::from_rho(first(&values, "provider info")?)?,
            meta,
        })
    }

    /// A provider's listing index.
    pub async fn provider_listings(
        &self,
        uri: &RegistryUri,
    ) -> Result<Read<BTreeMap<String, ProviderListingEntry>>> {
        let (values, meta) = self.read_uri(uri, "listings", &[]).await?;
        let map = first(&values, "provider listings")?
            .as_map()
            .ok_or_else(|| SdkError::Decode("listings is not a map".into()))?;
        let listings = map
            .iter()
            .map(|(id, entry)| Ok((id.clone(), ProviderListingEntry::from_rho(entry)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        Ok(Read {
            value: listings,
            meta,
        })
    }

    /// One entry in a provider's listing index.
    pub async fn provider_listing(
        &self,
        uri: &RegistryUri,
        listing_id: &str,
    ) -> Result<Read<ProviderListingEntry>> {
        let args = vec![quote_string(listing_id)?];
        let (values, meta) = self.read_uri(uri, "listing", &args).await?;
        Ok(Read {
            value: ProviderListingEntry::from_rho(first(&values, "provider listing")?)?,
            meta,
        })
    }

    /// `ProviderBook("providerOf", ownerId)`.
    pub async fn provider_of(&self, owner_id: &str) -> Result<Read<String>> {
        let component = self.resolve(ComponentKey::ProviderBook).await?;
        let args = vec![quote_string(owner_id)?];
        let (values, meta) = self.read_uri(&component.uri, "providerOf", &args).await?;
        let uri = first(&values, "providerOf")?
            .as_str()
            .ok_or_else(|| SdkError::Decode("provider uri is not a string".into()))?
            .to_string();
        Ok(Read { value: uri, meta })
    }

    /// A listing and its units. Slot records are not included.
    pub async fn listing_info(&self, uri: &RegistryUri) -> Result<Read<ListingView>> {
        let (values, meta) = self.read_uri(uri, "info", &[]).await?;
        Ok(Read {
            value: ListingView::from_rho(first(&values, "listing info")?)?,
            meta,
        })
    }

    /// A listing's terms map.
    pub async fn listing_terms(
        &self,
        uri: &RegistryUri,
        kind: ResourceKind,
    ) -> Result<Read<Terms>> {
        let (values, meta) = self.read_uri(uri, "terms", &[]).await?;
        let terms = Terms::from_value(kind, &first(&values, "listing terms")?.to_json())?;
        Ok(Read { value: terms, meta })
    }

    /// A listing's units.
    pub async fn listing_units(
        &self,
        uri: &RegistryUri,
    ) -> Result<Read<BTreeMap<String, UnitView>>> {
        let (values, meta) = self.read_uri(uri, "units", &[]).await?;
        let map = first(&values, "units")?
            .as_map()
            .ok_or_else(|| SdkError::Decode("units is not a map".into()))?;
        let units = map
            .iter()
            .map(|(id, unit)| Ok((id.clone(), UnitView::from_rho(unit)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        Ok(Read { value: units, meta })
    }

    /// One unit.
    pub async fn listing_unit(&self, uri: &RegistryUri, unit_id: &str) -> Result<Read<UnitView>> {
        let args = vec![quote_string(unit_id)?];
        let (values, meta) = self.read_uri(uri, "unit", &args).await?;
        Ok(Read {
            value: UnitView::from_rho(first(&values, "unit")?)?,
            meta,
        })
    }

    /// One unit's schedule.
    pub async fn listing_schedule(
        &self,
        uri: &RegistryUri,
        unit_id: &str,
    ) -> Result<Read<BTreeMap<String, SlotSpec>>> {
        let args = vec![quote_string(unit_id)?];
        let (values, meta) = self.read_uri(uri, "schedule", &args).await?;
        let map = first(&values, "schedule")?
            .as_map()
            .ok_or_else(|| SdkError::Decode("schedule is not a map".into()))?;
        let schedule = map
            .iter()
            .map(|(id, spec)| Ok((id.clone(), crate::models::slot_spec(spec)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        Ok(Read {
            value: schedule,
            meta,
        })
    }

    /// Slot states for a set of unit/slot requests. Free read; no release tokens.
    pub async fn listing_probe(
        &self,
        uri: &RegistryUri,
        requests: &[UnitSlotRequest],
    ) -> Result<Read<BTreeMap<String, BTreeMap<String, rshare_core::SlotState>>>> {
        let rendered = render_unit_slots(requests)?;
        let (values, meta) = self.read_uri(uri, "probe", &[rendered]).await?;
        let map = first(&values, "probe")?
            .as_map()
            .ok_or_else(|| SdkError::Decode("probe is not a map".into()))?;
        let mut states = BTreeMap::new();
        for (unit_id, unit_slots) in map {
            let unit_slots = unit_slots
                .as_map()
                .ok_or_else(|| SdkError::Decode(format!("probe unit {unit_id} is not a map")))?;
            let unit = unit_slots
                .iter()
                .map(|(slot_id, state)| (slot_id.clone(), slot_state(state)))
                .collect();
            states.insert(unit_id.clone(), unit);
        }
        Ok(Read {
            value: states,
            meta,
        })
    }

    /// A signed quote. The listing calls `Clock`, so explore is not enough.
    pub async fn listing_quote(
        &self,
        uri: &RegistryUri,
        requests: &[UnitSlotRequest],
    ) -> Result<Read<QuoteSummary>> {
        let rendered = render_unit_slots(requests)?;
        let (values, meta) = self.signed_uri(uri, "quote", &[rendered]).await?;
        Ok(Read {
            value: QuoteSummary::from_rho(first(&values, "quote")?)?,
            meta,
        })
    }

    /// A booking's full state.
    pub async fn booking_info(&self, uri: &RegistryUri) -> Result<Read<BookingView>> {
        let (values, meta) = self.read_uri(uri, "info", &[]).await?;
        Ok(Read {
            value: BookingView::from_rho(first(&values, "booking info")?)?,
            meta,
        })
    }

    /// A booking's status.
    pub async fn booking_status(&self, uri: &RegistryUri) -> Result<Read<String>> {
        let (values, meta) = self.read_uri(uri, "status", &[]).await?;
        let status = first(&values, "status")?
            .as_str()
            .ok_or_else(|| SdkError::Decode("status is not a string".into()))?
            .to_string();
        Ok(Read {
            value: status,
            meta,
        })
    }

    /// A booking's ticket.
    pub async fn booking_ticket(&self, uri: &RegistryUri) -> Result<Read<TicketView>> {
        let (values, meta) = self.read_uri(uri, "ticket", &[]).await?;
        Ok(Read {
            value: TicketView::from_rho(first(&values, "ticket")?)?,
            meta,
        })
    }

    /// A booking's operation log.
    pub async fn booking_ops(
        &self,
        uri: &RegistryUri,
    ) -> Result<Read<BTreeMap<String, serde_json::Value>>> {
        let (values, meta) = self.read_uri(uri, "ops", &[]).await?;
        let map = first(&values, "ops")?
            .as_map()
            .ok_or_else(|| SdkError::Decode("ops is not a map".into()))?;
        let ops = map
            .iter()
            .map(|(id, value)| (id.clone(), value.to_json()))
            .collect();
        Ok(Read { value: ops, meta })
    }

    /// A bond's {paid, expected, remaining}.
    pub async fn bond_info(&self, uri: &RegistryUri) -> Result<Read<BondInfo>> {
        let (values, meta) = self.read_uri(uri, "info", &[]).await?;
        Ok(Read {
            value: BondInfo::from_rho(first(&values, "bond info")?)?,
            meta,
        })
    }

    /// A bond's ERTP balance.
    pub async fn bond_balance(&self, uri: &RegistryUri) -> Result<Read<i64>> {
        let (values, meta) = self.read_uri(uri, "balance", &[]).await?;
        let amount = first(&values, "bond balance")?
            .as_int()
            .ok_or_else(|| SdkError::Decode("bond balance is not an integer".into()))?;
        Ok(Read {
            value: amount,
            meta,
        })
    }

    /// The global admin set and threshold.
    pub async fn share_admin_keys(&self) -> Result<Read<(Vec<String>, i64)>> {
        let component = self.resolve(ComponentKey::ShareAdmin).await?;
        let (values, meta) = self.read_uri(&component.uri, "keys", &[]).await?;
        let admins = crate::models::admin_list(first(&values, "admin keys")?);
        let threshold = values
            .get(1)
            .and_then(RhoValue::as_int)
            .ok_or_else(|| SdkError::Decode("admin threshold is not an integer".into()))?;
        Ok(Read {
            value: (admins, threshold),
            meta,
        })
    }

    /// Whether the global pause is on.
    pub async fn share_admin_is_paused(&self) -> Result<Read<bool>> {
        let component = self.resolve(ComponentKey::ShareAdmin).await?;
        let (values, meta) = self.read_uri(&component.uri, "isPaused", &[]).await?;
        let paused = first(&values, "paused")?
            .as_bool()
            .ok_or_else(|| SdkError::Decode("paused is not a boolean".into()))?;
        Ok(Read {
            value: paused,
            meta,
        })
    }

    /// The r-cash adapter's component versions.
    pub async fn payments_health(&self) -> Result<Read<serde_json::Value>> {
        let component = self.resolve(ComponentKey::Payments).await?;
        let (values, meta) = self.read_uri(&component.uri, "health", &[]).await?;
        Ok(Read {
            value: first(&values, "payments health")?.to_json(),
            meta,
        })
    }

    /// The platform owner id.
    pub async fn payments_platform_owner(&self) -> Result<Read<String>> {
        let component = self.resolve(ComponentKey::Payments).await?;
        let (values, meta) = self.read_uri(&component.uri, "platformOwner", &[]).await?;
        let owner = first(&values, "platform owner")?
            .as_str()
            .ok_or_else(|| SdkError::Decode("platform owner is not a string".into()))?
            .to_string();
        Ok(Read { value: owner, meta })
    }

    /// Native REV balance for an address. Free read.
    pub async fn native_rev_balance(&self, address: &str) -> Result<Read<i64>> {
        let term = wire::native_rev_balance_term(address)?;
        let (value, meta) = wire::explore_value(&self.inner, &term).await?;
        Ok(Read {
            value: value.as_int().unwrap_or_default(),
            meta,
        })
    }

    /// The caller's r-cash account balance. Signed: it takes a `deployerId`.
    pub async fn account_balance(&self) -> Result<Read<AccountView>> {
        let payments = self.resolve(ComponentKey::Payments).await?;
        let term = format!(
            "new rl(`rho:registry:lookup`), paymentsCh, accountCh, balanceCh, \
             deployerId(`rho:rchain:deployerId`), deployId(`rho:rchain:deployId`) in {{ \
             rl!(`{}`, *paymentsCh) | \
             for (Payments <- paymentsCh) {{ \
             Payments!(\"ensureAccount\", *deployerId, *accountCh) | \
             for (@a <- accountCh) {{ \
             match a {{ \
             (true, account, ownerId) => {{ \
             @account!(\"balance\", *deployerId, \"REV\", *balanceCh) | \
             for (@b <- balanceCh) {{ \
             match b {{ \
             (true, amount) => {{ deployId!((true, ownerId, amount)) }} \
             _ => {{ deployId!(b) }} }} }} }} \
             _ => {{ deployId!(a) }} }} }} }} }}",
            payments.uri
        );
        let receipt = wire::deploy_receipt(&self.inner, &term).await?;
        let reply = receipt
            .reply
            .ok_or_else(|| SdkError::Decode("account balance returned no value".into()))?;
        let values = wire::decode_reply(&reply)?;
        let owner_id = first(&values, "account owner")?
            .as_str()
            .ok_or_else(|| SdkError::Decode("account owner is not a string".into()))?
            .to_string();
        let rev_base_units = values
            .get(1)
            .and_then(RhoValue::as_int)
            .ok_or_else(|| SdkError::Decode("account balance is not an integer".into()))?;
        Ok(Read {
            value: AccountView {
                owner_id,
                rev_base_units,
            },
            meta: ReadMeta {
                at_height: receipt.block_number,
                block_hash: receipt.block_hash.unwrap_or_default(),
                source: ReadSource::SignedDeploy,
                deploy_id: Some(receipt.deploy_id),
            },
        })
    }

    /// Get or create the caller's guest ledger.
    pub async fn guest_ledger(&self) -> Result<Read<GuestLedgerRef>> {
        let component = self.resolve(ComponentKey::GuestBook).await?;
        let (values, meta) = self
            .signed_uri(&component.uri, "getOrCreate", &["*deployerId".into()])
            .await?;
        let uri = first(&values, "ledger uri")?
            .as_str()
            .ok_or_else(|| SdkError::Decode("ledger uri is not a string".into()))?
            .to_string();
        let guest_id = values
            .get(1)
            .and_then(RhoValue::as_str)
            .ok_or_else(|| SdkError::Decode("guest id is not a string".into()))?
            .to_string();
        Ok(Read {
            value: GuestLedgerRef { uri, guest_id },
            meta,
        })
    }

    /// The caller's guest-ledger operations. Signed: the arm checks the deployer.
    pub async fn guest_ops(
        &self,
        ledger_uri: &RegistryUri,
    ) -> Result<Read<BTreeMap<String, GuestOp>>> {
        let (values, meta) = self
            .signed_uri(ledger_uri, "ops", &["*deployerId".into()])
            .await?;
        let map = first(&values, "guest ops")?
            .as_map()
            .ok_or_else(|| SdkError::Decode("guest ops is not a map".into()))?;
        let ops = map
            .iter()
            .map(|(id, op)| Ok((id.clone(), GuestOp::from_rho(op)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        Ok(Read { value: ops, meta })
    }
}

fn first<'a>(values: &'a [RhoValue], label: &str) -> Result<&'a RhoValue> {
    values
        .first()
        .ok_or_else(|| SdkError::Decode(format!("{label} reply is empty")))
}

fn hash_text(value: &RhoValue) -> Result<String> {
    match value {
        RhoValue::Bytes(hex) => Ok(hex.clone()),
        other => other
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| SdkError::Decode("hash is not a byte array or string".into())),
    }
}

impl ClaimView {
    /// A default empty claim, for callers that only have a ticket.
    pub fn empty() -> Self {
        Self {
            amount: 0,
            evidence_hash: String::new(),
            at_height: 0,
        }
    }
}
