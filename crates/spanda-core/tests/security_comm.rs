//! Secure communication language and validation tests.

use spanda_core::{check, lexer::tokenize, parser::parse, security_check};
use spanda_security::{
    EncryptedMessage, EncryptionMode, SecurePolicy, SessionKey, TrustBoundaryKind,
    TrustBoundaryRegistry,
};

#[test]
fn secure_bus_block_parses() {
    let source = r#"
robot R {
  bus robot_mesh {
    transport: "dds";
    encryption: required;
    authentication: mutual;
  };
  behavior run() {}
}
"#;
    check(source).expect("secure bus should parse and type-check");
}

#[test]
fn secure_topic_new_syntax_parses() {
    let source = r#"
robot R {
  topic lidar_scan: Topic<LidarScan> {
    secure {
      encryption required;
      signed required;
      trusted_sources [LidarFront];
    }
  };

  behavior run() {}
}
"#;
    check(source).expect("secure topic block should parse");
}

#[test]
fn secure_service_block_parses() {
    let source = r#"
robot R {
  identity RoverIdentity { id: "r1"; public_key: "k1"; }
  secrets { tls_cert from file "certs/r1.pem"; }
  permissions [ crypto.encrypt, identity.verify, secret.read ];

  service GetBattery: Service<BatteryRequest, BatteryStatus> {
    secure {
      encryption required;
      authentication mutual;
    }
  };

  behavior run() {}
}
"#;
    check(source).expect("secure service should parse");
}

#[test]
fn identity_with_cert_parses() {
    let source = r#"
robot R {
  identity RoverIdentity {
    id: "rover-001";
    public_key: "pub";
    cert: "certs/rover-001.pem";
  };
  behavior run() {}
}
"#;
    check(source).expect("identity with cert should parse");
}

#[test]
fn secrets_block_from_env_and_file() {
    let source = r#"
robot R {
  permissions [ secret.read ];
  secrets {
    rover_private_key from env "ROVER_PRIVATE_KEY";
    tls_cert from file "certs/rover.pem";
  };
  behavior run() {}
}
"#;
    check(source).expect("secrets block should parse");
}

#[test]
fn secure_comm_policy_parses() {
    let source = r#"
robot R {
  secure_comm {
    encryption: required;
    authentication: mutual;
    integrity: required;
  };
  behavior run() {}
}
"#;
    check(source).expect("secure_comm policy should parse");
}

#[test]
fn trust_boundary_declarations_parse() {
    let source = r#"
robot R {
  trust_boundary robot_internal;
  trust_boundary robot_to_robot;
  trust_boundary robot_to_cloud;
  trust_boundary operator_to_robot;
  behavior run() {}
}
"#;
    check(source).expect("trust boundaries should parse");
}

#[test]
fn encrypted_message_type_requires_decrypt() {
    let session = SessionKey {
        id: "test-session".into(),
    };
    let mut msg = EncryptedMessage::<String>::encrypt(&"payload".to_string(), &session).unwrap();
    assert_eq!(msg.decrypt().unwrap(), "payload");
}

#[test]
fn trust_boundary_robot_to_robot_requires_encryption() {
    let reg = TrustBoundaryRegistry::new();
    let err = reg
        .validate_channel(
            TrustBoundaryKind::RobotToRobot,
            EncryptionMode::None,
            spanda_security::AuthenticationMode::None,
            spanda_security::IntegrityMode::None,
            "Pose",
        )
        .unwrap_err();
    assert!(err.to_string().contains("encryption"));
}

#[test]
fn security_check_rejects_secure_topic_without_identity() {
    let source = r#"
robot R {
  topic cmd: Velocity publish on "/cmd" secure {
    encryption required;
    signed required;
  };
  behavior run() {}
}
"#;
    let report = security_check(source).unwrap();
    assert!(report.has_errors());
}

#[test]
fn security_check_rejects_encrypted_bus_without_secrets() {
    let source = r#"
robot R {
  bus mesh {
    transport: "dds";
    encryption: required;
  };
  behavior run() {}
}
"#;
    let report = security_check(source).unwrap();
    assert!(report.has_errors());
}

#[test]
fn insecure_motion_command_detected() {
    let source = r#"
robot R {
  identity RoverIdentity { id: "r1"; public_key: "k1"; }
  trust_boundary robot_to_robot;
  topic motion_cmd: Topic<SafeAction> publish on "/motion";
  behavior run() {}
}
"#;
    let report = security_check(source).unwrap();
    assert!(report.has_errors());
}

#[test]
fn signed_policy_retains_backward_compat() {
    let policy = SecurePolicy::signed_trusted();
    assert!(policy.signed);
    assert_eq!(policy.encryption, EncryptionMode::None);
}

