//! Human health opt-in gate tests.
use spanda_security::{HumanHealthGate, HumanHealthSettings};

#[test]
fn health_gate_requires_config_and_env() {
    std::env::remove_var("SPANDA_HUMAN_HEALTH_ENABLED");
    let gate = HumanHealthGate::resolve(&HumanHealthSettings {
        enabled: true,
        require_consent: true,
        audit_health_reads: true,
        retention_days: 30,
    });
    assert!(!gate.active);
    assert!(!gate.allows_health_telemetry_read());

    std::env::set_var("SPANDA_HUMAN_HEALTH_ENABLED", "1");
    let gate = HumanHealthGate::resolve(&HumanHealthSettings {
        enabled: true,
        ..HumanHealthSettings::default()
    });
    assert!(gate.active);
    assert!(gate.allows_health_telemetry_read());
    std::env::remove_var("SPANDA_HUMAN_HEALTH_ENABLED");
}
