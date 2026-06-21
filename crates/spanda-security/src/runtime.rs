//! runtime support for Spanda.
//!
use crate::capability::{capability_for_operation, CapabilitySet};
use crate::error::{SecurityError, SecurityResult};
use crate::identity::RobotIdentity;
use crate::permissions::PackagePermissions;
use crate::policy::{AuthenticationMode, EncryptionMode, IntegrityMode};
use crate::secrets::SecretStore;
use crate::secure_comm::{SecureEndpointRegistry, SecurePolicy};
use crate::signed::SignedMessage;
use crate::trust::TrustLevel;
use crate::trust_boundary::{TrustBoundaryKind, TrustBoundaryRegistry};
use serde::{Deserialize, Serialize};
use spanda_audit::AuditRuntime;
use std::collections::{HashMap, HashSet};

/// Unified security context for the Spanda interpreter.
#[derive(Debug)]
pub struct SecurityContext {
    pub identity: Option<RobotIdentity>,
    pub trust: TrustLevel,
    pub secrets: SecretStore,
    pub capabilities: CapabilitySet,
    pub secure_endpoints: SecureEndpointRegistry,
    pub trust_boundaries: TrustBoundaryRegistry,
    pub transport_boundary: Option<TrustBoundaryKind>,
    pub bus_encryption: EncryptionMode,
    pub bus_authentication: AuthenticationMode,
    pub bus_integrity: IntegrityMode,
    pub audit_security_events: bool,
    pub strict_permissions: bool,
    pub security_faults_active: HashSet<String>,
    pub wire_cert_path: Option<String>,
    pub wire_key_secret: Option<String>,
    recent_payload_hashes: HashMap<String, HashSet<String>>,
}

impl Default for SecurityContext {
    fn default() -> Self {
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
        // let value = spanda_security::runtime::default();

        // Build the result via new.
        Self::new()
    }
}

impl SecurityContext {
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
        // let value = spanda_security::runtime::new();

