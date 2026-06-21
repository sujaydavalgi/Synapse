//! Signed deploy artifact bundle tests.

use spanda_core::{
    build_deploy_bundle, build_deploy_plan, compile, sign_deploy_bundle, verify_deploy_bundle,
};

#[test]
fn signed_deploy_bundle_verifies() {
    let source = include_str!("../../../examples/robotics/ota_deployment.sd");
    let program = compile(source).expect("compile").program;
    let program_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/robotics/ota_deployment.sd"
    );
    let plan = build_deploy_plan(&program, program_path, "2.0.0");
    let mut bundle = build_deploy_bundle(&plan);
    let key = "fleet-ota-signing-key";
    sign_deploy_bundle(&mut bundle, key).expect("sign bundle");
    assert!(verify_deploy_bundle(&bundle, key));
    assert!(bundle.signature.is_some());
    assert!(bundle.public_key.is_some());
}

#[test]
fn tampered_deploy_bundle_rejected() {
    let source = include_str!("../../../examples/robotics/ota_deployment.sd");
    let program = compile(source).expect("compile").program;
    let program_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/robotics/ota_deployment.sd"
    );
    let plan = build_deploy_plan(&program, program_path, "2.0.0");
    let mut bundle = build_deploy_bundle(&plan);
    let key = "fleet-ota-signing-key";
    sign_deploy_bundle(&mut bundle, key).expect("sign bundle");
    bundle.version = "9.9.9".into();
    assert!(!verify_deploy_bundle(&bundle, key));
}
