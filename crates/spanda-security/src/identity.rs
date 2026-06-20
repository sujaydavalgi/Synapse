//! Robot and device identity with trust metadata for secure Spanda programs.

use crate::trust::TrustLevel;
use serde::{Deserialize, Serialize};
use spanda_audit::DeviceIdentity;

/// Extended device identity with trust metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RobotIdentity {
    pub device: DeviceIdentity,
    pub trust: TrustLevel,
}

impl RobotIdentity {
    pub fn new(id: impl Into<String>, public_key: impl Into<String>) -> Self {
        // Create a new instance.
        //
        // Parameters:
        // - `id` — input value
        // - `public_key` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_security::identity::new(id, public_key);

        // Assemble the struct fields and return it.
        Self {
            device: DeviceIdentity::new(id, public_key),
            trust: TrustLevel::Trusted,
        }
    }

    pub fn with_trust(mut self, trust: TrustLevel) -> Self {
        //
        // Parameters:
        // - `mut self` — input value
        // - `trust` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_security::identity::with_trust(mut self, trust);

        // Call trust = trust; on the current instance.
        self.trust = trust;
        self
    }

    pub fn id(&self) -> &str {
        // Id.
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
        // let result = instance.id();

        // Return id from this handle.
        &self.device.id
    }

    pub fn public_key(&self) -> &str {
        // Public key.
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
        // let result = instance.public_key();

        // Return public key from this handle.
        &self.device.public_key
    }

    pub fn signing_key(&self) -> String {
        // Signing key.
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
        // let result = instance.signing_key();

        // Call default key on the current instance.
        self.device.default_key()
    }
}

impl From<DeviceIdentity> for RobotIdentity {
    fn from(device: DeviceIdentity) -> Self {
        // From.
        //
        // Parameters:
        // - `device` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_security::identity::from(device);

        // Assemble the struct fields and return it.
        Self {
            device,
            trust: TrustLevel::Trusted,
        }
    }
}
