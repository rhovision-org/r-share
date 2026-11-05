//! WASM facade for the browser wallet and UI.
//!
//! The private key lives only in this module. JS receives the encrypted keystore JSON,
//! public identifiers and operation results — never a secret.

use std::fmt::Display;

use rcash_chain::address::eth_address_from_public_key;
use rcash_chain::KeyPair;
use rshare_core::render::quote_string;
use rshare_core::render::render_json;
use rshare_core::{
    BasisPoints, Manifest, Nonce, OpId, RegistryUri, ResourceKind, SlotSpec, Terms, UnitSlotRequest,
};
use rshare_sdk::{BookingAction, Client, DeployReceipt, ReadMeta, ReadSource};
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

/// The crate version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Sign an admin payload with a fixed secret (test-only parity fixture).
#[wasm_bindgen]
pub fn parity_sign(secret_hex: &str, payload: &str) -> Result<String, JsValue> {
    let key = KeyPair::from_secret_hex(secret_hex).map_err(js)?;
    let digest = rchain_crypto::hash::blake2b256::hash(payload.as_bytes());
    let signature = key.sign_prehash(&digest).map_err(js)?;
    Ok(signature.iter().map(|byte| format!("{byte:02x}")).collect())
}

/// A connected wallet and SDK client.
#[wasm_bindgen]
pub struct Rshare {
    node: String,
    manifest: Manifest,
    shard: String,
    phlo_limit: i64,
    phlo_price: i64,
    client: Client,
}

#[wasm_bindgen]
impl Rshare {
    /// Create a reader client. Call `unlock` or `create_wallet` to sign.
    #[wasm_bindgen(constructor)]
    pub fn new(node: &str, manifest_json: &str) -> Result<Rshare, JsValue> {
        let manifest = Manifest::from_json(manifest_json).map_err(js)?;
        let client = Client::reader(node, manifest.clone()).map_err(js)?;
        Ok(Self {
            node: node.to_string(),
            manifest,
            shard: Client::DEFAULT_SHARD.to_string(),
            phlo_limit: Client::DEFAULT_PHLO_LIMIT,
            phlo_price: Client::DEFAULT_PHLO_PRICE,
            client,
        })
    }

    /// Generate a key, unlock it, and return the encrypted keystore JSON.
    pub fn create_wallet(&mut self, password: &str) -> Result<String, JsValue> {
        let key = KeyPair::generate().map_err(js)?;
        let eth = eth_address_from_public_key(key.public_bytes()).map_err(js)?;
        let document = rshare_keystore::encrypt(key.secret_bytes(), &eth, password).map_err(js)?;
        self.rebuild(key)?;
        Ok(document)
    }

    /// Decrypt a keystore and unlock the wallet. Returns the REV address.
    pub fn unlock(&mut self, document: &str, password: &str) -> Result<String, JsValue> {
        let secret = rshare_keystore::decrypt(document, password).map_err(js)?;
        let key = KeyPair::from_secret_bytes(&secret).map_err(js)?;
        let address = key.rev_address().map_err(js)?;
        self.rebuild(key)?;
        Ok(address)
    }

    /// Drop the key from memory and keep public reads available.
    pub fn lock(&mut self) -> Result<(), JsValue> {
        self.client = Client::reader(&self.node, self.manifest.clone()).map_err(js)?;
        self.client
            .set_network(&self.shard, self.phlo_limit, self.phlo_price);
        Ok(())
    }

    pub fn is_unlocked(&self) -> bool {
        self.client.has_key()
    }

    pub fn address(&self) -> Option<String> {
        self.client.address().ok()
    }

    pub fn public_key(&self) -> Option<String> {
        self.client.key().map(|key| key.public_hex())
    }

    /// Re-encrypt the current key with a (possibly new) password.
    pub fn export_keystore(&self, password: &str) -> Result<String, JsValue> {
        let key = self.client.key().ok_or_else(|| js("wallet is locked"))?;
        let eth = eth_address_from_public_key(key.public_bytes()).map_err(js)?;
        rshare_keystore::encrypt(key.secret_bytes(), &eth, password).map_err(js)
    }

    /// Sign a canonical admin payload. Returns lowercase hex DER.
    pub fn sign_payload(&self, payload: &str) -> Result<String, JsValue> {
        self.client.admin_sign(payload).map_err(js)
    }

