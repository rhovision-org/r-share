#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

//! Cross-decrypt proof against a document produced by r-cash's own keystore code.

const DOCUMENT: &str = include_str!("fixtures/rcash-v3.json");
const SECRET_HEX: &str = "030a11181f262d343b424950575e656c737a81888f969da4abb2b9c0c7ced5dc";
const PASSWORD: &str = "r-share-fixture";

#[test]
fn decrypts_a_document_written_by_rcash() {
    let secret = rshare_keystore::decrypt(DOCUMENT, PASSWORD).expect("decrypt");
    let hex: String = secret.iter().map(|byte| format!("{byte:02x}")).collect();
    assert_eq!(hex, SECRET_HEX);
}

#[test]
fn the_fixture_address_matches_the_derived_public_key() {
    let secret = rshare_keystore::decrypt(DOCUMENT, PASSWORD).expect("decrypt");
    let key = rcash_chain::KeyPair::from_secret_bytes(&secret).expect("key");
    let address = rcash_chain::address::eth_address_from_public_key(key.public_bytes())
        .expect("ethereum address");
    let document: serde_json::Value = serde_json::from_str(DOCUMENT).expect("json");
    assert_eq!(document["address"], address);
}

#[test]
fn a_wrong_password_is_refused() {
    assert!(rshare_keystore::decrypt(DOCUMENT, "not-the-password").is_err());
}
