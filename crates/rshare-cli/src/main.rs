//! The r-share reference command-line client.
//!
//! Reads, writes, the admin multisig surface and a devnet fixture/journey driver all use
//! the same `rshare-sdk` surface the browser will call.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use clap::{Parser, Subcommand};
use rcash_chain::keystore::DevnetKeys;
use rcash_chain::KeyPair;
use rshare_core::render::render_json;
use rshare_core::terms::FIELDS;
use rshare_core::{
    ComponentKey, Manifest, OpId, RegistryUri, ResourceKind, RshareError, SlotSpec, Terms,
    UnitSlotRequest,
};
use rshare_sdk::{new_op_id, BookingAction, Client, DeployReceipt, ReadMeta, ReadSource, SdkError};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Parser, Debug)]
#[command(name = "rshare-cli", version, about = "Reference client for r-share")]
struct Cli {
    #[arg(
        long,
        env = "RSHARE_NODE",
        default_value = "http://localhost:40403",
        global = true
    )]
    node: String,
    #[arg(
        long,
        env = "RSHARE_MANIFEST",
        default_value = "deployments/devnet.json",
        global = true
    )]
    manifest: PathBuf,
    #[arg(long, env = "RSHARE_SHARD", default_value = "/root", global = true)]
    shard: String,
    #[arg(
        long,
        env = "RSHARE_PHLO_LIMIT",
        default_value_t = 4_000_000,
        global = true
    )]
    phlo_limit: i64,
    #[arg(long, env = "RSHARE_PHLO_PRICE", default_value_t = 1, global = true)]
    phlo_price: i64,
    #[arg(
        long,
        env = "RSHARE_KEYS",
        default_value = ".devnet/keys.json",
        global = true
    )]
    keys: PathBuf,
    /// Key name in the devnet keys file. Defaults to the first key.
    #[arg(long, global = true)]
    key: Option<String>,
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Debug)]
struct Config {
    node: String,
    manifest: PathBuf,
    shard: String,
    phlo_limit: i64,
    phlo_price: i64,
    keys: PathBuf,
    key: Option<String>,
    json: bool,
}

impl Config {
    fn from_cli(cli: &Cli) -> Self {
        Self {
            node: cli.node.clone(),
            manifest: cli.manifest.clone(),
            shard: cli.shard.clone(),
            phlo_limit: cli.phlo_limit,
            phlo_price: cli.phlo_price,
            keys: cli.keys.clone(),
            key: cli.key.clone(),
            json: cli.json,
        }
    }
}

#[derive(Subcommand, Debug)]
enum Command {
    Doctor,
    Manifest {
        #[command(subcommand)]
        command: ManifestCommand,
    },
    Terms {
        #[command(subcommand)]
        command: TermsCommand,
    },
    Provider {
        #[command(subcommand)]
        command: ProviderCommand,
    },
    Listing {
        #[command(subcommand)]
        command: ListingCommand,
    },
    Unit {
        #[command(subcommand)]
        command: UnitCommand,
    },
    Book {
        #[arg(long)]
        listing: String,
        /// JSON list of {unitId, slots}; a path is read when it exists.
        #[arg(long)]
        unit_slots: String,
        #[arg(long)]
        op: Option<String>,
        #[arg(long)]
        expected_terms_hash: Option<String>,
    },
    Booking {
        #[command(subcommand)]
        command: BookingCommand,
    },
    Bond {
        #[command(subcommand)]
        command: BondCommand,
    },
    Admin {
        #[command(subcommand)]
        command: AdminCommand,
    },
    Payments {
        #[command(subcommand)]
        command: PaymentsCommand,
    },
    Wallet {
        #[command(subcommand)]
        command: WalletCommand,
    },
    Guest {
        #[command(subcommand)]
        command: GuestCommand,
    },
    Devnet {
        #[command(subcommand)]
        command: DevnetCommand,
    },
    Keystore {
        #[command(subcommand)]
        command: KeystoreCommand,
    },
}

#[derive(Subcommand, Debug)]
enum ManifestCommand {
    Verify,
}

