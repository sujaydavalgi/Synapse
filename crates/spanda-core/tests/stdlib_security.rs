//! Security foundation integration tests.

use spanda_core::{check, compile, run, RunOptions};
use spanda_security::{
    CapabilitySet, PackagePermissions, RobotIdentity, SecurePolicy, SecurityContext, TrustLevel,
};

#[test]
fn security_example_type_checks() {
    let source = include_str!("../../../examples/std/security.sd");
    check(source).expect("security example should type-check");
}

#[test]
fn security_example_runs_with_permissions() {
    let source = include_str!("../../../examples/std/security.sd");
    let result = run(source, RunOptions::default()).expect("security example should run");
    assert!(result.logs.iter().any(|l| l.contains("audit.record")));
}

#[test]
fn package_capability_set_denies_unknown_ops() {
    let caps = spanda_security::CapabilitySet::new();
    assert!(caps.require("identity.sign").is_err());
}

#[test]
fn secure_topic_requires_identity_at_publish() {
    let source = r#"
robot R {
  permissions [ identity.sign, identity.verify ];

  topic cmd: Velocity publish on "/cmd" secure {
    signed = true;
    min_trust = trusted;
    requires = [ identity.verify ];
  };

  behavior run() {
    publish cmd with velocity(linear: 0.1 m/s, angular: 0.0 rad/s);
  }
}
"#;
    let err = run(source, RunOptions::default()).expect_err("unsigned secure topic should fail");
    assert!(
        err.to_string().contains("Identity required") || err.to_string().contains("identity"),
        "expected identity requirement, got: {err}"
    );
}

#[test]
fn secret_declaration_registers_handle() {
    let source = r#"
robot R {
  secret api_key from "test-secret-value";

  behavior run() {
    let _ = api_key;
  }
}
"#;
    check(source).expect("secret declaration should type-check");
}

#[test]
fn trust_level_validation() {
    let source = r#"
robot R {
  trust certified;
  behavior run() {}
}
"#;
    check(source).expect("valid trust level should type-check");

    let bad = r#"
robot R {
  trust unknown_level;
  behavior run() {}
}
"#;
    assert!(check(bad).is_err());
}

#[test]
fn agent_comm_capability_runtime_enforcement() {
    let source = r#"
robot R {
  topic t: Velocity publish on "/t";

  agent NoPub {
    uses planner;
    tools [];
    goal "test";
    can [ subscribe(t), plan ];

    plan {
      publish t with velocity(linear: 0.0 m/s, angular: 0.0 rad/s);
    }
  }

  ai_model planner: LLM { provider: "mock"; model: "test"; temperature: 0.1; }

  behavior run() { NoPub.plan(); }
}
"#;
    let err = run(
        source,
        RunOptions {
            max_loop_iterations: 1,
            ..Default::default()
        },
    )
    .expect_err("agent without publish should fail");
    assert!(
        err.to_string().contains("lacks capability publish"),
        "got: {err}"
    );
}

#[test]
fn spanda_security_crate_api() {
    let mut ctx = SecurityContext::with_permissions(&PackagePermissions::from_capabilities([
        "audit.write",
        "identity.sign",
        "identity.verify",
    ]));
    ctx.set_identity(RobotIdentity::new("bot", "key").with_trust(TrustLevel::Trusted));
    ctx.register_secure_endpoint("/cmd", SecurePolicy::signed_trusted());

    let signed = ctx.sign_outbound("/cmd", "payload").unwrap().unwrap();
    assert!(signed.verify(ctx.identity.as_ref().unwrap()).unwrap());

    let mut caps = CapabilitySet::new();
    caps.grant("audit.write");
    let mut audit = spanda_audit::AuditRuntime::new("test", vec![]);
    ctx.audit_event(&mut audit, "test", "ok").unwrap();
}

#[test]
fn permissions_block_grants_package_caps() {
    let source = r#"
robot R {
  permissions [ audit.write, audit.read ];
  audit A { record robot.pose; }
  behavior run() { audit.record("e", "p"); }
}
"#;
    run(source, RunOptions::default()).expect("explicit permissions should allow audit");
}

#[test]
fn strict_permissions_blocks_auto_granted_audit() {
    let source = r#"
robot R {
  permissions [ audit.read ];
  audit A { record robot.pose; }
  behavior run() { audit.record("e", "p"); }
}
"#;
    let err = run(source, RunOptions::default())
        .expect_err("strict permissions should block audit.write");
    assert!(
        err.to_string().contains("capability denied"),
        "expected capability denial, got: {err}"
    );
}

#[test]
fn create_provenance_requires_sign_capability() {
    let source = r#"
robot R {
  permissions [ audit.write, identity.sign ];
  identity RobotIdentity { id: "r1"; public_key: "k1"; }
  audit A { record robot.pose; }

  behavior run() {
    let id = audit.record("evt", "data");
    audit.create_provenance("mission", id);
  }
}
"#;
    compile(source).expect("create_provenance should compile");
}
