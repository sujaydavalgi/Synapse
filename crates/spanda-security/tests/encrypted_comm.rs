//! Encrypted communication unit tests for spanda-security.

use spanda_security::{
    AuthenticationMode, EncryptedMessage, EncryptionMode, IntegrityMode, SecureCommPolicy,
    SecurePolicy, SessionKey, TrustBoundaryKind,
};

#[test]
fn secure_comm_policy_merge_bus() {
    let base = SecureCommPolicy {
        encryption: EncryptionMode::Optional,
        authentication: AuthenticationMode::None,
        integrity: IntegrityMode::None,
    };
    let merged = base.merge_bus(&spanda_security::BusSecurityConfig {
        encryption: Some(EncryptionMode::Required),
        authentication: Some(AuthenticationMode::Mutual),
        integrity: None,
    });
    assert_eq!(merged.encryption, EncryptionMode::Required);
    assert_eq!(merged.authentication, AuthenticationMode::Mutual);
}

#[test]
fn encrypted_message_roundtrip() {
    let mut msg =
        EncryptedMessage::<String>::encrypt(&"data".to_string(), "sess-material").unwrap();
    assert_eq!(msg.decrypt().unwrap(), "data");
}

#[test]
fn operator_boundary_requires_mutual_auth() {
    assert_eq!(
        TrustBoundaryKind::OperatorToRobot.required_authentication(),
        AuthenticationMode::Mutual
    );
}

#[test]
fn encrypted_signed_policy() {
    let policy = SecurePolicy::encrypted_signed();
    assert!(policy.signed);
    assert_eq!(policy.encryption, EncryptionMode::Required);
}
