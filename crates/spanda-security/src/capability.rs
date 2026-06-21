//! capability support for Spanda.
//!
use crate::error::{SecurityError, SecurityResult};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Known package/runtime capability identifiers.
pub fn known_capabilities() -> &'static [&'static str] {
    // Known capabilities.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // &'static [&'static str].
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_security::capability::known_capabilities();

    // Return the static list of known values.
    &[
        "network.outbound",
        "network.inbound",
        "camera.read",
        "lidar.read",
        "imu.read",
        "gps.read",
        "network.status",
        "wifi.connect",
        "bluetooth.scan",
        "bluetooth.pair",
        "cellular.connect",
        "network.failover",
        "motion.propose",
        "actuator.execute",
        "actuator.execute.safe",
        "serial.port",
        "storage.read",
        "storage.write",
        "ai.inference",
        "ros2.publish",
        "ros2.subscribe",
        "audit.write",
        "audit.read",
        "identity.sign",
        "identity.verify",
        "ledger.anchor",
    ]
}

pub fn is_known_capability(cap: &str) -> bool {
    //
    // Parameters:
    // - `cap` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_security::capability::is_known_capability(cap);

    // Produce contains as the result.
    known_capabilities().contains(&cap)
}

/// Granted capability token (maps to package `[capabilities]` and robot `permissions`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    pub capability: String,
}

impl Permission {
    pub fn new(capability: impl Into<String>) -> Self {
        // Create a new instance.
        //
        // Parameters:
        // - `capability` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_security::capability::new(capability);

        // Assemble the struct fields and return it.
        Self {
            capability: capability.into(),
        }
    }
}

/// Set of granted capabilities with runtime enforcement.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    granted: HashSet<String>,
    permissive: bool,
}

impl CapabilitySet {
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
        // let value = spanda_security::capability::new();

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
        // let result = spanda_security::capability::permissive();

        // Assemble the struct fields and return it.
        Self {
            granted: known_capabilities()
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            permissive: true,
        }
    }

    pub fn grant(&mut self, capability: impl Into<String>) {
        // Grant.
        //
        // Parameters:
        // - `self` — method receiver
        // - `capability` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.grant(capability);

        // Append into self.
        self.granted.insert(capability.into());
    }

    pub fn grant_all(&mut self, caps: impl IntoIterator<Item = impl Into<String>>) {
        // Grant all.
        //
        // Parameters:
        // - `self` — method receiver
        // - `caps` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.grant_all(caps);

        // Validate each requested capability.
        for cap in caps {
            self.grant(cap);
        }
    }

    pub fn has(&self, capability: &str) -> bool {
        // Has.
        //
        // Parameters:
        // - `self` — method receiver
        // - `capability` — input value
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.has(capability);

        // Call contains on the current instance.
        self.permissive || self.granted.contains(capability)
    }

    pub fn require(&self, capability: &str) -> SecurityResult<()> {
        // Require.
        //
        // Parameters:
        // - `self` — method receiver
        // - `capability` — input value
        //
        // Returns:
        // SecurityResult<()>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.require(capability);

        // take this path when self.has(capability).
        if self.has(capability) {
            Ok(())
        } else {
            Err(SecurityError::CapabilityDenied(capability.to_string()))
        }
    }

    pub fn granted(&self) -> impl Iterator<Item = &str> {
        // Granted.
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
        // let result = instance.granted();

        // Iterate over granted.
        self.granted.iter().map(String::as_str)
    }
}

/// Maps high-level runtime operations to required package capabilities.
pub fn capability_for_operation(operation: &str) -> Option<&'static str> {
    // Capability for operation.
    //
    // Parameters:
    // - `operation` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_security::capability::capability_for_operation(operation);

    // Match on operation and handle each case.
    match operation {
        "audit.record" | "audit.append" => Some("audit.write"),
        "audit.export" | "audit.read" => Some("audit.read"),
        "sign" | "identity.sign" => Some("identity.sign"),
        "verify_signature" | "identity.verify" => Some("identity.verify"),
        "ledger.anchor" => Some("ledger.anchor"),
        "actuator.execute" => Some("actuator.execute"),
        "cellular.sim_identity" => Some("cellular.connect"),
        "network.publish" => Some("network.outbound"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_enforcement() {
        // Capability enforcement.
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
        // let result = spanda_security::capability::capability_enforcement();

        let mut caps = CapabilitySet::new();
        caps.grant("audit.write");
        assert!(caps.require("audit.write").is_ok());
        assert!(caps.require("ledger.anchor").is_err());
    }
}
