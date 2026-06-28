//! Security foundation for Spanda robotics programs.
//!
//! Provides identity, secrets, capability enforcement, signed messages,
//! trust levels, package permissions, and secure communication policies.

pub mod capability;
pub mod encrypted;
pub mod error;
pub mod human_health;
pub mod identity;
pub mod permissions;
pub mod policy;
pub mod rate_limit;
pub mod rbac;
pub mod runtime;
pub mod secret_vault;
pub mod secrets;
pub mod secure_comm;
pub mod signed;
pub mod tenant;
pub mod trust;
pub mod trust_boundary;
pub mod validate;
pub mod wire_crypto;

pub use capability::{
    capability_for_operation, is_known_capability, known_capabilities, CapabilitySet, Permission,
};
pub use encrypted::{
    Certificate, EncryptedMessage, PrivateKey, PublicKey, SessionKey, TrustedSource,
    VerifiedMessage,
};
pub use human_health::{HumanHealthGate, HumanHealthSettings};
pub use error::{SecurityError, SecurityResult};
pub use identity::RobotIdentity;
pub use permissions::PackagePermissions;
pub use policy::{
    AuthenticationMode, BusSecurityConfig, EncryptionMode, IntegrityMode, SecureCommPolicy,
};
pub use rate_limit::RateLimiter;
pub use rbac::{permission_matrix, ApiKeyRecord, ApiKeyStore, RbacAction, RbacContext, Role};
pub use runtime::{SecurityContext, SecuritySnapshot};
pub use secret_vault::{ManagedSecretVault, SecretMetadata, SecretVaultBackend};
pub use secrets::{SecretHandle, SecretSource, SecretStore};
pub use secure_comm::{SecureEndpointRegistry, SecurePolicy};
pub use signed::{Signature, SignedMessage};
pub use tenant::{default_tenant_id, tenant_matches};
pub use trust::TrustLevel;
pub use trust_boundary::{boundary_for_transport_name, TrustBoundaryKind, TrustBoundaryRegistry};
pub use validate::{
    security_analyze_program, security_audit, security_check, SecurityFinding, SecurityReport,
    SecuritySeverity,
};
pub use wire_crypto::WireCryptoSession;
