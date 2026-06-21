//! OTA deploy service integration tests.

use spanda_core::{
    apply_rollout, build_deploy_plan, plan_rollout, RolloutOptions, RolloutStrategy, check,
    compile,
};

#[test]
fn ota_canary_rollout_from_example() {
    let source = include_str!("../../../examples/robotics/ota_deployment.sd");
    check(source).expect("ota example should type-check");
    let program = compile(source).expect("compile").program;
    let plan = build_deploy_plan(&program, "ota_deployment.sd", "1.0.0");
    assert!(!plan.assignments.is_empty());
    let result = plan_rollout(
        &plan,
        &RolloutOptions {
            strategy: RolloutStrategy::All,
            version: "1.0.0".into(),
            dry_run: false,
            ..Default::default()
        },
    );
    assert!(result.success);
    let canary = plan_rollout(
        &plan,
        &RolloutOptions {
            strategy: RolloutStrategy::Canary,
            canary_percent: 50,
            version: "1.1.0".into(),
            ..Default::default()
        },
    );
    assert!(canary.success);
}

#[test]
fn ota_apply_rollout_updates_state() {
    let source = include_str!("../../../examples/robotics/ota_deployment.sd");
    let program = compile(source).expect("compile").program;
    let plan = build_deploy_plan(&program, "ota.sd", "2.0.0");
    let mut state = spanda_core::DeployState::default();
    let rollout = plan_rollout(
        &plan,
        &RolloutOptions {
            strategy: RolloutStrategy::All,
            version: "2.0.0".into(),
            ..Default::default()
        },
    );
    apply_rollout(&mut state, &rollout);
    assert!(!state.current_version.is_empty());
}
