//! Admin and arbiter actions.
//!
//! The payload is always produced by the deployed `Codec("adminPayload")` arm, never
//! reconstructed from scratch in the client. `setAdmins` has one deliberate quirk: the
//! signed payload uses the comma-joined public-key text and threshold, while the call
//! carries the byte arrays and threshold.

use rshare_core::render::quote_string;
use rshare_core::{ComponentKey, OpId, RegistryUri};

use crate::error::{Result, SdkError};
use crate::wire::{self, Read};
use crate::writes::{admins_rho, admins_text, signatures_rho, Write};
use crate::Client;

impl Client {
    /// `Codec("adminPayload")`. The returned string is what every signer hashes.
    pub async fn admin_payload(
        &self,
        domain: &str,
        nonce: i64,
        action: &str,
        args: &[String],
    ) -> Result<Read<String>> {
        let component = self.resolve(ComponentKey::Codec).await?;
        let args = vec![
            quote_string(domain)?,
            nonce.to_string(),
            quote_string(action)?,
            render_list(args),
        ];
        let term = wire::call_term(&component.uri, "adminPayload", &args)?;
        let (value, meta) = wire::explore_value(&self.inner, &term).await?;
        let payload = value
            .as_str()
            .ok_or_else(|| SdkError::Decode("admin payload is not a string".into()))?
            .to_string();
        Ok(Read {
            value: payload,
            meta,
        })
    }

    /// The BLAKE2b-256 digest the contract verifies.
    pub fn admin_digest(payload: &str) -> String {
        hex(&rchain_crypto::hash::blake2b256::hash(payload.as_bytes()))
    }

    /// Sign a payload with the unlocked key. Returns lowercase hex DER.
    pub fn admin_sign(&self, payload: &str) -> Result<String> {
        let key = self.inner.key().ok_or(SdkError::NoKey)?;
        let digest = rchain_crypto::hash::blake2b256::hash(payload.as_bytes());
        let signature = key.sign_prehash(&digest)?;
        Ok(hex(&signature))
    }

    /// `ShareAdmin("admin")`.
    pub async fn admin_submit_share(
        &self,
        nonce: i64,
        action: &str,
        args: &[String],
        signatures: &[String],
    ) -> Result<Write<()>> {
        let component = self.resolve(ComponentKey::ShareAdmin).await?;
        let args = vec![
            nonce.to_string(),
            quote_string(action)?,
            render_list(args),
            signatures_rho(signatures)?,
        ];
        submit(&self.inner, &component.uri, "admin", &args).await
    }

    /// `ListingContract("adminPause")`.
    pub async fn admin_submit_listing(
        &self,
        listing_uri: &RegistryUri,
        nonce: i64,
        action: &str,
        args: &[String],
        signatures: &[String],
    ) -> Result<Write<()>> {
        let args = vec![
            nonce.to_string(),
            quote_string(action)?,
            render_list(args),
            signatures_rho(signatures)?,
        ];
        submit(&self.inner, listing_uri, "adminPause", &args).await
    }

    /// `ProviderContract("adminPause")`.
    pub async fn admin_submit_provider(
        &self,
        provider_uri: &RegistryUri,
        nonce: i64,
        signatures: &[String],
    ) -> Result<Write<()>> {
        let args = vec![nonce.to_string(), signatures_rho(signatures)?];
        submit(&self.inner, provider_uri, "adminPause", &args).await
    }

    /// `ListingContract("setAdmins")`.
    pub async fn listing_set_admins(
        &self,
        listing_uri: &RegistryUri,
        nonce: i64,
        new_admins: &[String],
        threshold: i64,
        signatures: &[String],
    ) -> Result<Write<()>> {
        let args = vec![
            nonce.to_string(),
            admins_rho(new_admins)?,
            threshold.to_string(),
            signatures_rho(signatures)?,
        ];
        submit(&self.inner, listing_uri, "setAdmins", &args).await
    }

    /// `ProviderContract("setAdmins")`.
    pub async fn provider_set_admins(
        &self,
        provider_uri: &RegistryUri,
        nonce: i64,
        new_admins: &[String],
        threshold: i64,
        signatures: &[String],
    ) -> Result<Write<()>> {
        let args = vec![
            nonce.to_string(),
            admins_rho(new_admins)?,
            threshold.to_string(),
            signatures_rho(signatures)?,
        ];
        submit(&self.inner, provider_uri, "setAdmins", &args).await
    }

    /// `BookingContract("setAdmins")`.
    pub async fn booking_set_admins(
        &self,
        booking_uri: &RegistryUri,
        nonce: i64,
        new_admins: &[String],
        threshold: i64,
        signatures: &[String],
    ) -> Result<Write<()>> {
        let args = vec![
            nonce.to_string(),
            admins_rho(new_admins)?,
            threshold.to_string(),
            signatures_rho(signatures)?,
        ];
        submit(&self.inner, booking_uri, "setAdmins", &args).await
    }

    /// `BookingContract("arbitrate")` with a bounded deposit split in basis points.
    pub async fn arbitrate(
        &self,
        booking_uri: &RegistryUri,
        nonce: i64,
        split_bps: i64,
        signatures: &[String],
        op: &OpId,
    ) -> Result<Write<()>> {
        let args = vec![
            nonce.to_string(),
            split_bps.to_string(),
            signatures_rho(signatures)?,
            quote_string(op.as_str())?,
        ];
        submit(&self.inner, booking_uri, "arbitrate", &args).await
    }

    /// The payload arguments `setAdmins` signs: the comma-joined key text and threshold.
    pub fn set_admins_payload_args(new_admins: &[String], threshold: i64) -> Result<Vec<String>> {
        Ok(vec![
            quote_string(&admins_text(new_admins))?,
            threshold.to_string(),
        ])
    }
}

fn render_list(args: &[String]) -> String {
    format!("[{}]", args.join(", "))
}

async fn submit(
    inner: &rcash_sdk::Client,
    uri: &RegistryUri,
    arm: &str,
    args: &[String],
) -> Result<Write<()>> {
    let term = wire::call_term_signed(uri, arm, args)?;
    let receipt = wire::deploy_receipt(inner, &term).await?;
    let reply = receipt
        .reply
        .clone()
        .ok_or_else(|| SdkError::Decode(format!("{arm} returned no value")))?;
    wire::decode_reply(&reply)?;
    Ok(Write { value: (), receipt })
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
