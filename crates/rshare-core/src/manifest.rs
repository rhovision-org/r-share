//! The deployment manifest written by `harness/install.py`.
//!
//! Only `root_uri` is required. The component map is advisory: the Root is the authority,
//! and a client must resolve keys through it rather than trust a copied URI.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::{Result, RshareError};
use crate::RegistryUri;

/// A deployment manifest. `components` maps a deploy file to the URIs it registered.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub root_uri: RegistryUri,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub components: BTreeMap<String, Vec<Vec<String>>>,
}

impl Manifest {
    /// Parse a manifest document and validate the root URI.
    pub fn from_json(document: &str) -> Result<Self> {
        let value: serde_json::Value = serde_json::from_str(document)
            .map_err(|error| RshareError::Manifest(error.to_string()))?;
        let root = value
            .get("root_uri")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| RshareError::Manifest("missing root_uri".into()))?;
        let root_uri = RegistryUri::new(root)?;
        let components = value
            .get("components")
            .cloned()
            .map(serde_json::from_value)
            .transpose()
            .map_err(|error| RshareError::Manifest(format!("components: {error}")))?
            .unwrap_or_default();
        Ok(Self {
            root_uri,
            components,
        })
    }

    /// Serialize the manifest back to JSON.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(RshareError::from)
    }

    /// The Root URI.
    pub fn root_uri(&self) -> &RegistryUri {
        &self.root_uri
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_installer_shape() {
        let manifest = Manifest::from_json(
            r#"{"root_uri":"rho:id:abc123","components":{"00_root.rho":[["rho:id:abc123"]]}}"#,
        )
        .unwrap();
        assert_eq!(manifest.root_uri.as_str(), "rho:id:abc123");
        assert_eq!(manifest.components.len(), 1);
    }

    #[test]
    fn components_are_optional() {
        let manifest = Manifest::from_json(r#"{"root_uri":"rho:id:abc123"}"#).unwrap();
        assert!(manifest.components.is_empty());
    }

    #[test]
    fn refuses_a_non_registry_root() {
        assert!(Manifest::from_json(r#"{"root_uri":"http://example.com"}"#).is_err());
    }
}