#[derive(Subcommand, Debug)]
enum TermsCommand {
    Fields,
    Validate {
        #[arg(long)]
        kind: String,
        #[arg(long)]
        terms: Option<String>,
        #[arg(long)]
        file: Option<PathBuf>,
    },
    Hash {
        #[arg(long)]
        kind: String,
        #[arg(long)]
        terms: Option<String>,
        #[arg(long)]
        file: Option<PathBuf>,
    },
    Show {
        #[arg(long)]
        kind: String,
        #[arg(long)]
        terms: Option<String>,
        #[arg(long)]
        file: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
enum ProviderCommand {
    Info {
        #[arg(long)]
        uri: String,
    },
    Listings {
        #[arg(long)]
        uri: String,
    },
    Of {
        #[arg(long)]
        owner: String,
    },
    Register {
        #[arg(long)]
        profile: String,
    },
    UpdateProfile {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        profile: String,
        #[arg(long)]
        op: String,
    },
    Pause {
        #[arg(long)]
        uri: String,
    },
    Resume {
        #[arg(long)]
        uri: String,
    },
}

#[derive(Subcommand, Debug)]
enum ListingCommand {
    Info {
        #[arg(long)]
        uri: String,
    },
    Units {
        #[arg(long)]
        uri: String,
    },
    Schedule {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        unit: String,
    },
    Probe {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        unit_slots: String,
    },
    Quote {
        #[arg(long)]
        uri: String,
        /// Signed read: costs phlo.
        #[arg(long)]
        unit_slots: String,
    },
    Create {
        #[arg(long)]
        provider: String,
        #[arg(long)]
        id: String,
        #[arg(long)]
        kind: String,
        #[arg(long)]
        terms: Option<String>,
        #[arg(long)]
        file: Option<PathBuf>,
    },
    Activate {
        #[arg(long)]
        uri: String,
    },
    Pause {
        #[arg(long)]
        uri: String,
    },
    Resume {
        #[arg(long)]
        uri: String,
    },
    Delist {
        #[arg(long)]
        uri: String,
    },
}

#[derive(Subcommand, Debug)]
enum UnitCommand {
    Ensure {
        #[arg(long)]
        listing: String,
        #[arg(long)]
        unit: String,
        #[arg(long)]
        attrs: String,
        #[arg(long)]
        slots: String,
        #[arg(long)]
        price: i64,
    },
    Pause {
        #[arg(long)]
        listing: String,
        #[arg(long)]
        unit: String,
    },
    Resume {
        #[arg(long)]
        listing: String,
        #[arg(long)]
        unit: String,
    },
    Reset {
        #[arg(long)]
        listing: String,
        #[arg(long)]
        unit: String,
        #[arg(long)]
        slot: String,
    },
}

#[derive(Subcommand, Debug)]
enum BookingCommand {
    Info {
        #[arg(long)]
        uri: String,
    },
    Status {
        #[arg(long)]
        uri: String,
    },
    Ticket {
        #[arg(long)]
        uri: String,
    },
    Ops {
        #[arg(long)]
        uri: String,
    },
    Accept {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        op: Option<String>,
    },
    Reject {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        op: Option<String>,
        #[arg(long)]
        reason: String,
    },
    Cancel {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        op: Option<String>,
    },
    CancelByProvider {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        op: Option<String>,
    },
    CheckIn {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        op: Option<String>,
    },
    CheckOut {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        op: Option<String>,
    },
    GuestCheckOut {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        op: Option<String>,
    },
    Settle {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        op: Option<String>,
    },
    ClaimDeposit {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        op: Option<String>,
        #[arg(long)]
        amount: i64,
        #[arg(long)]
        evidence_hash: String,
    },
    ResolveDeposit {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        op: Option<String>,
    },
    Dispute {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        op: Option<String>,
        #[arg(long)]
        reason_hash: String,
    },
    ResolveDispute {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        op: Option<String>,
    },
    Expire {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        op: Option<String>,
    },
    ClaimNoShow {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        op: Option<String>,
    },
    Redeem {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        op: Option<String>,
    },
    CloseEvent {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        op: Option<String>,
    },
    TransferTicket {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        op: Option<String>,
        #[arg(long)]
        new_guest: String,
    },
}

#[derive(Subcommand, Debug)]
enum BondCommand {
    Info {
        #[arg(long)]
        uri: String,
    },
    Balance {
        #[arg(long)]
        uri: String,
    },
}

#[derive(Subcommand, Debug)]
enum AdminCommand {
    Keys,
    Paused,
    Payload {
        #[arg(long)]
        domain: String,
        #[arg(long)]
        nonce: i64,
        #[arg(long)]
        action: String,
        /// JSON array of arguments. Strings, integers and booleans are rendered.
        #[arg(long, default_value = "[]")]
        args: String,
    },
    Sign {
        #[arg(long)]
        payload: String,
    },
    PauseAll {
        #[arg(long)]
        nonce: i64,
        #[arg(long = "sig", value_delimiter = ',')]
        sigs: Vec<String>,
    },
    ResumeAll {
        #[arg(long)]
        nonce: i64,
        #[arg(long = "sig", value_delimiter = ',')]
        sigs: Vec<String>,
    },
    ListingPause {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        nonce: i64,
        #[arg(long)]
        action: String,
        #[arg(long = "sig", value_delimiter = ',')]
        sigs: Vec<String>,
    },
    ProviderPause {
        #[arg(long)]
        uri: String,
        #[arg(long)]
        nonce: i64,
        #[arg(long = "sig", value_delimiter = ',')]
        sigs: Vec<String>,
    },
    Arbitrate {
        #[arg(long)]
        booking: String,
        #[arg(long)]
        nonce: i64,
        #[arg(long)]
        split_bps: i64,
        #[arg(long)]
        op: Option<String>,
        #[arg(long = "sig", value_delimiter = ',')]
        sigs: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
enum PaymentsCommand {
    Health,
    PlatformOwner,
}

#[derive(Subcommand, Debug)]
enum WalletCommand {
    Address,
    Balance,
    Account,
    Faucet,
    /// Move native REV into the caller's r-cash account. Amount is base units.
    Fund {
        #[arg(long)]
        amount: i64,
    },
    /// Move account REV to a native REV address. Amount is base units.
    Withdraw {
        #[arg(long)]
        amount: i64,
        #[arg(long)]
        to: String,
    },
}

#[derive(Subcommand, Debug)]
enum GuestCommand {
    Ledger,
    Ops {
        #[arg(long)]
        ledger: String,
    },
}

#[derive(Subcommand, Debug)]
enum KeystoreCommand {
    Encrypt {
        #[arg(long)]
        name: String,
        #[arg(long)]
        password: String,
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
enum DevnetCommand {
    Fixtures {
        #[arg(long, default_value = ".devnet/client-fixtures.json")]
        out: PathBuf,
    },
    Journey {
        #[arg(long, default_value = ".devnet/client-fixtures.json")]
        fixtures: PathBuf,
        #[arg(long, default_value = "deployments/client-results.json")]
        results: PathBuf,
    },
}

#[derive(Debug)]
enum CliError {
    Config(String),
    Core(RshareError),
    Sdk(SdkError),
    Io(std::io::Error),
    Json(serde_json::Error),
    Failed(String),
}

impl From<RshareError> for CliError {
    fn from(error: RshareError) -> Self {
        Self::Core(error)
    }
}

impl From<SdkError> for CliError {
    fn from(error: SdkError) -> Self {
        Self::Sdk(error)
    }
}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(message) => write!(formatter, "{message}"),
            Self::Core(error) => write!(formatter, "{error}"),
            Self::Sdk(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Json(error) => write!(formatter, "{error}"),
            Self::Failed(message) => write!(formatter, "{message}"),
        }
    }
}

impl CliError {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Sdk(SdkError::Refused { .. }) => 1,
            Self::Sdk(SdkError::DeployFailed { .. } | SdkError::Unknown { .. }) => 4,
            Self::Failed(_) => 1,
            Self::Config(_) | Self::Core(_) | Self::Io(_) | Self::Json(_) => 3,
            Self::Sdk(_) => 3,
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(error) = run(cli).await {
        eprintln!("rshare-cli: {error}");
        std::process::exit(error.exit_code());
    }
}

async fn run(cli: Cli) -> Result<(), CliError> {
    let config = Config::from_cli(&cli);
    match cli.command {
        Command::Devnet { command } => devnet(&config, command).await,
        Command::Keystore { command } => keystore(&config, command).await,
        command => {
            let client = connect(&config).await?;
            dispatch(&config, &client, command).await
        }
    }
}

async fn dispatch(config: &Config, client: &Client, command: Command) -> Result<(), CliError> {
    match command {
        Command::Doctor => doctor(config, client).await,
        Command::Manifest {
            command: ManifestCommand::Verify,
        } => manifest_verify(config, client).await,
        Command::Terms { command } => terms(config, client, command).await,
        Command::Provider { command } => provider(config, client, command).await,
        Command::Listing { command } => listing(config, client, command).await,
        Command::Unit { command } => unit(config, client, command).await,
        Command::Book {
            listing,
            unit_slots,
            op,
            expected_terms_hash,
        } => {
            book(
                config,
                client,
                &listing,
                &unit_slots,
                op,
                expected_terms_hash,
            )
            .await
        }
        Command::Booking { command } => booking(config, client, command).await,
        Command::Bond { command } => bond(config, client, command).await,
        Command::Admin { command } => admin(config, client, command).await,
        Command::Payments { command } => payments(config, client, command).await,
        Command::Wallet { command } => wallet(config, client, command).await,
        Command::Guest { command } => guest(config, client, command).await,
        Command::Devnet { .. } | Command::Keystore { .. } => {
            unreachable!("offline commands are dispatched first")
        }
    }
}

async fn keystore(config: &Config, command: KeystoreCommand) -> Result<(), CliError> {
    match command {
        KeystoreCommand::Encrypt {
            name,
            password,
            out,
        } => {
            let keys = DevnetKeys::load(&config.keys)
                .map_err(|error| CliError::Config(format!("{}: {error}", config.keys.display())))?;
            let key = keys
                .key(&name)
                .map_err(|error| CliError::Config(error.to_string()))?;
            let eth = rcash_chain::address::eth_address_from_public_key(key.public_bytes())
                .map_err(|error| CliError::Config(error.to_string()))?;
            let document = rshare_keystore::encrypt(key.secret_bytes(), &eth, &password)
                .map_err(CliError::Config)?;
            std::fs::write(&out, format!("{document}\n"))?;
            emit(
                config,
                "keystore encrypt",
                json!({"out": out, "address": key.rev_address().map_err(|error| CliError::Config(error.to_string()))?}),
                None,
            )
        }
    }
}

async fn connect(config: &Config) -> Result<Client, CliError> {
    client_for(config, None).await
}

async fn client_for(config: &Config, name: Option<&str>) -> Result<Client, CliError> {
    let document = std::fs::read_to_string(&config.manifest).map_err(|error| {
        CliError::Config(format!(
            "cannot read manifest {}: {error}",
            config.manifest.display()
        ))
    })?;
    let manifest = Manifest::from_json(&document)?;
    let key = load_key(config, name)?;
    let client = match key {
        Some(key) => Client::new(&config.node, manifest, key)?,
        None => Client::reader(&config.node, manifest)?,
    };
    Ok(client.with_network(&config.shard, config.phlo_limit, config.phlo_price))
}

fn load_key(config: &Config, name: Option<&str>) -> Result<Option<KeyPair>, CliError> {
    let requested = name.or(config.key.as_deref());
    if !config.keys.exists() {
        if requested.is_some() {
            return Err(CliError::Config(format!(
                "keys file {} does not exist",
                config.keys.display()
            )));
        }
        return Ok(None);
    }
    let keys = DevnetKeys::load(&config.keys)
        .map_err(|error| CliError::Config(format!("{}: {error}", config.keys.display())))?;
    match requested {
        Some(name) => Ok(Some(
            keys.key(name)
                .map_err(|error| CliError::Config(error.to_string()))?,
        )),
        None => match keys.first_key() {
            Ok(key) => Ok(Some(key)),
            Err(_) => Ok(None),
        },
    }
}

async fn doctor(config: &Config, client: &Client) -> Result<(), CliError> {
    let status = client.status().await?;
    let height = client.height().await?;
    let components = client.resolve_many(&ComponentKey::ALL).await?;
    let terms = client.terms_fields().await?;
    let payments = client.payments_health().await?;
    let fields_ok = terms.value == FIELDS;
    let mut wallet = Value::Null;
    if client.has_key() {
        let address = client.address()?;
        let native = client.native_rev_balance(&address).await?;
        let account = match client.account_balance().await {
            Ok(read) => json!({
                "ownerId": read.value.owner_id,
                "revBaseUnits": read.value.rev_base_units,
            }),
            Err(error) => json!({"error": error.to_string()}),
        };
        wallet = json!({
            "address": address,
            "nativeRevBaseUnits": native.value,
            "account": account,
        });
    }
    let data = json!({
        "node": config.node,
        "status": status,
        "height": height,
        "components": components.iter().map(|(key, resolved)| {
            (key.as_str().to_string(), json!({
                "uri": resolved.uri.as_str(),
                "version": resolved.version,
            }))
        }).collect::<serde_json::Map<_, _>>(),
        "termsFields": terms.value,
        "termsFieldsOk": fields_ok,
        "payments": payments.value,
        "wallet": wallet,
    });
    emit(config, "doctor", data, None)?;
    if !fields_ok {
        return Err(CliError::Failed(
            "Terms(\"fields\") does not match the client's 20-field order".into(),
        ));
    }
    Ok(())
}

async fn manifest_verify(config: &Config, client: &Client) -> Result<(), CliError> {
    let components = client.resolve_many(&ComponentKey::ALL).await?;
    let data = json!({
        "root": client.manifest().root_uri().as_str(),
        "components": components.iter().map(|(key, resolved)| {
            (key.as_str().to_string(), json!({
                "uri": resolved.uri.as_str(),
                "version": resolved.version,
            }))
        }).collect::<serde_json::Map<_, _>>(),
    });
    emit(config, "manifest verify", data, None)
}

async fn terms(config: &Config, client: &Client, command: TermsCommand) -> Result<(), CliError> {
    match command {
        TermsCommand::Fields => {
            let read = client.terms_fields().await?;
            emit(config, "terms fields", json!(read.value), Some(&read.meta))
        }
        TermsCommand::Validate { kind, terms, file } => {
            let terms = load_terms(&kind, terms, file)?;
            let read = client.terms_validate(&terms).await?;
            emit(config, "terms validate", json!(true), Some(&read.meta))
        }
        TermsCommand::Hash { kind, terms, file } => {
            let terms = load_terms(&kind, terms, file)?;
            let read = client.terms_hash(&terms).await?;
            emit(config, "terms hash", json!(read.value), Some(&read.meta))
        }
        TermsCommand::Show { kind, terms, file } => {
            let terms = load_terms(&kind, terms, file)?;
            emit(config, "terms show", terms.to_json(), None)
        }
    }
}

async fn provider(
    config: &Config,
    client: &Client,
    command: ProviderCommand,
) -> Result<(), CliError> {
    match command {
        ProviderCommand::Info { uri } => {
            let read = client.provider_info(&parse_uri(&uri)?).await?;
            emit(
                config,
                "provider info",
                read.value.to_json(),
                Some(&read.meta),
            )
        }
        ProviderCommand::Listings { uri } => {
            let read = client.provider_listings(&parse_uri(&uri)?).await?;
            let data = read
                .value
                .iter()
                .map(|(id, entry)| (id.clone(), entry.to_json()))
                .collect::<serde_json::Map<_, _>>();
            emit(config, "provider listings", json!(data), Some(&read.meta))
        }
        ProviderCommand::Of { owner } => {
            let read = client.provider_of(&owner).await?;
            emit(config, "provider of", json!(read.value), Some(&read.meta))
        }
        ProviderCommand::Register { profile } => {
            require_key(client)?;
            let profile = parse_json(&profile, "profile")?;
            let write = client.register_provider(&profile).await?;
            emit(
                config,
                "provider register",
                json!({
                    "providerUri": write.value.0,
                    "ownerId": write.value.1,
                    "deploy": receipt_json(&write.receipt),
                }),
                None,
            )
        }
        ProviderCommand::UpdateProfile { uri, profile, op } => {
            require_key(client)?;
            let profile = parse_json(&profile, "profile")?;
            let op = parse_op(Some(&op))?;
            let write = client
                .update_profile(&parse_uri(&uri)?, &profile, &op)
                .await?;
            emit(
                config,
                "provider update-profile",
                json!({"deploy": receipt_json(&write.receipt)}),
                None,
            )
        }
        ProviderCommand::Pause { uri } => {
            require_key(client)?;
            let write = client.provider_pause(&parse_uri(&uri)?).await?;
            emit(
                config,
                "provider pause",
                json!({"deploy": receipt_json(&write.receipt)}),
                None,
            )
        }
        ProviderCommand::Resume { uri } => {
            require_key(client)?;
            let write = client.provider_resume(&parse_uri(&uri)?).await?;
            emit(
                config,
                "provider resume",
                json!({"deploy": receipt_json(&write.receipt)}),
                None,
            )
        }
    }
}

async fn listing(
    config: &Config,
    client: &Client,
    command: ListingCommand,
) -> Result<(), CliError> {
    match command {
        ListingCommand::Info { uri } => {
            let read = client.listing_info(&parse_uri(&uri)?).await?;
            emit(
                config,
                "listing info",
                read.value.to_json(),
                Some(&read.meta),
            )
        }
        ListingCommand::Units { uri } => {
            let read = client.listing_units(&parse_uri(&uri)?).await?;
            let data = read
                .value
                .iter()
                .map(|(id, unit)| (id.clone(), unit.to_json()))
                .collect::<serde_json::Map<_, _>>();
            emit(config, "listing units", json!(data), Some(&read.meta))
        }
        ListingCommand::Schedule { uri, unit } => {
            let read = client.listing_schedule(&parse_uri(&uri)?, &unit).await?;
            emit(
                config,
                "listing schedule",
                json!(read.value),
                Some(&read.meta),
            )
        }
        ListingCommand::Probe { uri, unit_slots } => {
            let requests = parse_unit_slots(&unit_slots)?;
            let read = client.listing_probe(&parse_uri(&uri)?, &requests).await?;
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
            emit(config, "listing probe", json!(data), Some(&read.meta))
        }
        ListingCommand::Quote { uri, unit_slots } => {
            require_key(client)?;
            let requests = parse_unit_slots(&unit_slots)?;
            let read = client.listing_quote(&parse_uri(&uri)?, &requests).await?;
            emit(
                config,
                "listing quote",
                read.value.to_json(),
                Some(&read.meta),
            )
        }
        ListingCommand::Create {
            provider,
            id,
            kind,
            terms: terms_json,
            file,
        } => {
            require_key(client)?;
            let kind_text = kind;
            let kind = parse_kind(&kind_text)?;
            let terms = load_terms(&kind_text, terms_json, file)?;
            let provider_uri = parse_uri(&provider)?;
            let listing_uri = client
                .create_listing(&provider_uri, &id, kind, &terms)
                .await?;
            emit(
                config,
                "listing create",
                json!({
                    "listingUri": listing_uri.value,
                    "deploy": receipt_json(&listing_uri.receipt),
                }),
                None,
            )
        }
        ListingCommand::Activate { uri } => {
            require_key(client)?;
            let write = client.activate_listing(&parse_uri(&uri)?).await?;
            emit(
                config,
                "listing activate",
                json!({"deploy": receipt_json(&write.receipt)}),
                None,
            )
        }
        ListingCommand::Pause { uri } => {
            require_key(client)?;
            let write = client.pause_listing(&parse_uri(&uri)?).await?;
            emit(
                config,
                "listing pause",
                json!({"deploy": receipt_json(&write.receipt)}),
                None,
            )
        }
        ListingCommand::Resume { uri } => {
            require_key(client)?;
            let write = client.resume_listing(&parse_uri(&uri)?).await?;
            emit(
                config,
                "listing resume",
                json!({"deploy": receipt_json(&write.receipt)}),
                None,
            )
        }
        ListingCommand::Delist { uri } => {
            require_key(client)?;
            let write = client.delist_listing(&parse_uri(&uri)?).await?;
            emit(
                config,
                "listing delist",
                json!({"deploy": receipt_json(&write.receipt)}),
                None,
            )
        }
    }
}

async fn unit(config: &Config, client: &Client, command: UnitCommand) -> Result<(), CliError> {
    match command {
        UnitCommand::Ensure {
            listing,
            unit,
            attrs,
            slots,
            price,
        } => {
            require_key(client)?;
            let attrs = parse_json(&attrs, "attrs")?;
            let slots = parse_slot_specs(&slots)?;
            let write = client
                .ensure_unit(&parse_uri(&listing)?, &unit, &attrs, &slots, price)
                .await?;
            emit(
                config,
                "unit ensure",
                json!({"unitId": write.value, "deploy": receipt_json(&write.receipt)}),
                None,
            )
        }
        UnitCommand::Pause { listing, unit } => {
            require_key(client)?;
            let write = client.pause_unit(&parse_uri(&listing)?, &unit).await?;
            emit(
                config,
                "unit pause",
                json!({"deploy": receipt_json(&write.receipt)}),
                None,
            )
        }
        UnitCommand::Resume { listing, unit } => {
            require_key(client)?;
            let write = client.resume_unit(&parse_uri(&listing)?, &unit).await?;
            emit(
                config,
                "unit resume",
                json!({"deploy": receipt_json(&write.receipt)}),
                None,
            )
        }
        UnitCommand::Reset {
            listing,
            unit,
            slot,
        } => {
            require_key(client)?;
            let write = client
                .reset_unit(&parse_uri(&listing)?, &unit, &slot)
                .await?;
            emit(
                config,
                "unit reset",
                json!({"deploy": receipt_json(&write.receipt)}),
                None,
            )
        }
    }
}

async fn book(
    config: &Config,
    client: &Client,
    listing: &str,
    unit_slots: &str,
    op: Option<String>,
    expected_terms_hash: Option<String>,
) -> Result<(), CliError> {
    require_key(client)?;
    let requests = parse_unit_slots(unit_slots)?;
    let op = parse_op(op.as_deref())?;
    let outcome = client
        .book(
            &parse_uri(listing)?,
            &requests,
            &op,
            expected_terms_hash.as_deref(),
        )
        .await?;
    emit(
        config,
        "book",
        json!({
            "bookingId": outcome.booking_id,
            "bookingUri": outcome.booking_uri,
            "ledgerUri": outcome.ledger_uri,
            "guestId": outcome.guest_id,
            "deploy": receipt_json(&outcome.receipt),
        }),
        None,
    )
}

async fn booking(
    config: &Config,
    client: &Client,
    command: BookingCommand,
) -> Result<(), CliError> {
    match command {
        BookingCommand::Info { uri } => {
            let read = client.booking_info(&parse_uri(&uri)?).await?;
            emit(
                config,
                "booking info",
                read.value.to_json(),
                Some(&read.meta),
            )
        }
        BookingCommand::Status { uri } => {
            let read = client.booking_status(&parse_uri(&uri)?).await?;
            emit(
                config,
                "booking status",
                json!(read.value),
                Some(&read.meta),
            )
        }
        BookingCommand::Ticket { uri } => {
            let read = client.booking_ticket(&parse_uri(&uri)?).await?;
            emit(
                config,
                "booking ticket",
                read.value.to_json(),
                Some(&read.meta),
            )
        }
        BookingCommand::Ops { uri } => {
            let read = client.booking_ops(&parse_uri(&uri)?).await?;
            emit(config, "booking ops", json!(read.value), Some(&read.meta))
        }
        action => booking_action(config, client, action).await,
    }
}

async fn booking_action(
    config: &Config,
    client: &Client,
    command: BookingCommand,
) -> Result<(), CliError> {
    require_key(client)?;
    let (uri, action, op) = match command {
        BookingCommand::Accept { uri, op } => (uri, BookingAction::Accept, op),
        BookingCommand::Reject { uri, op, reason } => (uri, BookingAction::Reject { reason }, op),
        BookingCommand::Cancel { uri, op } => (uri, BookingAction::Cancel, op),
        BookingCommand::CancelByProvider { uri, op } => (uri, BookingAction::CancelByProvider, op),
        BookingCommand::CheckIn { uri, op } => (uri, BookingAction::CheckIn, op),
        BookingCommand::CheckOut { uri, op } => (uri, BookingAction::CheckOut, op),
        BookingCommand::GuestCheckOut { uri, op } => (uri, BookingAction::GuestCheckOut, op),
        BookingCommand::Settle { uri, op } => (uri, BookingAction::Settle, op),
        BookingCommand::ClaimDeposit {
            uri,
            op,
            amount,
            evidence_hash,
        } => (
            uri,
            BookingAction::ClaimDeposit {
                amount,
                evidence_hash,
            },
            op,
        ),
        BookingCommand::ResolveDeposit { uri, op } => (uri, BookingAction::ResolveDeposit, op),
        BookingCommand::Dispute {
            uri,
            op,
            reason_hash,
        } => (uri, BookingAction::Dispute { reason_hash }, op),
        BookingCommand::ResolveDispute { uri, op } => (uri, BookingAction::ResolveDispute, op),
        BookingCommand::Expire { uri, op } => (uri, BookingAction::Expire, op),
        BookingCommand::ClaimNoShow { uri, op } => (uri, BookingAction::ClaimNoShow, op),
        BookingCommand::Redeem { uri, op } => (uri, BookingAction::Redeem, op),
        BookingCommand::CloseEvent { uri, op } => (uri, BookingAction::CloseEvent, op),
        BookingCommand::TransferTicket { uri, op, new_guest } => (
            uri,
            BookingAction::TransferTicket {
                new_guest_id: new_guest,
            },
            op,
        ),
        BookingCommand::Info { .. }
        | BookingCommand::Status { .. }
        | BookingCommand::Ticket { .. }
        | BookingCommand::Ops { .. } => unreachable!("reads are dispatched earlier"),
    };
    let op = parse_op(op.as_deref())?;
    let write = client
        .booking_action(&parse_uri(&uri)?, &action, &op)
        .await?;
    emit(
        config,
        "booking action",
        json!({"deploy": receipt_json(&write.receipt)}),
        None,
    )
}

async fn bond(config: &Config, client: &Client, command: BondCommand) -> Result<(), CliError> {
    match command {
        BondCommand::Info { uri } => {
            let read = client.bond_info(&parse_uri(&uri)?).await?;
            emit(config, "bond info", read.value.to_json(), Some(&read.meta))
        }
        BondCommand::Balance { uri } => {
            let read = client.bond_balance(&parse_uri(&uri)?).await?;
            emit(config, "bond balance", json!(read.value), Some(&read.meta))
        }
    }
}

async fn admin(config: &Config, client: &Client, command: AdminCommand) -> Result<(), CliError> {
    match command {
        AdminCommand::Keys => {
            let read = client.share_admin_keys().await?;
            emit(
                config,
                "admin keys",
                json!({"admins": read.value.0, "threshold": read.value.1}),
                Some(&read.meta),
            )
        }
        AdminCommand::Paused => {
            let read = client.share_admin_is_paused().await?;
            emit(config, "admin paused", json!(read.value), Some(&read.meta))
        }
        AdminCommand::Payload {
            domain,
            nonce,
            action,
            args,
        } => {
            let args = parse_admin_args(&args)?;
            let read = client.admin_payload(&domain, nonce, &action, &args).await?;
            emit(
                config,
                "admin payload",
                json!({
                    "payload": read.value,
                    "digest": Client::admin_digest(&read.value),
                }),
                Some(&read.meta),
            )
        }
        AdminCommand::Sign { payload } => {
            require_key(client)?;
            let signature = client.admin_sign(&payload)?;
            emit(
                config,
                "admin sign",
                json!({
                    "signature": signature,
                    "digest": Client::admin_digest(&payload),
                }),
                None,
            )
        }
        AdminCommand::PauseAll { nonce, sigs } => {
            require_key(client)?;
            let write = client
                .admin_submit_share(nonce, "pauseAll", &[], &sigs)
                .await?;
            emit(
                config,
                "admin pause-all",
                json!({"deploy": receipt_json(&write.receipt)}),
                None,
            )
        }
        AdminCommand::ResumeAll { nonce, sigs } => {
            require_key(client)?;
            let write = client
                .admin_submit_share(nonce, "resumeAll", &[], &sigs)
                .await?;
            emit(
                config,
                "admin resume-all",
                json!({"deploy": receipt_json(&write.receipt)}),
                None,
            )
        }
        AdminCommand::ListingPause {
            uri,
            nonce,
            action,
            sigs,
        } => {
            require_key(client)?;
            let write = client
                .admin_submit_listing(&parse_uri(&uri)?, nonce, &action, &[], &sigs)
                .await?;
            emit(
                config,
                "admin listing-pause",
                json!({"deploy": receipt_json(&write.receipt)}),
                None,
            )
        }
        AdminCommand::ProviderPause { uri, nonce, sigs } => {
            require_key(client)?;
            let write = client
                .admin_submit_provider(&parse_uri(&uri)?, nonce, &sigs)
                .await?;
            emit(
                config,
                "admin provider-pause",
                json!({"deploy": receipt_json(&write.receipt)}),
                None,
            )
        }
        AdminCommand::Arbitrate {
            booking,
            nonce,
            split_bps,
            op,
            sigs,
        } => {
            require_key(client)?;
            let op = parse_op(op.as_deref())?;
            let write = client
                .arbitrate(&parse_uri(&booking)?, nonce, split_bps, &sigs, &op)
                .await?;
            emit(
                config,
                "admin arbitrate",
                json!({"deploy": receipt_json(&write.receipt)}),
                None,
            )
        }
    }
}

async fn payments(
    config: &Config,
    client: &Client,
    command: PaymentsCommand,
) -> Result<(), CliError> {
    match command {
        PaymentsCommand::Health => {
            let read = client.payments_health().await?;
            emit(config, "payments health", read.value, Some(&read.meta))
        }
        PaymentsCommand::PlatformOwner => {
            let read = client.payments_platform_owner().await?;
            emit(
                config,
                "payments platform-owner",
                json!(read.value),
                Some(&read.meta),
            )
        }
    }
}

async fn wallet(config: &Config, client: &Client, command: WalletCommand) -> Result<(), CliError> {
    match command {
        WalletCommand::Address => {
            require_key(client)?;
            emit(config, "wallet address", json!(client.address()?), None)
        }
        WalletCommand::Balance => {
            require_key(client)?;
            let address = client.address()?;
            let read = client.native_rev_balance(&address).await?;
            emit(
                config,
                "wallet balance",
                json!({"address": address, "revBaseUnits": read.value}),
                Some(&read.meta),
            )
        }
        WalletCommand::Account => {
            require_key(client)?;
            let read = client.account_balance().await?;
            emit(
                config,
                "wallet account",
                read.value.to_json(),
                Some(&read.meta),
            )
        }
        WalletCommand::Faucet => {
            require_key(client)?;
            let address = client.address()?;
            let value = client.faucet(&address).await?;
            emit(config, "wallet faucet", value, None)
        }
        WalletCommand::Fund { amount } => {
            require_key(client)?;
            let write = client.fund_account_rev(amount).await?;
            emit(
                config,
                "wallet fund",
                json!({"revBaseUnits": write.value, "deploy": receipt_json(&write.receipt)}),
                None,
            )
        }
        WalletCommand::Withdraw { amount, to } => {
            require_key(client)?;
            let write = client.withdraw_rev_to(amount, &to).await?;
            emit(
                config,
                "wallet withdraw",
                json!({"remainingRevBaseUnits": write.value, "deploy": receipt_json(&write.receipt)}),
                None,
            )
        }
    }
}

async fn guest(config: &Config, client: &Client, command: GuestCommand) -> Result<(), CliError> {
    match command {
        GuestCommand::Ledger => {
            require_key(client)?;
            let read = client.guest_ledger().await?;
            emit(
                config,
                "guest ledger",
                json!({"uri": read.value.uri, "guestId": read.value.guest_id}),
                Some(&read.meta),
            )
        }
        GuestCommand::Ops { ledger } => {
            require_key(client)?;
            let read = client.guest_ops(&parse_uri(&ledger)?).await?;
            let data = read
                .value
                .iter()
                .map(|(id, op)| (id.clone(), op.to_json()))
                .collect::<serde_json::Map<_, _>>();
            emit(config, "guest ops", json!(data), Some(&read.meta))
        }
    }
}

// ---- devnet fixtures and journey ---------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Fixtures {
    provider_key: String,
    guest_key: String,
    provider_uri: String,
    listing_uri: String,
    unit_id: String,
    slot_id: String,
    slot_start: i64,
    slot_end: i64,
    price: i64,
    deposit: i64,
}

async fn devnet(config: &Config, command: DevnetCommand) -> Result<(), CliError> {
    match command {
        DevnetCommand::Fixtures { out } => fixtures(config, &out).await,
        DevnetCommand::Journey { fixtures, results } => journey(config, &fixtures, &results).await,
    }
}

async fn fixtures(config: &Config, out: &Path) -> Result<(), CliError> {
    let provider = client_for(config, Some("business")).await?;
    let guest = client_for(config, Some("sender")).await?;
    ensure_native(&provider).await?;
    ensure_native(&guest).await?;
    let provider_owner = provider.address()?;
    let provider_uri = match provider.provider_of(&provider_owner).await {
        Ok(read) => read.value,
        Err(SdkError::Refused { reason, .. }) if reason == "no provider" => {
            provider
                .register_provider(&json!({"name": "r-share devnet"}))
                .await?
                .value
                .0
        }
        Err(error) => return Err(error.into()),
    };
    let height = provider.height().await?;
    let start = height + 300;
    let end = start + 100;
    let terms = Terms::default_for(ResourceKind::Robot);
    let listing_id = format!("listing:client:{}", now_ms());
    let listing = provider
        .create_listing(
            &parse_uri(&provider_uri)?,
            &listing_id,
            ResourceKind::Robot,
            &terms,
        )
        .await?;
    let unit_id = format!("unit:{listing_id}");
    let slots = [SlotSpec {
        slot_id: "s1".into(),
        start_height: start,
        end_height: end,
    }];
    provider
        .ensure_unit(
            &parse_uri(&listing.value)?,
            &unit_id,
            &json!({"name": "demo robot"}),
            &slots,
            terms.price_per_slot,
        )
        .await?;
    provider
        .activate_listing(&parse_uri(&listing.value)?)
        .await?;
    ensure_account(&guest, 1_000).await?;
    let fixtures = Fixtures {
        provider_key: "business".into(),
        guest_key: "sender".into(),
        provider_uri,
        listing_uri: listing.value,
        unit_id,
        slot_id: "s1".into(),
        slot_start: start,
        slot_end: end,
        price: terms.price_per_slot,
        deposit: terms.deposit,
    };
    write_json(out, &fixtures)?;
    let catalogue = json!({
        "generatedAt": now_ms().to_string(),
        "providers": [{
            "uri": fixtures.provider_uri.clone(),
            "ownerId": provider_owner,
            "label": "r-share devnet",
            "kind": "robot",
            "listings": [{
                "uri": fixtures.listing_uri.clone(),
                "listingId": listing_id,
                "label": "Demo robot",
                "kind": "robot"
            }]
        }]
    });
    write_json(Path::new("deployments/catalogue.json"), &catalogue)?;
    emit(
        config,
        "devnet fixtures",
        json!({"fixtures": fixtures, "path": out, "catalogue": "deployments/catalogue.json"}),
        None,
    )
}

async fn journey(
    config: &Config,
    fixtures_path: &Path,
    results_path: &Path,
) -> Result<(), CliError> {
    if !fixtures_path.exists() {
        fixtures(config, fixtures_path).await?;
    }
    let fixtures: Fixtures = serde_json::from_str(&std::fs::read_to_string(fixtures_path)?)?;
    let provider = client_for(config, Some(&fixtures.provider_key)).await?;
    let guest = client_for(config, Some(&fixtures.guest_key)).await?;
    ensure_native(&provider).await?;
    ensure_native(&guest).await?;
    ensure_account(&guest, 1_000).await?;
    let listing = parse_uri(&fixtures.listing_uri)?;
    let requests = vec![UnitSlotRequest {
        unit_id: fixtures.unit_id.clone(),
        slots: vec![fixtures.slot_id.clone()],
    }];
    let mut checks = Vec::new();

    let quote = guest.listing_quote(&listing, &requests).await?;
    checks.push(json!({
        "name": "quote",
        "ok": quote.value.rent == fixtures.price,
        "rent": quote.value.rent,
    }));

    let book_op = parse_op(None)?;
    let booked = guest.book(&listing, &requests, &book_op, None).await?;
    let info = guest.booking_info(&parse_uri(&booked.booking_uri)?).await?;
    checks.push(json!({
        "name": "book",
        "ok": info.value.status == rshare_core::BookingStatus::Confirmed,
        "bookingUri": booked.booking_uri,
        "status": info.value.status.as_str(),
    }));

    let held = guest
        .book(&listing, &requests, &parse_op(None)?, None)
        .await;
    checks.push(json!({
        "name": "held slot refuses a second booking",
        "ok": matches!(held, Err(SdkError::Refused { ref reason, .. }) if reason == "slot held"),
        "error": format!("{held:?}"),
    }));

    let replay = guest.book(&listing, &requests, &book_op, None).await?;
    checks.push(json!({
        "name": "replay returns the same booking",
        "ok": replay.booking_uri == booked.booking_uri,
        "bookingUri": replay.booking_uri,
    }));

    let wrong_hash = guest
        .book(&listing, &requests, &parse_op(None)?, Some("00"))
        .await;
    checks.push(json!({
        "name": "wrong terms hash refuses",
        "ok": matches!(wrong_hash, Err(SdkError::Refused { ref reason, .. }) if reason == "terms"),
        "error": format!("{wrong_hash:?}"),
    }));

    // A non-instant listing exercises accept and provider cancel.
    let mut manual = Terms::default_for(ResourceKind::Robot);
    manual.instant_book = false;
    let height = provider.height().await?;
    let manual_listing_id = format!("listing:client-manual:{}", now_ms());
    let manual_listing = provider
        .create_listing(
            &parse_uri(&fixtures.provider_uri)?,
            &manual_listing_id,
            ResourceKind::Robot,
            &manual,
        )
        .await?;
    let manual_unit = format!("unit:{manual_listing_id}");
    let manual_slots = [SlotSpec {
        slot_id: "s1".into(),
        start_height: height + 200,
        end_height: height + 300,
    }];
    provider
        .ensure_unit(
            &parse_uri(&manual_listing.value)?,
            &manual_unit,
            &json!({"name": "manual robot"}),
            &manual_slots,
            manual.price_per_slot,
        )
        .await?;
    provider
        .activate_listing(&parse_uri(&manual_listing.value)?)
        .await?;
    let manual_requests = vec![UnitSlotRequest {
        unit_id: manual_unit,
        slots: vec!["s1".into()],
    }];
    let manual_booked = guest
        .book(
            &parse_uri(&manual_listing.value)?,
            &manual_requests,
            &parse_op(None)?,
            None,
        )
        .await?;
    let requested = guest
        .booking_info(&parse_uri(&manual_booked.booking_uri)?)
        .await?;
    let accepted = provider
        .booking_action(
            &parse_uri(&manual_booked.booking_uri)?,
            &BookingAction::Accept,
            &parse_op(None)?,
        )
        .await;
    let confirmed = guest
        .booking_info(&parse_uri(&manual_booked.booking_uri)?)
        .await?;
    let cancelled = provider
        .booking_action(
            &parse_uri(&manual_booked.booking_uri)?,
            &BookingAction::CancelByProvider,
            &parse_op(None)?,
        )
        .await;
    let final_state = guest
        .booking_info(&parse_uri(&manual_booked.booking_uri)?)
        .await?;
    checks.push(json!({
        "name": "manual listing: requested -> accepted -> cancelled",
        "ok": requested.value.status.as_str() == "requested"
            && accepted.is_ok()
            && confirmed.value.status.as_str() == "confirmed"
            && cancelled.is_ok()
            && final_state.value.status.as_str() == "cancelled",
        "requested": requested.value.status.as_str(),
        "confirmed": confirmed.value.status.as_str(),
        "final": final_state.value.status.as_str(),
    }));

    // One admin signature is below the installed 2-of-3 threshold; two succeed.
    let admin1 = client_for(config, Some("admin1")).await?;
    let admin2 = client_for(config, Some("admin2")).await?;
    ensure_native(&admin1).await?;
    ensure_native(&admin2).await?;
    let nonce = admin1.height().await? + 1000;
    let payload = admin1
        .admin_payload("admin", nonce, "pauseAll", &[])
        .await?;
    let sig1 = admin1.admin_sign(&payload.value)?;
    let sig2 = admin2.admin_sign(&payload.value)?;
    let one = admin1
        .admin_submit_share(nonce, "pauseAll", &[], std::slice::from_ref(&sig1))
        .await;
    checks.push(json!({
        "name": "one admin signature is below threshold",
        "ok": matches!(one, Err(SdkError::Refused { ref reason, .. }) if reason == "below threshold"),
        "error": format!("{one:?}"),
    }));
    let paused = admin1
        .admin_submit_share(nonce, "pauseAll", &[], &[sig1, sig2])
        .await;
    let resume_payload = admin1
        .admin_payload("admin", nonce + 1, "resumeAll", &[])
        .await?;
    let resume_sigs = [
        admin1.admin_sign(&resume_payload.value)?,
        admin2.admin_sign(&resume_payload.value)?,
    ];
    let resumed = admin1
        .admin_submit_share(nonce + 1, "resumeAll", &[], &resume_sigs)
        .await;
    checks.push(json!({
        "name": "two admin signatures pause and resume",
        "ok": paused.is_ok() && resumed.is_ok(),
        "pause": format!("{paused:?}"),
        "resume": format!("{resumed:?}"),
    }));

    let ok = checks
        .iter()
        .all(|check| check.get("ok").and_then(Value::as_bool).unwrap_or(false));
    let report = json!({
        "ok": ok,
        "node": config.node,
        "bookingUri": booked.booking_uri,
        "checks": checks,
    });
    write_json(results_path, &report)?;
    emit(config, "devnet journey", report, None)?;
    if ok {
        Ok(())
    } else {
        Err(CliError::Failed("one or more journey checks failed".into()))
    }
}

async fn ensure_native(client: &Client) -> Result<(), CliError> {
    let address = client.address()?;
    if client.native_rev_balance(&address).await?.value >= 2_000_000 {
        return Ok(());
    }
    for _ in 0..6 {
        let _ = client.faucet(&address).await;
        tokio::time::sleep(Duration::from_secs(2)).await;
        if client.native_rev_balance(&address).await?.value >= 2_000_000 {
            return Ok(());
        }
    }
    Err(CliError::Config(format!(
        "{address} still has no native REV after six faucet attempts"
    )))
}

async fn ensure_account(client: &Client, need: i64) -> Result<(), CliError> {
    let balance = client
        .account_balance()
        .await
        .map(|read| read.value.rev_base_units)
        .unwrap_or_default();
    if balance < need {
        client.fund_account_rev(need + 1_000).await?;
    }
    Ok(())
}

fn emit(
    config: &Config,
    command: &str,
    data: Value,
    meta: Option<&ReadMeta>,
) -> Result<(), CliError> {
    let value = if config.json {
        json!({
            "ok": true,
            "command": command,
            "data": data,
            "read": meta.map(meta_json),
        })
    } else {
        data
    };
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
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

fn parse_uri(value: &str) -> Result<RegistryUri, CliError> {
    RegistryUri::new(value).map_err(|error| CliError::Config(error.to_string()))
}

fn parse_kind(value: &str) -> Result<ResourceKind, CliError> {
    ResourceKind::parse(value)
        .ok_or_else(|| CliError::Config(format!("unknown resource kind: {value}")))
}

fn parse_json(value: &str, label: &str) -> Result<Value, CliError> {
    serde_json::from_str(value).map_err(|error| CliError::Config(format!("{label} JSON: {error}")))
}

fn parse_admin_args(value: &str) -> Result<Vec<String>, CliError> {
    let values: Vec<Value> = serde_json::from_str(value)
        .map_err(|error| CliError::Config(format!("admin args JSON: {error}")))?;
    let mut rendered = Vec::with_capacity(values.len());
    for value in values {
        rendered.push(render_json(&value)?);
    }
    Ok(rendered)
}

fn parse_op(value: Option<&str>) -> Result<OpId, CliError> {
    match value {
        Some(value) => Ok(OpId::new(value)?),
        None => Ok(new_op_id("op", now_ms())?),
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

fn load_terms(
    kind: &str,
    inline: Option<String>,
    file: Option<PathBuf>,
) -> Result<Terms, CliError> {
    let kind = parse_kind(kind)?;
    let text = match (file, inline) {
        (Some(path), _) => std::fs::read_to_string(path)?,
        (None, Some(text)) => text,
        (None, None) => {
            return Err(CliError::Config(
                "pass --terms <json> or --file <path>".into(),
            ))
        }
    };
    Ok(Terms::from_json(kind, &text)?)
}

fn parse_unit_slots(value: &str) -> Result<Vec<UnitSlotRequest>, CliError> {
    let text = if Path::new(value).is_file() {
        std::fs::read_to_string(value)?
    } else {
        value.to_string()
    };
    serde_json::from_str(&text)
        .map_err(|error| CliError::Config(format!("unit-slots JSON: {error}")))
}

fn parse_slot_specs(value: &str) -> Result<Vec<SlotSpec>, CliError> {
    let text = if Path::new(value).is_file() {
        std::fs::read_to_string(value)?
    } else {
        value.to_string()
    };
    serde_json::from_str(&text).map_err(|error| CliError::Config(format!("slots JSON: {error}")))
}

fn require_key(client: &Client) -> Result<(), CliError> {
    if client.has_key() {
        Ok(())
    } else {
        Err(CliError::Config(
            "this command needs a signing key; pass --key and a devnet keys file".into(),
        ))
    }
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))?;
    Ok(())
}