    /// BLAKE2b-256 of a UTF-8 string, hex. Use it to anchor evidence without sending the
    /// text on-chain.
    pub fn hash_text(&self, text: &str) -> String {
        let digest = rchain_crypto::hash::blake2b256::hash(text.as_bytes());
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    /// Generate a fresh operation id. JS supplies the timestamp (`Date.now()`).
    pub fn new_op_id(&self, prefix: &str, now_ms: i64) -> Result<String, JsValue> {
        Ok(rshare_sdk::new_op_id(prefix, now_ms)
            .map_err(js)?
            .as_str()
            .to_string())
    }

    pub fn set_network(&mut self, shard: &str, phlo_limit: i64, phlo_price: i64) {
        self.shard = shard.to_string();
        self.phlo_limit = phlo_limit;
        self.phlo_price = phlo_price;
        self.client.set_network(shard, phlo_limit, phlo_price);
    }

    pub async fn height(&self) -> Result<String, JsValue> {
        let height = self.client.height().await.map_err(js)?;
        Ok(height.to_string())
    }

    pub async fn status(&self) -> Result<String, JsValue> {
        let status = self.client.status().await.map_err(js)?;
        Ok(status.to_string())
    }

    pub async fn terms_fields(&self) -> Result<String, JsValue> {
        let read = self.client.terms_fields().await.map_err(js)?;
        Ok(read_out(json!(read.value), &read.meta))
    }

    pub async fn terms_validate(&self, kind: &str, terms_json: &str) -> Result<String, JsValue> {
        let terms = parse_terms(kind, terms_json)?;
        let read = self.client.terms_validate(&terms).await.map_err(js)?;
        Ok(read_out(json!(true), &read.meta))
    }

    pub async fn terms_hash(&self, kind: &str, terms_json: &str) -> Result<String, JsValue> {
        let terms = parse_terms(kind, terms_json)?;
        let read = self.client.terms_hash(&terms).await.map_err(js)?;
        Ok(read_out(json!(read.value), &read.meta))
    }

    pub fn terms_show(&self, kind: &str, terms_json: &str) -> Result<String, JsValue> {
        let terms = parse_terms(kind, terms_json)?;
        Ok(terms.to_json().to_string())
    }

    pub async fn provider_info(&self, uri: &str) -> Result<String, JsValue> {
        let read = self
            .client
            .provider_info(&parse_uri(uri)?)
            .await
            .map_err(js)?;
        Ok(read_out(read.value.to_json(), &read.meta))
    }

    pub async fn provider_listings(&self, uri: &str) -> Result<String, JsValue> {
        let read = self
            .client
            .provider_listings(&parse_uri(uri)?)
            .await
            .map_err(js)?;
        let data = read
            .value
            .iter()
            .map(|(id, entry)| (id.clone(), entry.to_json()))
            .collect::<serde_json::Map<_, _>>();
        Ok(read_out(json!(data), &read.meta))
    }

    pub async fn provider_of(&self, owner: &str) -> Result<String, JsValue> {
        let read = self.client.provider_of(owner).await.map_err(js)?;
        Ok(read_out(json!(read.value), &read.meta))
    }

    pub async fn register_provider(&self, profile_json: &str) -> Result<String, JsValue> {
        let profile = parse_json(profile_json, "profile")?;
        let write = self.client.register_provider(&profile).await.map_err(js)?;
        Ok(write_out(
            json!({"providerUri": write.value.0, "ownerId": write.value.1}),
            &write.receipt,
        ))
    }

    pub async fn update_profile(
        &self,
        uri: &str,
        profile_json: &str,
        op_id: &str,
    ) -> Result<String, JsValue> {
        let profile = parse_json(profile_json, "profile")?;
        let write = self
            .client
            .update_profile(&parse_uri(uri)?, &profile, &parse_op(op_id)?)
            .await
            .map_err(js)?;
        Ok(write_out(json!({}), &write.receipt))
    }

    pub async fn provider_pause(&self, uri: &str) -> Result<String, JsValue> {
        let write = self
            .client
            .provider_pause(&parse_uri(uri)?)
            .await
            .map_err(js)?;
        Ok(write_out(json!({}), &write.receipt))
    }

    pub async fn provider_resume(&self, uri: &str) -> Result<String, JsValue> {
        let write = self
            .client
            .provider_resume(&parse_uri(uri)?)
            .await
            .map_err(js)?;
        Ok(write_out(json!({}), &write.receipt))
    }

    pub async fn listing_info(&self, uri: &str) -> Result<String, JsValue> {
        let read = self
            .client
            .listing_info(&parse_uri(uri)?)
            .await
            .map_err(js)?;
        Ok(read_out(read.value.to_json(), &read.meta))
    }

    pub async fn listing_units(&self, uri: &str) -> Result<String, JsValue> {
        let read = self
            .client
            .listing_units(&parse_uri(uri)?)
            .await
            .map_err(js)?;
        let data = read
            .value
            .iter()
            .map(|(id, unit)| (id.clone(), unit.to_json()))
            .collect::<serde_json::Map<_, _>>();
        Ok(read_out(json!(data), &read.meta))
    }

    pub async fn listing_schedule(&self, uri: &str, unit_id: &str) -> Result<String, JsValue> {
        let read = self
            .client
            .listing_schedule(&parse_uri(uri)?, unit_id)
            .await
            .map_err(js)?;
        Ok(read_out(json!(read.value), &read.meta))
    }

    pub async fn probe(&self, uri: &str, unit_slots_json: &str) -> Result<String, JsValue> {
        let requests = parse_unit_slots(unit_slots_json)?;
        let read = self
            .client
            .listing_probe(&parse_uri(uri)?, &requests)
            .await
            .map_err(js)?;
        let data = read
            .value
            .iter()
            .map(|(unit, slots)| {
                let slots: serde_json::Map<String, Value> = slots
                    .iter()
                    .map(|(slot, state)| (slot.clone(), state.to_json()))
                    .collect();
                (unit.clone(), Value::Object(slots))
            })
            .collect::<serde_json::Map<_, _>>();
        Ok(read_out(json!(data), &read.meta))
    }

    pub async fn quote(&self, uri: &str, unit_slots_json: &str) -> Result<String, JsValue> {
        let requests = parse_unit_slots(unit_slots_json)?;
        let read = self
            .client
            .listing_quote(&parse_uri(uri)?, &requests)
            .await
            .map_err(js)?;
        Ok(read_out(read.value.to_json(), &read.meta))
    }

    pub async fn create_listing(
        &self,
        provider_uri: &str,
        listing_id: &str,
        kind: &str,
        terms_json: &str,
    ) -> Result<String, JsValue> {
        let terms = parse_terms(kind, terms_json)?;
        let write = self
            .client
            .create_listing(
                &parse_uri(provider_uri)?,
                listing_id,
                parse_kind(kind)?,
                &terms,
            )
            .await
            .map_err(js)?;
        Ok(write_out(
            json!({"listingUri": write.value}),
            &write.receipt,
        ))
    }

    pub async fn ensure_unit(
        &self,
        listing_uri: &str,
        unit_id: &str,
        attrs_json: &str,
        slots_json: &str,
        price_per_slot: i64,
    ) -> Result<String, JsValue> {
        let attrs = parse_json(attrs_json, "attrs")?;
        let slots: Vec<SlotSpec> = serde_json::from_str(slots_json).map_err(js)?;
        let write = self
            .client
            .ensure_unit(
                &parse_uri(listing_uri)?,
                unit_id,
                &attrs,
                &slots,
                price_per_slot,
            )
            .await
            .map_err(js)?;
        Ok(write_out(json!({"unitId": write.value}), &write.receipt))
    }

    pub async fn activate_listing(&self, uri: &str) -> Result<String, JsValue> {
        let write = self
            .client
            .activate_listing(&parse_uri(uri)?)
            .await
            .map_err(js)?;
        Ok(write_out(json!({}), &write.receipt))
    }

    pub async fn pause_listing(&self, uri: &str) -> Result<String, JsValue> {
        let write = self
            .client
            .pause_listing(&parse_uri(uri)?)
            .await
            .map_err(js)?;
        Ok(write_out(json!({}), &write.receipt))
    }

    pub async fn resume_listing(&self, uri: &str) -> Result<String, JsValue> {
        let write = self
            .client
            .resume_listing(&parse_uri(uri)?)
            .await
            .map_err(js)?;
        Ok(write_out(json!({}), &write.receipt))
    }

    pub async fn delist_listing(&self, uri: &str) -> Result<String, JsValue> {
        let write = self
            .client
            .delist_listing(&parse_uri(uri)?)
            .await
            .map_err(js)?;
        Ok(write_out(json!({}), &write.receipt))
    }

    pub async fn pause_unit(&self, uri: &str, unit_id: &str) -> Result<String, JsValue> {
        let write = self
            .client
            .pause_unit(&parse_uri(uri)?, unit_id)
            .await
            .map_err(js)?;
        Ok(write_out(json!({}), &write.receipt))
    }

    pub async fn resume_unit(&self, uri: &str, unit_id: &str) -> Result<String, JsValue> {
        let write = self
            .client
            .resume_unit(&parse_uri(uri)?, unit_id)
            .await
            .map_err(js)?;
        Ok(write_out(json!({}), &write.receipt))
    }

    pub async fn reset_unit(
        &self,
        uri: &str,
        unit_id: &str,
        slot_id: &str,
    ) -> Result<String, JsValue> {
        let write = self
            .client
            .reset_unit(&parse_uri(uri)?, unit_id, slot_id)
            .await
            .map_err(js)?;
        Ok(write_out(json!({}), &write.receipt))
    }

    pub async fn book(
        &self,
        listing_uri: &str,
        unit_slots_json: &str,
        op_id: &str,
        expected_terms_hash: &str,
    ) -> Result<String, JsValue> {
        let requests = parse_unit_slots(unit_slots_json)?;
        let expected = if expected_terms_hash.is_empty() {
            None
        } else {
            Some(expected_terms_hash)
        };
        let outcome = self
            .client
            .book(
                &parse_uri(listing_uri)?,
                &requests,
                &parse_op(op_id)?,
                expected,
            )
            .await
            .map_err(js)?;
        Ok(write_out(
            json!({
                "bookingId": outcome.booking_id,
                "bookingUri": outcome.booking_uri,
                "ledgerUri": outcome.ledger_uri,
                "guestId": outcome.guest_id,
            }),
            &outcome.receipt,
        ))
    }

    pub async fn booking_info(&self, uri: &str) -> Result<String, JsValue> {
        let read = self
            .client
            .booking_info(&parse_uri(uri)?)
            .await
            .map_err(js)?;
        Ok(read_out(read.value.to_json(), &read.meta))
    }

    pub async fn booking_status(&self, uri: &str) -> Result<String, JsValue> {
        let read = self
            .client
            .booking_status(&parse_uri(uri)?)
            .await
            .map_err(js)?;
        Ok(read_out(json!(read.value), &read.meta))
    }

    pub async fn booking_ticket(&self, uri: &str) -> Result<String, JsValue> {
        let read = self
            .client
            .booking_ticket(&parse_uri(uri)?)
            .await
            .map_err(js)?;
        Ok(read_out(read.value.to_json(), &read.meta))
    }

    pub async fn booking_ops(&self, uri: &str) -> Result<String, JsValue> {
        let read = self
            .client
            .booking_ops(&parse_uri(uri)?)
            .await
            .map_err(js)?;
        Ok(read_out(json!(read.value), &read.meta))
    }

    pub async fn bond_info(&self, uri: &str) -> Result<String, JsValue> {
        let read = self.client.bond_info(&parse_uri(uri)?).await.map_err(js)?;
        Ok(read_out(read.value.to_json(), &read.meta))
    }

    pub async fn bond_balance(&self, uri: &str) -> Result<String, JsValue> {
        let read = self
            .client
            .bond_balance(&parse_uri(uri)?)
            .await
            .map_err(js)?;
        Ok(read_out(json!(read.value), &read.meta))
    }

    pub async fn native_rev_balance(&self, address: &str) -> Result<String, JsValue> {
        let read = self.client.native_rev_balance(address).await.map_err(js)?;
        Ok(read_out(json!(read.value), &read.meta))
    }

    pub async fn share_admin_is_paused(&self) -> Result<String, JsValue> {
        let read = self.client.share_admin_is_paused().await.map_err(js)?;
        Ok(read_out(json!(read.value), &read.meta))
    }

    pub async fn share_admin_nonce(&self) -> Result<String, JsValue> {
        let read = self.client.share_admin_nonce().await.map_err(js)?;
        let next = read.value.next();
        Ok(read_out(
            json!({"current": read.value, "next": next}),
            &read.meta,
        ))
    }

    pub async fn share_root_nonce(&self) -> Result<String, JsValue> {
        let read = self.client.share_root_nonce().await.map_err(js)?;
        let next = read.value.next();
        Ok(read_out(
            json!({"current": read.value, "next": next}),
            &read.meta,
        ))
    }

    pub async fn platform_owner(&self) -> Result<String, JsValue> {
        let read = self.client.payments_platform_owner().await.map_err(js)?;
        Ok(read_out(json!(read.value), &read.meta))
    }

    pub async fn booking_action(
        &self,
        uri: &str,
        action: &str,
        op_id: &str,
        args_json: &str,
    ) -> Result<String, JsValue> {
        let args = parse_json_array(args_json)?;
        let action = booking_action_from(action, &args)?;
        let write = self
            .client
            .booking_action(&parse_uri(uri)?, &action, &parse_op(op_id)?)
            .await
            .map_err(js)?;
        Ok(write_out(json!({}), &write.receipt))
    }

    pub async fn guest_ledger(&self) -> Result<String, JsValue> {
        let read = self.client.guest_ledger().await.map_err(js)?;
        Ok(read_out(
            json!({"uri": read.value.uri, "guestId": read.value.guest_id}),
            &read.meta,
        ))
    }

    pub async fn guest_ops(&self, ledger_uri: &str) -> Result<String, JsValue> {
        let read = self
            .client
            .guest_ops(&parse_uri(ledger_uri)?)
            .await
            .map_err(js)?;
        let data = read
            .value
            .iter()
            .map(|(id, op)| (id.clone(), op.to_json()))
            .collect::<serde_json::Map<_, _>>();
        Ok(read_out(json!(data), &read.meta))
    }

    pub async fn account_balance(&self) -> Result<String, JsValue> {
        let read = self.client.account_balance().await.map_err(js)?;
        Ok(read_out(read.value.to_json(), &read.meta))
    }

    pub async fn payments_health(&self) -> Result<String, JsValue> {
        let read = self.client.payments_health().await.map_err(js)?;
        Ok(read_out(read.value, &read.meta))
    }

    pub async fn admin_keys(&self) -> Result<String, JsValue> {
        let read = self.client.share_admin_keys().await.map_err(js)?;
        Ok(read_out(
            json!({"admins": read.value.0, "threshold": read.value.1}),
            &read.meta,
        ))
    }

    pub async fn admin_payload(
        &self,
        domain: &str,
        nonce: i64,
        action: &str,
        args_json: &str,
    ) -> Result<String, JsValue> {
        let args = render_arguments(args_json)?;
        let nonce = Nonce::new(nonce).map_err(js)?;
        let read = self
            .client
            .admin_payload(domain, nonce, action, &args)
            .await
            .map_err(js)?;
        Ok(read_out(
            json!({"payload": read.value, "digest": Client::admin_digest(&read.value)}),
            &read.meta,
        ))
    }

    pub async fn admin_submit_share(
        &self,
        nonce: i64,
        action: &str,
        args_json: &str,
        signatures_json: &str,
    ) -> Result<String, JsValue> {
        let args = render_arguments(args_json)?;
        let signatures = parse_strings(signatures_json)?;
        let nonce = Nonce::new(nonce).map_err(js)?;
        let write = self
            .client
            .admin_submit_share(nonce, action, &args, &signatures)
            .await
            .map_err(js)?;
        Ok(write_out(json!({}), &write.receipt))
    }

    pub async fn admin_submit_listing(
        &self,
        uri: &str,
        nonce: i64,
        action: &str,
        args_json: &str,
        signatures_json: &str,
    ) -> Result<String, JsValue> {
        let args = render_arguments(args_json)?;
        let signatures = parse_strings(signatures_json)?;
        let nonce = Nonce::new(nonce).map_err(js)?;
        let write = self
            .client
            .admin_submit_listing(&parse_uri(uri)?, nonce, action, &args, &signatures)
            .await
            .map_err(js)?;
        Ok(write_out(json!({}), &write.receipt))
    }

    pub async fn admin_submit_provider(
        &self,
        uri: &str,
        nonce: i64,
        signatures_json: &str,
    ) -> Result<String, JsValue> {
        let signatures = parse_strings(signatures_json)?;
        let nonce = Nonce::new(nonce).map_err(js)?;
        let write = self
            .client
            .admin_submit_provider(&parse_uri(uri)?, nonce, &signatures)
            .await
            .map_err(js)?;
        Ok(write_out(json!({}), &write.receipt))
    }

    pub async fn arbitrate(
        &self,
        booking_uri: &str,
        nonce: i64,
        split_bps: i64,
        signatures_json: &str,
        op_id: &str,
    ) -> Result<String, JsValue> {
        let signatures = parse_strings(signatures_json)?;
        let nonce = Nonce::new(nonce).map_err(js)?;
        let split_bps = u16::try_from(split_bps)
            .map_err(|_| js("split bps is outside the basis-point range"))
            .and_then(|value| BasisPoints::new(value).map_err(js))?;
        let write = self
            .client
            .arbitrate(
                &parse_uri(booking_uri)?,
                nonce,
                split_bps,
                &signatures,
                &parse_op(op_id)?,
            )
            .await
            .map_err(js)?;
        Ok(write_out(json!({}), &write.receipt))
    }

    pub async fn faucet(&self, address: &str) -> Result<String, JsValue> {
        let value = self.client.faucet(address).await.map_err(js)?;
        Ok(value.to_string())
    }

    pub async fn fund_account_rev(&self, amount: i64) -> Result<String, JsValue> {
        let write = self.client.fund_account_rev(amount).await.map_err(js)?;
        Ok(write_out(
            json!({"revBaseUnits": write.value}),
            &write.receipt,
        ))
    }

    pub async fn withdraw_rev_to(&self, amount: i64, address: &str) -> Result<String, JsValue> {
        let write = self
            .client
            .withdraw_rev_to(amount, address)
            .await
            .map_err(js)?;
        Ok(write_out(
            json!({"remainingRevBaseUnits": write.value}),
            &write.receipt,
        ))
    }

    pub fn preview_book(
        &self,
        ledger_uri: &str,
        listing_uri: &str,
        unit_slots_json: &str,
        op_id: &str,
        expected_terms_hash: &str,
    ) -> Result<String, JsValue> {
        let requests = parse_unit_slots(unit_slots_json)?;
        let preview = self
            .client
            .preview_book(
                &parse_uri(ledger_uri)?,
                &parse_uri(listing_uri)?,
                &requests,
                &parse_op(op_id)?,
                expected_terms_hash,
            )
            .map_err(js)?;
        Ok(json!({"term": preview.term, "bytes": preview.bytes}).to_string())
    }

    pub fn preview_ensure_unit(
        &self,
        listing_uri: &str,
        unit_id: &str,
        attrs_json: &str,
        slots_json: &str,
        price_per_slot: i64,
    ) -> Result<String, JsValue> {
        let attrs = parse_json(attrs_json, "attrs")?;
        let slots: Vec<SlotSpec> = serde_json::from_str(slots_json).map_err(js)?;
        // The preview is synchronous in the UI; the term builder is the same one the write
        // uses, minus signing.
        let term = format!(
            "new rl(`rho:registry:lookup`), capabilityCh, resultCh, deployerId(`rho:rchain:deployerId`), deployId(`rho:rchain:deployId`) in {{ rl!(`{}`, *capabilityCh) | for (capability <- capabilityCh) {{ capability!(\"ensureUnit\", *deployerId, {}, {}, {}, {}, *resultCh) | for (@replyValue <- resultCh) {{ deployId!(replyValue) }} }} }}",
            parse_uri(listing_uri)?,
            quote_string(unit_id).map_err(js)?,
            render_json(&attrs).map_err(js)?,
            rshare_core::slots::render_slot_specs(&slots).map_err(js)?,
            price_per_slot
        );
        let bytes = term.len();
        Ok(json!({"term": term, "bytes": bytes}).to_string())
    }
}

impl Rshare {
    fn rebuild(&mut self, key: KeyPair) -> Result<(), JsValue> {
        let mut client = Client::new(&self.node, self.manifest.clone(), key).map_err(js)?;
        client.set_network(&self.shard, self.phlo_limit, self.phlo_price);
        self.client = client;
        Ok(())
    }
}

fn js(error: impl Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

fn parse_uri(value: &str) -> Result<RegistryUri, JsValue> {
    RegistryUri::new(value).map_err(js)
}

fn parse_kind(value: &str) -> Result<ResourceKind, JsValue> {
    ResourceKind::parse(value).ok_or_else(|| js(format!("unknown resource kind: {value}")))
}

fn parse_op(value: &str) -> Result<OpId, JsValue> {
    OpId::new(value).map_err(js)
}

fn parse_terms(kind: &str, terms_json: &str) -> Result<Terms, JsValue> {
    Terms::from_json(parse_kind(kind)?, terms_json).map_err(js)
}

fn parse_json(value: &str, label: &str) -> Result<Value, JsValue> {
    serde_json::from_str(value).map_err(|error| js(format!("{label} JSON: {error}")))
}

fn parse_json_array(value: &str) -> Result<Vec<Value>, JsValue> {
    serde_json::from_str(value).map_err(|error| js(format!("args JSON: {error}")))
}

fn parse_strings(value: &str) -> Result<Vec<String>, JsValue> {
    serde_json::from_str(value).map_err(|error| js(format!("string list JSON: {error}")))
}

fn parse_unit_slots(value: &str) -> Result<Vec<UnitSlotRequest>, JsValue> {
    serde_json::from_str(value).map_err(|error| js(format!("unit-slots JSON: {error}")))
}

fn render_arguments(value: &str) -> Result<Vec<String>, JsValue> {
    let values = parse_json_array(value)?;
    let mut rendered = Vec::with_capacity(values.len());
    for value in values {
        rendered.push(render_json(&value).map_err(js)?);
    }
    Ok(rendered)
}

fn read_out(data: Value, meta: &ReadMeta) -> String {
    json!({
        "ok": true,
        "data": data,
        "read": meta_json(meta),
    })
    .to_string()
}

fn write_out(data: Value, receipt: &DeployReceipt) -> String {
    json!({
        "ok": true,
        "data": data,
        "deploy": receipt_json(receipt),
    })
    .to_string()
}

fn meta_json(meta: &ReadMeta) -> Value {
    json!({
        "atHeight": meta.at_height,
        "blockHash": meta.block_hash,
        "source": match meta.source {
            ReadSource::Explore => "explore",
            ReadSource::SignedDeploy => "signedDeploy",
        },
        "deployId": meta.deploy_id,
    })
}

fn receipt_json(receipt: &DeployReceipt) -> Value {
    json!({
        "deployId": receipt.deploy_id,
        "finalized": receipt.finalized,
        "blockHash": receipt.block_hash,
        "blockNumber": receipt.block_number,
        "termBytes": receipt.term_bytes,
        "status": receipt.status_text,
    })
}

fn booking_action_from(action: &str, args: &[Value]) -> Result<BookingAction, JsValue> {
    let arg = |index: usize, label: &str| -> Result<String, JsValue> {
        args.get(index)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| js(format!("{action} needs {label} at args[{index}]")))
    };
    Ok(match action {
        "accept" => BookingAction::Accept,
        "reject" => BookingAction::Reject {
            reason: arg(0, "a reason")?,
        },
        "cancel" => BookingAction::Cancel,
        "cancelByProvider" => BookingAction::CancelByProvider,
        "checkIn" => BookingAction::CheckIn,
        "checkOut" => BookingAction::CheckOut,
        "guestCheckOut" => BookingAction::GuestCheckOut,
        "settle" => BookingAction::Settle,
        "claimDeposit" => BookingAction::ClaimDeposit {
            amount: args
                .first()
                .and_then(Value::as_i64)
                .ok_or_else(|| js("claimDeposit needs an amount at args[0]"))?,
            evidence_hash: arg(1, "an evidence hash")?,
        },
        "resolveDeposit" => BookingAction::ResolveDeposit,
        "dispute" => BookingAction::Dispute {
            reason_hash: arg(0, "a reason hash")?,
        },
        "resolveDispute" => BookingAction::ResolveDispute,
        "disputeNoShow" => BookingAction::DisputeNoShow {
            reason_hash: arg(0, "a reason hash")?,
        },
        "expire" => BookingAction::Expire,
        "claimNoShow" => BookingAction::ClaimNoShow,
        "resolveNoShow" => BookingAction::ResolveNoShow,
        "redeem" => BookingAction::Redeem,
        "closeEvent" => BookingAction::CloseEvent,
        "transferTicket" => BookingAction::TransferTicket {
            new_guest_id: arg(0, "a new guest id")?,
        },
        other => return Err(js(format!("unknown booking action: {other}"))),
    })
}