#[test]
fn invalid_signature_example_parses_faults() {
    let source = include_str!("../../../examples/security/invalid_signature.sd");
    let tokens = tokenize(source).expect("tokenize");
    parse(tokens).expect("invalid signature example should parse");
}

#[test]
fn untrusted_source_rejected_at_publish() {
    let source = r#"
robot R {
  trust trusted;
  permissions [
    crypto.encrypt,
    identity.sign,
    identity.verify,
    secure_topic.publish
  ];

  identity RoverIdentity { id: "rover"; public_key: "k1"; }

  topic motion_cmd: Velocity publish on "/motion" {
    secure {
      encryption required;
      signed required;
      trusted_sources [Navigator];
      reject_untrusted true;
    }
  };

  agent BadAgent {
    uses planner;
    tools [];
    goal "bad";
    can [ publish(motion_cmd), plan ];
    plan { publish motion_cmd with velocity(linear: 0.0 m/s, angular: 0.0 rad/s); }
  }

  ai_model planner: LLM { provider: "mock"; model: "test"; temperature: 0.1; }

  behavior run() { BadAgent.plan(); }
}
"#;
    let err = spanda_core::run(
        source,
        spanda_core::RunOptions {
            max_loop_iterations: 1,
            ..Default::default()
        },
    )
    .expect_err("untrusted agent should be rejected");
    assert!(
        err.to_string().contains("untrusted") || err.to_string().contains("Untrusted"),
        "got: {err}"
    );
}

#[test]
fn transport_wire_frame_with_source_id() {
    use spanda_core::comm::{CommBus, TransportKind};
    use spanda_core::runtime::RuntimeValue;
    use spanda_core::transport::{RoutingCommBus, TransportConfig};
    use spanda_core::transport_security::{TlsTransportSession, TransportSecurityConfig};
    use spanda_security::{AuthenticationMode, EncryptionMode, IntegrityMode};

    let mut tls = TlsTransportSession::default();
    let security = TransportSecurityConfig {
        encryption: EncryptionMode::Required,
        authentication: AuthenticationMode::None,
        integrity: IntegrityMode::None,
        cert_path: Some("certs/test.pem".into()),
        key_secret: Some("test_key".into()),
    };
    tls.connect(&security).unwrap();
    let mut bus = RoutingCommBus::new();
    bus.configure(TransportConfig {
        security,
        tls,
        ..Default::default()
    })
    .unwrap();
    bus.subscribe("/motion", "motion_cmd");
    bus.reconnect_transport(TransportKind::Mqtt);
    bus.publish(
        "/motion",
        "Velocity",
        RuntimeValue::Velocity {
            linear: 0.5,
            angular: 0.0,
        },
        TransportKind::Mqtt,
        Some("Navigator"),
    );
    let inbound = bus.poll_inbound(TransportKind::Mqtt);
    assert_eq!(inbound.len(), 1);
    assert_eq!(inbound[0].1.source_id.as_deref(), Some("Navigator"));
}

#[test]
fn inbound_trusted_source_enforced() {
    use spanda_security::{SecurePolicy, SecurityContext};
    let mut ctx = SecurityContext::new();
    ctx.capabilities.grant("secure_topic.subscribe");
    ctx.secure_endpoints.register(
        "/motion",
        SecurePolicy {
            trusted_sources: vec!["Navigator".into()],
            reject_untrusted: true,
            ..Default::default()
        },
    );
    assert!(ctx
        .verify_inbound_message("/motion", "payload", Some("BadAgent"), None)
        .is_err());
    assert!(ctx
        .verify_inbound_message("/motion", "payload", Some("Navigator"), None)
        .is_ok());
}

#[test]
fn timed_security_fault_parses() {
    let source = include_str!("../../../examples/security/invalid_signature.sd");
    let tokens = tokenize(source).expect("tokenize");
    let program = parse(tokens).expect("parse timed faults");
    let spanda_core::Program::Program {
        simulate_compatibility,
        ..
    } = program;
    let sim = simulate_compatibility.expect("simulate_compatibility");
    let spanda_core::foundations::SimulateCompatibilityDecl::SimulateCompatibilityDecl {
        faults,
        ..
    } = sim;
    assert_eq!(faults.len(), 3);
    assert_eq!(faults[0].at_offset_ms, Some(10_000.0));
}

#[test]
fn secret_without_capability_flagged() {
    let source = r#"
robot R {
  secrets { api_key from env "API_KEY"; }
  behavior run() {}
}
"#;
    let report = security_check(source).unwrap();
    assert!(report.has_errors());
}
