//! Typed r-share operations over the r-cash signed-deploy client.
//!
//! This crate owns the chain connection and the typed protocol surface. It deliberately
//! reuses r-cash's `rcash-sdk`/`rcash-chain` for signing, deploy encoding, node HTTP,
//! finality polling and `RhoValue` decoding. It does not use r-cash's typed FX, oracle,
//! pool or manifest operations.

pub mod admin;
pub mod error;
pub mod funding;
pub mod models;
pub mod reads;
pub mod wire;
pub mod writes;

use std::collections::BTreeMap;
use std::sync::Mutex;

use rcash_chain::node::pool::{DiscoveryMode, PoolOptions};
use rcash_chain::node::NodeClient;
use rcash_chain::KeyPair;
use rcash_sdk::Client as RcashClient;

use rshare_core::{ComponentKey, Manifest, RegistryUri};

pub use error::{DecodeError, DecodeKind, RefusalCode, SdkError};
pub use models::{
    AccountView, BondInfo, BookingUnit, BookingView, ClaimView, GuestOp, ListingView,
    OpaqueMetadata, ProviderListingEntry, ProviderView, QuoteSummary, TicketView, UnitView,
};
pub use reads::GuestLedgerRef;
pub use wire::{DeployReceipt, Read, ReadMeta, ReadSource};
pub use writes::{BookOutcome, BookingAction, Preview, Write};

/// Generate a fresh operation id. The caller supplies the timestamp so the browser can
/// use `Date.now()` and the CLI can use `SystemTime` without the SDK touching a clock.
pub fn new_op_id(prefix: &str, now_ms: i64) -> Result<rshare_core::OpId, SdkError> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|error| SdkError::Entropy(error.to_string()))?;
    let random: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    Ok(rshare_core::OpId::new(format!(
        "{prefix}-{now_ms}-{random}"
    ))?)
}

/// A component resolved through the Root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedComponent {
    pub uri: RegistryUri,
    pub version: i64,
}

/// The r-share client. A reader has no key; a signer owns one and can submit deploys.
pub struct Client {
    inner: RcashClient,
    manifest: Manifest,
    shard: String,
    phlo_limit: i64,
    phlo_price: i64,
    cache: Mutex<BTreeMap<ComponentKey, ResolvedComponent>>,
}

impl Client {
    /// The client's default phlo limit. A book deploy resolves several components, so
    /// r-cash's 1,000,000 default is too tight.
    pub const DEFAULT_PHLO_LIMIT: i64 = 4_000_000;
    pub const DEFAULT_PHLO_PRICE: i64 = 1;
    pub const DEFAULT_SHARD: &'static str = "/root";

    /// Connect for public reads.
    pub fn reader(node: &str, manifest: Manifest) -> Result<Self, SdkError> {
        Self::build(
            node,
            manifest,
            None,
            Self::DEFAULT_SHARD,
            Self::DEFAULT_PHLO_LIMIT,
            Self::DEFAULT_PHLO_PRICE,
        )
    }

    /// Connect with a signing key.
    pub fn new(node: &str, manifest: Manifest, key: KeyPair) -> Result<Self, SdkError> {
        Self::build(
            node,
            manifest,
            Some(key),
            Self::DEFAULT_SHARD,
            Self::DEFAULT_PHLO_LIMIT,
            Self::DEFAULT_PHLO_PRICE,
        )
    }

    /// Build around a caller-supplied node client (fixture transport).
    pub fn with_node(
        node: NodeClient,
        manifest: Manifest,
        key: Option<KeyPair>,
        shard: &str,
        phlo_limit: i64,
        phlo_price: i64,
    ) -> Result<Self, SdkError> {
        let inner = RcashClient::with_node(node, key)
            .with_shard(shard)
            .with_phlo(phlo_limit, phlo_price);
        Ok(Self {
            inner,
            manifest,
            shard: shard.to_string(),
            phlo_limit,
            phlo_price,
            cache: Mutex::new(BTreeMap::new()),
        })
    }

    fn build(
        node: &str,
        manifest: Manifest,
        key: Option<KeyPair>,
        shard: &str,
        phlo_limit: i64,
        phlo_price: i64,
    ) -> Result<Self, SdkError> {
        let node_client = node_client(node, shard)?;
        Self::with_node(node_client, manifest, key, shard, phlo_limit, phlo_price)
    }

    /// Change the shard and phlo policy used by subsequent writes.
    pub fn with_network(mut self, shard: &str, phlo_limit: i64, phlo_price: i64) -> Self {
        self.set_network(shard, phlo_limit, phlo_price);
        self
    }

    /// Change the network in place (the WASM facade keeps one client handle).
    pub fn set_network(&mut self, shard: &str, phlo_limit: i64, phlo_price: i64) {
        self.inner
            .set_network(shard.to_string(), phlo_limit, phlo_price);
        self.shard = shard.to_string();
        self.phlo_limit = phlo_limit;
        self.phlo_price = phlo_price;
    }

