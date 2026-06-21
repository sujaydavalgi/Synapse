//! secure comm support for Spanda.
//!
use crate::capability::CapabilitySet;
use crate::encrypted::EncryptedMessage;
use crate::error::{SecurityError, SecurityResult};
use crate::identity::RobotIdentity;
use crate::policy::{AuthenticationMode, EncryptionMode, IntegrityMode};
use crate::signed::SignedMessage;
use crate::trust::TrustLevel;
use serde::{Deserialize, Serialize};

/// Security policy attached to a topic, service, or action endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SecurePolicy {
    pub signed: bool,
    pub min_trust: Option<TrustLevel>,
    pub requires: Vec<String>,
    pub encryption: EncryptionMode,
    pub authentication: AuthenticationMode,
    pub integrity: IntegrityMode,
    pub trusted_sources: Vec<String>,
    pub reject_untrusted: bool,
}

impl SecurePolicy {
    pub fn open() -> Self {
        // Open.
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
        // let result = spanda_security::secure_comm::open();

        // Build the result via default.
        Self::default()
    }

    pub fn signed_trusted() -> Self {
        // Signed trusted.
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
        // let result = spanda_security::secure_comm::signed_trusted();

        // Assemble the struct fields and return it.
        Self {
            signed: true,
            min_trust: Some(TrustLevel::Trusted),
            requires: vec!["identity.verify".into()],
            encryption: EncryptionMode::None,
            authentication: AuthenticationMode::Signed,
            integrity: IntegrityMode::None,
            trusted_sources: Vec::new(),
            reject_untrusted: false,
        }
    }

    pub fn encrypted_signed() -> Self {
        Self {
            signed: true,
            encryption: EncryptionMode::Required,
            authentication: AuthenticationMode::Signed,
            integrity: IntegrityMode::Required,
            ..Self::signed_trusted()
        }
    }

    pub fn check_trust(&self, trust: TrustLevel) -> SecurityResult<()> {
        // Check trust.
        //
        // Parameters:
        // - `self` — method receiver
        // - `trust` — input value
        //
        // Returns:
        // SecurityResult<()>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_trust(trust);

        // use required when min trust is present.

        // Emit output when min trust provides a required.
        if let Some(required) = self.min_trust {
            // Take the branch when satisfies is false.
            if !trust.satisfies(required) {
                return Err(SecurityError::TrustInsufficient {
                    required: required.as_str().into(),
                    actual: trust.as_str().into(),
                });
            }
        }
        Ok(())
    }

    pub fn check_capabilities(&self, caps: &CapabilitySet) -> SecurityResult<()> {
        // Check capabilities.
        //
        // Parameters:
        // - `self` — method receiver
        // - `caps` — input value
        //
        // Returns:
        // SecurityResult<()>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_capabilities(caps);

        // Validate each requested capability.
        for cap in &self.requires {
            caps.require(cap)?;
        }
        Ok(())
    }

    pub fn prepare_outbound(
        &self,
        payload: &str,
        identity: Option<&RobotIdentity>,
        caps: &CapabilitySet,
        endpoint: &str,
    ) -> SecurityResult<Option<SignedMessage>> {
        // Prepare outbound.
        //
        // Parameters:
        // - `self` — method receiver
        // - `payload` — input value
        // - `identity` — input value
        // - `caps` — input value
        // - `endpoint` — input value
        //
        // Returns:
        // SecurityResult<Option<SignedMessage>>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.prepare_outbound(payload, identity, caps, endpoint);

        let secured = self.signed
            || self.min_trust.is_some()
            || !self.requires.is_empty()
            || self.encryption != EncryptionMode::None
            || self.authentication != AuthenticationMode::None
            || self.integrity != IntegrityMode::None;

        if secured {
            self.check_capabilities(caps)?;

            if self.encryption == EncryptionMode::Required {
                caps.require("crypto.encrypt")?;
            }
            if self.authentication == AuthenticationMode::Mutual {
                caps.require("identity.verify")?;
            }

            if let Some(id) = identity {
                self.check_trust(id.trust)?;

                if self.signed {
                    caps.require("identity.sign")?;
                    return Ok(Some(SignedMessage::sign(payload, id)));
                }
                return Ok(None);
            }
            return Err(SecurityError::IdentityRequired {
                operation: endpoint.to_string(),
            });
        }
        Ok(None)
    }

