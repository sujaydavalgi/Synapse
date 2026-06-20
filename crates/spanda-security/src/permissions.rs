//! Application-level package permissions derived from manifest capability declarations.

use crate::capability::CapabilitySet;
use serde::{Deserialize, Serialize};

/// Application-level permissions for package validation and runtime gating.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackagePermissions {
    pub capabilities: CapabilitySet,
}

impl PackagePermissions {
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
        // let value = spanda_security::permissions::new();

        // Build the result via default.
        Self::default()
    }

    pub fn permissive() -> Self {
        // Permissive.
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
        // let result = spanda_security::permissions::permissive();

        // Assemble the struct fields and return it.
        Self {
            capabilities: CapabilitySet::permissive(),
        }
    }

    pub fn from_capabilities(caps: impl IntoIterator<Item = impl Into<String>>) -> Self {
        // Construct from capabilities.
        //
        // Parameters:
        // - `caps` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_security::permissions::from_capabilities(caps);

        // Create mutable set for accumulating results.
        let mut set = CapabilitySet::new();
        set.grant_all(caps);
        Self { capabilities: set }
    }
}
