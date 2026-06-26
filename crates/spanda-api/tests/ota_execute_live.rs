//! Live OTA execute against a spawned deploy agent.

use spanda_api::e3::ota_execute;
use spanda_ota::{
    agent_entry_for_port, deploy_target_key, register_agent, save_agent_registry, spawn_test_agent,
    DeployAgentRegistry,
};
use spanda_security::{RbacContext, Role};
use std::thread;
use std::time::Duration;

#[test]
fn ota_execute_live_rollout_updates_agent() {
    let target = deploy_target_key("RoverProgram", "JetsonOrin");
    let (port, _handle) = spawn_test_agent(&target, None).expect("spawn test agent");
    thread::sleep(Duration::from_millis(100));

    let agents_path = std::env::temp_dir().join(format!(
        "spanda-ota-exec-test-{}.json",
        std::process::id()
    ));
    let mut registry = DeployAgentRegistry::default();
    let entry = agent_entry_for_port(&target, port, None);
    register_agent(&mut registry, target.clone(), entry.url, None).expect("register agent");
    save_agent_registry(&agents_path, &registry).expect("save registry");

    std::env::set_var(
        "SPANDA_DEPLOY_AGENTS",
        agents_path.to_string_lossy().to_string(),
    );
    let ctx = RbacContext {
        key_id: "ota-live-test".into(),
        role: Role::Administrator,
    };
    let body = serde_json::json!({
        "strategy": "all",
        "version": "2.0.0",
        "dry_run": false,
        "assignments": [{
            "robot_name": "RoverProgram",
            "hardware": "JetsonOrin"
        }],
    });
    let response = ota_execute(&body.to_string(), Some(&ctx));
    assert_eq!(response.status, 200, "ota execute failed: {}", response.body);
    assert!(response.body.contains("\"executed\":true"));
    assert!(response.body.contains("\"success\":true"));
    let _ = std::fs::remove_file(agents_path);
}