    pub fn encrypt_payload(
        &self,
        payload: &str,
        caps: &CapabilitySet,
        session_material: &str,
    ) -> SecurityResult<String> {
        if self.encryption == EncryptionMode::None {
            return Ok(payload.to_string());
        }
        if self.encryption == EncryptionMode::Required {
            caps.require("crypto.encrypt")?;
        }
        let enc = EncryptedMessage::<String>::encrypt(&payload.to_string(), session_material)?;
        Ok(enc.ciphertext().to_string())
    }

    pub fn verify_inbound(
        &self,
        signed: Option<&SignedMessage>,
        identity: Option<&RobotIdentity>,
        caps: &CapabilitySet,
        endpoint: &str,
        source_id: Option<&str>,
    ) -> SecurityResult<()> {
        // Verify inbound.
        //
        // Parameters:
        // - `self` — method receiver
        // - `signed` — input value
        // - `identity` — input value
        // - `caps` — input value
        // - `endpoint` — input value
        //
        // Returns:
        // SecurityResult<()>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.verify_inbound(signed, identity, caps, endpoint);

        if !self.trusted_sources.is_empty() {
            let sid =
                source_id.ok_or_else(|| SecurityError::UntrustedSource("unknown".to_string()))?;
            self.check_trusted_source(sid)?;
        }

        let secured = self.signed
            || self.min_trust.is_some()
            || !self.requires.is_empty()
            || self.encryption != EncryptionMode::None
            || self.authentication != AuthenticationMode::None
            || self.integrity != IntegrityMode::None;

        if secured {
            self.check_capabilities(caps)?;
            let id = identity.ok_or_else(|| SecurityError::IdentityRequired {
                operation: endpoint.to_string(),
            })?;
            self.check_trust(id.trust)?;

            if self.encryption == EncryptionMode::Required {
                caps.require("crypto.decrypt")?;
            }

            if self.signed || self.integrity == IntegrityMode::Required {
                let msg = signed.ok_or_else(|| SecurityError::SecureEndpoint {
                    endpoint: endpoint.to_string(),
                    reason: "missing signature".into(),
                })?;

                if !msg.verify(id)? {
                    return Err(SecurityError::SignatureInvalid);
                }
            }
        }
        Ok(())
    }

    pub fn check_trusted_source(&self, source_id: &str) -> SecurityResult<()> {
        if self.trusted_sources.is_empty() {
            return Ok(());
        }
        if self.trusted_sources.iter().any(|s| s == source_id) {
            Ok(())
        } else if self.reject_untrusted {
            Err(SecurityError::UntrustedSource(source_id.to_string()))
        } else {
            Err(SecurityError::SecureEndpoint {
                endpoint: "trusted_sources".into(),
                reason: format!("untrusted source '{source_id}'"),
            })
        }
    }
}

/// Registry of secure policies keyed by endpoint path.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecureEndpointRegistry {
    policies: std::collections::HashMap<String, SecurePolicy>,
}

impl SecureEndpointRegistry {
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
        // let value = spanda_security::secure_comm::new();

        // Build the result via default.
        Self::default()
    }

    pub fn register(&mut self, path: impl Into<String>, policy: SecurePolicy) {
        // Register the value.
        //
        // Parameters:
        // - `self` — method receiver
        // - `path` — input value
        // - `policy` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.register(path, policy);

        // Append into self.
        self.policies.insert(path.into(), policy);
    }

    pub fn get(&self, path: &str) -> Option<&SecurePolicy> {
        // Get.
        //
        // Parameters:
        // - `self` — method receiver
        // - `path` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.get(path);

        // Call get on the current instance.
        self.policies.get(path)
    }

    pub fn policy_or_open(&self, path: &str) -> SecurePolicy {
        // Policy or open.
        //
        // Parameters:
        // - `self` — method receiver
        // - `path` — input value
        //
        // Returns:
        // SecurePolicy.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.policy_or_open(path);

        // Call get on the current instance.
        self.get(path).cloned().unwrap_or_default()
    }

    pub fn len(&self) -> usize {
        // Len.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Numeric result.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.len();

        // Call len on the current instance.
        self.policies.len()
    }

    pub fn is_empty(&self) -> bool {
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
        // let result = instance.is_empty();

        // Call is empty on the current instance.
        self.policies.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilitySet;

    #[test]
    fn secure_topic_requires_identity() {
        // Secure topic requires identity.
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
        // let result = spanda_security::secure_comm::secure_topic_requires_identity();

        let policy = SecurePolicy::signed_trusted();
        let mut caps = CapabilitySet::new();
        caps.grant("identity.sign");
        caps.grant("identity.verify");
        let err = policy
            .prepare_outbound("data", None, &caps, "/cmd")
            .unwrap_err();
        assert!(matches!(err, SecurityError::IdentityRequired { .. }));
    }
}