        // Assemble the struct fields and return it.
        Self {
            identity: None,
            trust: TrustLevel::Trusted,
            secrets: SecretStore::new(),
            capabilities: CapabilitySet::new(),
            secure_endpoints: SecureEndpointRegistry::new(),
            trust_boundaries: TrustBoundaryRegistry::new(),
            transport_boundary: None,
            bus_encryption: EncryptionMode::None,
            bus_authentication: AuthenticationMode::None,
            bus_integrity: IntegrityMode::None,
            audit_security_events: true,
            strict_permissions: false,
            security_faults_active: HashSet::new(),
            wire_cert_path: None,
            wire_key_secret: None,
            recent_payload_hashes: HashMap::new(),
        }
    }

    pub fn enable_strict_permissions(&mut self) {
        // Enable strict permissions.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.enable_strict_permissions();

        // Call strict permissions = true; on the current instance.
        self.strict_permissions = true;
    }

    pub fn grant_if_not_strict(&mut self, capability: impl Into<String>) {
        // Grant if not strict.
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
        // let result = instance.grant_if_not_strict(capability);

        // take the branch when strict permissions is false.
        if !self.strict_permissions {
            self.capabilities.grant(capability);
        }
    }

    pub fn with_permissions(perms: &PackagePermissions) -> Self {
        //
        // Parameters:
        // - `perms` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_security::runtime::with_permissions(perms);

        // Assemble the struct fields and return it.
        Self {
            capabilities: perms.capabilities.clone(),
            ..Self::new()
        }
    }

    pub fn set_identity(&mut self, identity: RobotIdentity) {
        // Set identity.
        //
        // Parameters:
        // - `self` — method receiver
        // - `identity` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.set_identity(identity);

        // Call trust; on the current instance.
        self.trust = identity.trust;
        self.identity = Some(identity);
    }

    pub fn require_operation(&self, operation: &str) -> SecurityResult<()> {
        // Require operation.
        //
        // Parameters:
        // - `self` — method receiver
        // - `operation` — input value
        //
        // Returns:
        // SecurityResult<()>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.require_operation(operation);

        // use cap when capability for operation is present.

        // Emit output when capability for operation provides a cap.
        if let Some(cap) = capability_for_operation(operation) {
            self.capabilities.require(cap)?;
        }
        Ok(())
    }

    pub fn register_secure_endpoint(&mut self, path: impl Into<String>, policy: SecurePolicy) {
        // Register secure endpoint.
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
        // let result = instance.register_secure_endpoint(path, policy);

        // Call register on the current instance.
        self.secure_endpoints.register(path, policy);
    }

    pub fn set_transport_context(
        &mut self,
        boundary: Option<TrustBoundaryKind>,
        encryption: EncryptionMode,
        authentication: AuthenticationMode,
        integrity: IntegrityMode,
    ) {
        self.transport_boundary = boundary;
        self.bus_encryption = encryption;
        self.bus_authentication = authentication;
        self.bus_integrity = integrity;
    }

    pub fn enforce_trust_boundary(
        &self,
        message_type: &str,
        endpoint: &SecurePolicy,
    ) -> SecurityResult<()> {
        let Some(boundary) = self.transport_boundary else {
            return Ok(());
        };
        if !self.trust_boundaries.contains(boundary) {
            return Ok(());
        }
        let encryption = if endpoint.encryption != EncryptionMode::None {
            endpoint.encryption
        } else {
            self.bus_encryption
        };
        let authentication = if endpoint.authentication != AuthenticationMode::None {
            endpoint.authentication
        } else {
            self.bus_authentication
        };
        let integrity = if endpoint.integrity != IntegrityMode::None {
            endpoint.integrity
        } else {
            self.bus_integrity
        };
        self.trust_boundaries.validate_channel(
            boundary,
            encryption,
            authentication,
            integrity,
            message_type,
        )
    }

    pub fn sign_outbound(
        &self,
        path: &str,
        payload: &str,
    ) -> SecurityResult<Option<SignedMessage>> {
        // Sign outbound.
        //
        // Parameters:
        // - `self` — method receiver
        // - `path` — input value
        // - `payload` — input value
        //
        // Returns:
        // SecurityResult<Option<SignedMessage>>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.sign_outbound(path, payload);

        // Compute policy for the following logic.
        let policy = self.secure_endpoints.policy_or_open(path);
        policy.prepare_outbound(payload, self.identity.as_ref(), &self.capabilities, path)
    }

    pub fn verify_inbound(
        &self,
        path: &str,
        signed: Option<&SignedMessage>,
        source_id: Option<&str>,
    ) -> SecurityResult<()> {
        // Verify inbound.
        //
        // Parameters:
        // - `self` — method receiver
        // - `path` — input value
        // - `signed` — input value
        // - `source_id` — optional publisher identity for trusted-source checks
        //
        // Returns:
        // SecurityResult<()>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.verify_inbound(path, signed, source_id);

        // Compute policy for the following logic.
        let policy = self.secure_endpoints.policy_or_open(path);
        policy.verify_inbound(
            signed,
            self.identity.as_ref(),
            &self.capabilities,
            path,
            source_id,
        )
    }

    pub fn authorize_subscribe(&self, path: &str) -> SecurityResult<()> {
        let policy = self.secure_endpoints.policy_or_open(path);
        if policy.encryption != crate::policy::EncryptionMode::None
            || policy.signed
            || policy.authentication != crate::policy::AuthenticationMode::None
            || !policy.trusted_sources.is_empty()
        {
            self.capabilities.require("secure_topic.subscribe")?;
        }
        Ok(())
    }

    pub fn verify_inbound_message(
        &mut self,
        path: &str,
        payload: &str,
        source_id: Option<&str>,
        signed: Option<&SignedMessage>,
        message_type: &str,
    ) -> SecurityResult<()> {
        let policy = self.secure_endpoints.policy_or_open(path);
        self.enforce_trust_boundary(message_type, &policy)?;
        self.authorize_subscribe(path)?;
        self.check_security_faults(path, payload)?;
        self.verify_inbound(path, signed, source_id)
    }

    pub fn inject_security_fault(&mut self, fault: impl Into<String>) {
        self.security_faults_active.insert(fault.into());
    }

    pub fn authorize_publish(&self, path: &str, source_id: &str) -> SecurityResult<()> {
        let policy = self.secure_endpoints.policy_or_open(path);
        if !policy.trusted_sources.is_empty() {
            policy.check_trusted_source(source_id)?;
            self.capabilities.require("secure_topic.publish")?;
        } else if policy.encryption != crate::policy::EncryptionMode::None
            || policy.signed
            || policy.authentication != crate::policy::AuthenticationMode::None
        {
            self.capabilities.require("secure_topic.publish")?;
        }
        Ok(())
    }

    pub fn check_security_faults(&mut self, path: &str, payload: &str) -> SecurityResult<()> {
        if self.security_faults_active.contains("InvalidSignature") {
            return Err(SecurityError::SignatureInvalid);
        }
        if self.security_faults_active.contains("ExpiredCertificate") {
            return Err(SecurityError::CertificateExpired {
                subject: self
                    .identity
                    .as_ref()
                    .map(|i| i.id().to_string())
                    .unwrap_or_else(|| "unknown".into()),
            });
        }
        if self.security_faults_active.contains("ReplayAttack") {
            let hash = spanda_audit::sha256(payload).0;
            let seen = self
                .recent_payload_hashes
                .entry(path.to_string())
                .or_default();
            if seen.contains(&hash) {
                return Err(SecurityError::ReplayDetected {
                    endpoint: path.to_string(),
                });
            }
            seen.insert(hash);
        }
        if self.security_faults_active.contains("ManInTheMiddle") {
            return Err(SecurityError::AuthenticationFailed {
                reason: "man-in-the-middle detected".into(),
            });
        }
        if self
            .security_faults_active
            .contains("SecureHandshakeDropped")
        {
            return Err(SecurityError::SecureEndpoint {
                endpoint: path.to_string(),
                reason: "secure handshake dropped".into(),
            });
        }
        Ok(())
    }

    pub fn configure_wire_session(
        &mut self,
        cert_path: Option<String>,
        key_secret: Option<String>,
    ) {
        // Store cert path and key secret name for wire encryption session derivation.
        self.wire_cert_path = cert_path;
        self.wire_key_secret = key_secret;
    }

    pub fn wire_session_material(&self) -> String {
        // Derive session key material from configured cert path and resolved key secret.
        let key = self
            .wire_key_secret
            .as_ref()
            .and_then(|name| self.secrets.resolve(name).ok())
            .unwrap_or_else(|| "spanda-local-key".into());
        format!(
            "{}:{}",
            self.wire_cert_path.as_deref().unwrap_or("spanda-local"),
            key
        )
    }

    pub fn prepare_publish(
        &mut self,
        path: &str,
        payload: &str,
        source_id: &str,
        message_type: &str,
    ) -> SecurityResult<Option<SignedMessage>> {
        let policy = self.secure_endpoints.policy_or_open(path);
        self.enforce_trust_boundary(message_type, &policy)?;
        self.authorize_publish(path, source_id)?;
        self.check_security_faults(path, payload)?;
        let policy = self.secure_endpoints.policy_or_open(path);
        if policy.encryption != crate::policy::EncryptionMode::None {
            self.capabilities.require("crypto.encrypt")?;
            let material = self.wire_session_material();
            let _ = policy.encrypt_payload(payload, &self.capabilities, &material)?;
        }
        self.sign_outbound(path, payload)
    }

    pub fn audit_security_event(
        &self,
        audit: &mut AuditRuntime,
        event_type: &str,
        detail: &str,
    ) -> SecurityResult<()> {
        self.audit_event(audit, event_type, detail)
    }

    /// Record security-relevant events into the audit log when configured.
    pub fn audit_event(
        &self,
        audit: &mut AuditRuntime,
        event_type: &str,
        detail: &str,
    ) -> SecurityResult<()> {
        // Audit event.
        //
        // Parameters:
        // - `self` — method receiver
        // - `audit` — input value
        // - `event_type` — input value
        // - `detail` — input value
        //
        // Returns:
        // SecurityResult<()>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.audit_event(audit, event_type, detail);

        // take the branch when audit security events is false.
        if !self.audit_security_events {
            return Ok(());
        }
        self.require_operation("audit.record")?;
        let redacted = detail.to_string();
        audit
            .record_event(&format!("security.{event_type}"), &redacted)
            .map_err(|e| SecurityError::Other(format!("audit failed: {e}")))?;
        Ok(())
    }
}

/// Serializable snapshot of security state for export/debugging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySnapshot {
    pub identity_id: Option<String>,
    pub trust: TrustLevel,
    pub granted_capabilities: Vec<String>,
    pub secret_names: Vec<String>,
    pub secure_endpoint_count: usize,
}

impl SecurityContext {
    pub fn snapshot(&self) -> SecuritySnapshot {
        // Snapshot.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // SecuritySnapshot.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.snapshot();

        // Produce SecuritySnapshot as the result.
        SecuritySnapshot {
            identity_id: self.identity.as_ref().map(|i| i.id().to_string()),
            trust: self.trust,
            granted_capabilities: self.capabilities.granted().map(str::to_string).collect(),
            secret_names: self.secrets.names().map(str::to_string).collect(),
            secure_endpoint_count: self.secure_endpoints.len(),
        }
    }
}
