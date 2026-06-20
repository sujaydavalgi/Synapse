//! hardware req support for Spanda.
//!
use crate::error::{PackageError, PackageResult};
use serde::{Deserialize, Serialize};

/// Hardware requirements declared in `[requires_hardware]` of spanda.toml.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct HardwareRequirements {
    #[serde(default)]
    pub memory: Option<String>,
    #[serde(default)]
    pub storage: Option<String>,
    #[serde(default)]
    pub gpu: Option<String>,
    #[serde(default)]
    pub sensors: Vec<String>,
    #[serde(default)]
    pub actuators: Vec<String>,
}

impl HardwareRequirements {
    /// Parse memory string like `">=2GB"` into megabytes.
    pub fn memory_mb_min(&self) -> Option<f64> {
        // Memory mb min.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.memory_mb_min();

        // Transform self and continue the chain.
        self.memory.as_ref().and_then(|s| parse_memory_mb(s))
    }

    pub fn storage_mb_min(&self) -> Option<f64> {
        // Storage mb min.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.storage_mb_min();

        // Transform self and continue the chain.
        self.storage.as_ref().and_then(|s| parse_memory_mb(s))
    }

    pub fn gpu_tops_min(&self) -> Option<f64> {
        // Gpu tops min.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.gpu_tops_min();

        // Transform self and continue the chain.
        self.gpu.as_ref().and_then(|s| parse_gpu_tops(s))
    }

    pub fn gpu_required(&self) -> bool {
        // Gpu required.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.gpu_required();

        // Call is some on the current instance.
        self.gpu.is_some()
    }
}

fn parse_memory_mb(s: &str) -> Option<f64> {
    // Parse memory mb.
    //
    // Parameters:
    // - `s` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::hardware_req::parse_memory_mb(s);

    // Compute s for the following logic.
    let s = s.trim();
    let (op, rest) = if let Some(r) = s.strip_prefix(">=") {
        (">=", r.trim())
    } else if let Some(r) = s.strip_prefix('>') {
        (">", r.trim())
    } else {
        ("=", s)
    };
    let rest_upper = rest.to_uppercase();
    let (num_str, unit) = if rest_upper.ends_with("GB") {
        (&rest[..rest.len() - 2], "GB")
    } else if rest_upper.ends_with("MB") {
        (&rest[..rest.len() - 2], "MB")
    } else if rest_upper.ends_with('G') {
        (&rest[..rest.len() - 1], "GB")
    } else if rest_upper.ends_with('M') {
        (&rest[..rest.len() - 1], "MB")
    } else {
        (rest, "MB")
    };
    let value: f64 = num_str.trim().parse().ok()?;
    let mb = match unit {
        "GB" => value * 1024.0,
        _ => value,
    };

    // Match on op and handle each case.
    match op {
        ">" => Some(mb + 1.0),
        _ => Some(mb),
    }
}

fn parse_gpu_tops(s: &str) -> Option<f64> {
    // Parse gpu tops.
    //
    // Parameters:
    // - `s` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::hardware_req::parse_gpu_tops(s);

    // Compute s for the following logic.
    let s = s.trim();
    let rest = s.strip_prefix(">=").unwrap_or(s).trim();
    let rest = rest.strip_suffix("TOPS").unwrap_or(rest).trim();
    rest.parse().ok()
}

/// Capability declarations from `[capabilities]` in spanda.toml.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CapabilityRequirements {
    /// Capabilities the package needs at runtime (`uses`).
    #[serde(default)]
    pub uses: Vec<String>,

    /// Capabilities the consuming application must grant (`required`).
    #[serde(default)]
    pub required: Vec<String>,
}

impl CapabilityRequirements {
    pub fn all(&self) -> impl Iterator<Item = &str> {
        // All.
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
        // let result = instance.all();

        // Call uses on the current instance.
        self.uses
            .iter()
            .chain(self.required.iter())
            .map(String::as_str)
    }
}

/// Known capability identifiers for validation.
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
    // let result = spanda_package::hardware_req::known_capabilities();

    // Return the static list of known values.
    &[
        "network.outbound",
        "network.inbound",
        "camera.read",
        "lidar.read",
        "imu.read",
        "gps.read",
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

/// Capabilities that require explicit application approval.
pub fn high_risk_capabilities() -> &'static [&'static str] {
    // High risk capabilities.
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
    // let result = spanda_package::hardware_req::high_risk_capabilities();

    // Return the static list of known values.
    &[
        "ledger.anchor",
        "identity.sign",
        "actuator.execute",
        "network.outbound",
        "audit.write",
    ]
}

pub fn is_high_risk_capability(cap: &str) -> bool {
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
    // let result = spanda_package::hardware_req::is_high_risk_capability(cap);

    // Produce contains as the result.
    high_risk_capabilities().contains(&cap)
}

pub fn validate_capability(cap: &str) -> PackageResult<()> {
    // Validate capability.
    //
    // Parameters:
    // - `cap` — input value
    //
    // Returns:
    // PackageResult<()>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::hardware_req::validate_capability(cap);

    // Check membership before continuing.
    if known_capabilities().contains(&cap) {
        Ok(())
    } else {
        Err(PackageError::Validation(format!(
            "unknown capability '{cap}' — expected one of: {}",
            known_capabilities().join(", ")
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_memory_gb() {
        // Parses memory gb.
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
        // let result = spanda_package::hardware_req::parses_memory_gb();

        let req = HardwareRequirements {
            memory: Some(">=2GB".into()),
            ..Default::default()
        };
        assert_eq!(req.memory_mb_min(), Some(2048.0));
    }

    #[test]
    fn parses_gpu_tops() {
        // Parses gpu tops.
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
        // let result = spanda_package::hardware_req::parses_gpu_tops();

        let req = HardwareRequirements {
            gpu: Some(">=1 TOPS".into()),
            ..Default::default()
        };
        assert_eq!(req.gpu_tops_min(), Some(1.0));
    }
}