    /// The signing key, for the WASM facade's keystore export. Never serialized to JS.
    pub fn key(&self) -> Option<&KeyPair> {
        self.inner.key()
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn shard(&self) -> &str {
        &self.shard
    }

    pub fn phlo(&self) -> (i64, i64) {
        (self.phlo_limit, self.phlo_price)
    }

    /// The node client, for raw HTTP reads (status, block, explore) when needed.
    pub fn node(&self) -> &NodeClient {
        self.inner.node()
    }

    /// Whether the client owns a signing key.
    pub fn has_key(&self) -> bool {
        self.inner.key().is_some()
    }

    /// The unlocked wallet's REV address.
    pub fn address(&self) -> Result<String, SdkError> {
        Ok(self.inner.address()?)
    }

    /// The current finalized tip height.
    pub async fn height(&self) -> Result<i64, SdkError> {
        Ok(self.inner.height().await?)
    }

    /// The node's public status document.
    pub async fn status(&self) -> Result<serde_json::Value, SdkError> {
        Ok(self.inner.node().status().await?)
    }

    /// The cache lock. Poisoning cannot make the map invalid, so recover the guard
    /// instead of panicking.
    fn cache_lock(&self) -> std::sync::MutexGuard<'_, BTreeMap<ComponentKey, ResolvedComponent>> {
        self.cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Resolve one component through the r-share Root. Results are cached for the life of
    /// the client; call `refresh_components` after a rotation.
    pub async fn resolve(&self, key: ComponentKey) -> Result<ResolvedComponent, SdkError> {
        if let Some(hit) = self.cache_lock().get(&key).cloned() {
            return Ok(hit);
        }
        let term = wire::resolve_term(self.manifest.root_uri(), key)?;
        let (value, _meta) = wire::explore_value(&self.inner, &term).await?;
        let values = wire::decode_reply(&value)?;
        let resolved = parse_resolved(&values)?;
        self.cache_lock().insert(key, resolved.clone());
        Ok(resolved)
    }

    /// Resolve several components in one Root call.
    pub async fn resolve_many(
        &self,
        keys: &[ComponentKey],
    ) -> Result<BTreeMap<ComponentKey, ResolvedComponent>, SdkError> {
        if keys.is_empty() {
            return Ok(BTreeMap::new());
        }
        let term = wire::resolve_many_term(self.manifest.root_uri(), keys)?;
        let (value, _meta) = wire::explore_value(&self.inner, &term).await?;
        let values = wire::decode_reply(&value)?;
        let map = values
            .first()
            .and_then(rcash_chain::rho_expr::RhoValue::as_map)
            .ok_or_else(|| SdkError::decode("resolveMany", "reply", DecodeKind::WrongType))?;
        let mut resolved = BTreeMap::new();
        for key in keys {
            let field = key.as_str();
            let entry = map
                .get(field)
                .ok_or_else(|| SdkError::decode("resolveMany", field, DecodeKind::Missing))?;
            let entry = entry
                .as_map()
                .ok_or_else(|| SdkError::decode("resolveMany", field, DecodeKind::WrongType))?;
            let uri = entry
                .get("uri")
                .and_then(rcash_chain::rho_expr::RhoValue::as_str)
                .ok_or_else(|| SdkError::decode("resolveMany", "uri", DecodeKind::WrongType))?;
            let version = entry
                .get("version")
                .and_then(rcash_chain::rho_expr::RhoValue::as_int)
                .ok_or_else(|| SdkError::decode("resolveMany", "version", DecodeKind::WrongType))?;
            let component = ResolvedComponent {
                uri: RegistryUri::new(uri)?,
                version,
            };
            self.cache_lock().insert(*key, component.clone());
            resolved.insert(*key, component);
        }
        Ok(resolved)
    }

    /// Drop the component cache.
    pub fn refresh_components(&self) {
        self.cache_lock().clear();
    }
}

fn node_client(node: &str, shard: &str) -> Result<NodeClient, SdkError> {
    let options = PoolOptions {
        discovery: DiscoveryMode::Disabled,
        shard_id: shard.to_string(),
        ..PoolOptions::default()
    };
    Ok(NodeClient::new_with_options(node, options)?)
}

fn parse_resolved(
    values: &[rcash_chain::rho_expr::RhoValue],
) -> Result<ResolvedComponent, SdkError> {
    let uri = values
        .first()
        .and_then(rcash_chain::rho_expr::RhoValue::as_str)
        .ok_or_else(|| SdkError::decode("resolve", "uri", DecodeKind::WrongType))?;
    let version = values
        .get(1)
        .and_then(rcash_chain::rho_expr::RhoValue::as_int)
        .ok_or_else(|| SdkError::decode("resolve", "version", DecodeKind::WrongType))?;
    Ok(ResolvedComponent {
        uri: RegistryUri::new(uri)?,
        version,
    })
}
