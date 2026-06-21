//! Remote OTA deploy agent integration tests.

use spanda_core::{
    agent_entry_for_port, agent_health, agent_rollout, agent_status, build_deploy_bundle,
    build_deploy_plan, compile, default_agents_path, deploy_target_key, execute_remote_rollout,
    load_agent_registry, register_agent, save_agent_registry, sign_deploy_bundle,
    spawn_test_agent, DeployAgentRegistry, RolloutOptions, RolloutStrategy,
};
use std::thread;
use std::time::Duration;

#[test]
fn remote_rollout_updates_agent_state() {
    let target = deploy_target_key("RoverProgram", "JetsonOrin");
    let (port, _handle) = spawn_test_agent(&target, None).expect("spawn test agent");
    thread::sleep(Duration::from_millis(50));

    let entry = agent_entry_for_port(&target, port, None);
    assert!(agent_health(&entry).expect("health check"));
    let source = include_str!("../../../examples/robotics/ota_deployment.sd");
    let program = compile(source).expect("compile").program;
    let program_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/robotics/ota_deployment.sd"
    );
    let plan = build_deploy_plan(&program, program_path, "2.0.0");
    assert!(plan.program_hash.is_some(), "deploy plan should include program hash");
    let mut bundle = build_deploy_bundle(&plan);
    sign_deploy_bundle(&mut bundle, "test-signing-key").expect("sign bundle");
    let mut registry = DeployAgentRegistry::default();
    register_agent(
        &mut registry,
        target.clone(),
        entry.url.clone(),
        None,
    )
    .expect("register agent");

    let result = execute_remote_rollout(
        &plan,
        &RolloutOptions {
            strategy: RolloutStrategy::All,
            version: "2.0.0".into(),
            dry_run: false,
            ..Default::default()
        },
        &registry,
        &bundle,
    );
    assert!(result.success, "remote rollout failed: {:?}", result.steps);
    let status = agent_status(&entry).expect("agent status");
    assert_eq!(status.current_version, "2.0.0");

    bundle.version = "2.1.0".into();
    sign_deploy_bundle(&mut bundle, "test-signing-key").expect("resign bundle");
    let rollout = agent_rollout(&entry, &bundle).expect("agent rollout");
    assert!(rollout.ok);
    assert_eq!(rollout.version, "2.1.0");
}

#[test]
fn agent_registry_persists_entries() {
    let path = std::env::temp_dir().join("spanda-deploy-agents-test.json");
    let mut registry = DeployAgentRegistry::default();
    register_agent(
        &mut registry,
        "Rover@Jetson".into(),
        "http://127.0.0.1:8765".into(),
        Some("secret".into()),
    )
    .expect("register");
    save_agent_registry(&path, &registry).expect("save");
    let loaded = load_agent_registry(&path);
    assert_eq!(loaded.agents.len(), 1);
    assert_eq!(loaded.agents[0].target, "Rover@Jetson");
    let _ = std::fs::remove_file(path);
    let _ = default_agents_path();
}
