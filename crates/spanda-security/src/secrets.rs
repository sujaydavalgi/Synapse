//! secrets support for Spanda.
//!
use crate::error::{SecurityError, SecurityResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Where a secret value is resolved from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum SecretSource {
    Env { var: String },
    Literal { value: String },
}

/// Opaque handle to a resolved secret (value is not exposed in logs).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecretHandle {
    pub name: String,
    pub source: SecretSource,
}

impl SecretHandle {
    pub fn resolve(&self) -> SecurityResult<String> {
        // Resolve.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // SecurityResult<String>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.resolve();

        // Match on source and handle each case.
        match &self.source {
            SecretSource::Env { var } => std::env::var(var).map_err(|_| {
                SecurityError::SecretNotFound(format!("environment variable '{var}'"))
            }),
            SecretSource::Literal { value } => Ok(value.clone()),
        }
    }

    /// Redacted representation safe for audit logs.
    pub fn redacted_label(&self) -> String {
        // Redacted label.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Text result.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.redacted_label();

        // Produce name) as the result.
        format!("secret:{}", self.name)
    }
}

/// In-memory secret store keyed by declaration name.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecretStore {
    secrets: HashMap<String, SecretHandle>,
}

impl SecretStore {
    pub fn new() -> Self {
        // Create a new instance.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_security::secrets::new();

        // Build the result via default.
        Self::default()
    }

    pub fn register(&mut self, handle: SecretHandle) {
        // Register the value.
        //
        // Parameters:
        // - `self` — method receiver
        // - `handle` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.register(handle);

        // Append into self.
        self.secrets.insert(handle.name.clone(), handle);
    }

    pub fn get(&self, name: &str) -> SecurityResult<&SecretHandle> {
        // Get.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        //
        // Returns:
        // SecurityResult<&SecretHandle>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.get(name);

        // Call secrets on the current instance.
        self.secrets
            .get(name)
            .ok_or_else(|| SecurityError::SecretNotFound(name.to_string()))
    }

    pub fn resolve(&self, name: &str) -> SecurityResult<String> {
        // Resolve.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        //
        // Returns:
        // SecurityResult<String>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.resolve(name);

        // Call get on the current instance.
        self.get(name)?.resolve()
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        // Names.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // impl Iterator<Item = &str>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.names();

        // Transform self and continue the chain.
        self.secrets.keys().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_secret_roundtrip() {
        // Literal secret roundtrip.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_security::secrets::literal_secret_roundtrip();

        let mut store = SecretStore::new();
        store.register(SecretHandle {
            name: "api_key".into(),
            source: SecretSource::Literal {
                value: "test-key".into(),
            },
        });
        assert_eq!(store.resolve("api_key").unwrap(), "test-key");
    }
}
