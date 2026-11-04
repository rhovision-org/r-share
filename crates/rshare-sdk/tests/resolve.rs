//! Offline tests for the wire glue and Root resolution.

use std::sync::Arc;

use async_trait::async_trait;
use rcash_chain::node::transport::{HttpRequest, HttpResponse, Transport};
use rcash_chain::node::NodeClient;
use rcash_chain::ChainError;
use serde_json::json;

use rshare_core::{ComponentKey, Manifest};
use rshare_sdk::Client;

struct FakeTransport {
    body: String,
}

#[async_trait(?Send)]
impl Transport for FakeTransport {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse, ChainError> {
        assert!(
            request.url.ends_with("/api/v1/explore-deploy"),
            "unexpected request {}",
            request.url
        );
        assert!(request.body.is_some(), "explore needs a term");
        Ok(HttpResponse {
            status: 200,
            body: self.body.clone(),
            retry_after: None,
        })
    }
}

fn client(body: serde_json::Value) -> Client {
    let manifest = Manifest::from_json(r#"{"root_uri":"rho:id:root123"}"#).expect("manifest");
    let transport = Arc::new(FakeTransport {
        body: body.to_string(),
    });
    let node = NodeClient::with_transport("http://node.test", transport).expect("node");
    Client::with_node(node, manifest, None, "/root", 4_000_000, 1).expect("client")
}

fn resolve_body(uri: &str, version: i64) -> serde_json::Value {
    json!({
        "expr": [{
            "ExprTuple": {"data": [
                {"ExprBool": {"data": true}},
                {"ExprUri": {"data": uri}},
                {"ExprInt": {"data": version}}
            ]}
        }],
        "block": {"blockHash": "block-1", "blockNumber": 42, "shardId": "/root", "timestamp": 0},
        "replySource": "firstPrivateName"
    })
}

#[tokio::test(flavor = "current_thread")]
async fn resolves_a_component_and_reads_the_block_height() {
    let client = client(resolve_body("rho:id:terms123", 7));
    let resolved = client.resolve(ComponentKey::Terms).await.unwrap();
    assert_eq!(resolved.uri.as_str(), "rho:id:terms123");
    assert_eq!(resolved.version, 7);
}

#[tokio::test(flavor = "current_thread")]
async fn resolve_many_decodes_the_map_and_caches() {
    let client = client(json!({
        "expr": [{
            "ExprTuple": {"data": [
                {"ExprBool": {"data": true}},
                {"ExprMap": {"data": {
                    "component:terms": {"ExprMap": {"data": {
                        "uri": {"ExprUri": {"data": "rho:id:terms123"}},
                        "version": {"ExprInt": {"data": 3}}
                    }}},
                    "component:remedy": {"ExprMap": {"data": {
                        "uri": {"ExprUri": {"data": "rho:id:remedy456"}},
                        "version": {"ExprInt": {"data": 4}}
                    }}}
                }}}
            ]}
        }],
        "block": {"blockHash": "block-2", "blockNumber": 43, "shardId": "/root", "timestamp": 0},
        "replySource": "firstPrivateName"
    }));
    let resolved = client
        .resolve_many(&[ComponentKey::Terms, ComponentKey::Remedy])
        .await
        .unwrap();
    assert_eq!(resolved.len(), 2);
    assert_eq!(
        resolved[&ComponentKey::Terms].uri.as_str(),
        "rho:id:terms123"
    );
    assert_eq!(resolved[&ComponentKey::Remedy].version, 4);
}

#[tokio::test(flavor = "current_thread")]
async fn surfaces_a_designed_refusal() {
    let client = client(json!({
        "expr": [{
            "ExprTuple": {"data": [
                {"ExprBool": {"data": false}},
                {"ExprString": {"data": "unknown component"}}
            ]}
        }],
        "block": {"blockHash": "block-3", "blockNumber": 44, "shardId": "/root", "timestamp": 0},
        "replySource": "firstPrivateName"
    }));
    let error = client.resolve(ComponentKey::Terms).await.unwrap_err();
    match error {
        rshare_sdk::SdkError::Refused { reason, .. } => {
            assert_eq!(reason, "unknown component")
        }
        other => panic!("expected refusal, got {other:?}"),
    }
}
