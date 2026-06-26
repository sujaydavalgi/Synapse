//! Device pool, quarantine, and provisioning integration tests.

use spanda_config::{
    detect_identity_anomalies, evaluate_quarantine_policy, run_provision_workflow,
    AssignDeviceOptions, DeviceIdentityRecord, DeviceLifecycleState, DeviceRegistry,
};

fn sample_registry() -> DeviceRegistry {
    DeviceRegistry {
        devices: vec![
            DeviceIdentityRecord {
                id: "gps-001".into(),
                device_type: "GPS".into(),
                logical_name: Some("gps".into()),
                serial: Some("GPS-001".into()),
                firmware_version: Some("2.1.0".into()),
                trust_level: Some("verified".into()),
                lifecycle_state: Some("verified".into()),
                robot_id: Some("rover-001".into()),
                capabilities: vec!["read_location".into()],
                ..Default::default()
            },
            DeviceIdentityRecord {
                id: "gps-002".into(),
                device_type: "GPS".into(),
                logical_name: Some("gps".into()),
                redundant_group: Some("gps".into()),
                failover_priority: Some(2),
                serial: Some("GPS-002".into()),
                firmware_version: Some("2.1.0".into()),
                trust_level: Some("verified".into()),
                lifecycle_state: Some("active".into()),
                capabilities: vec!["read_location".into()],
                ..Default::default()
            },
        ],
    }
}

#[test]
fn detects_duplicate_serial() {
    let registry = DeviceRegistry {
        devices: vec![
            DeviceIdentityRecord {
                id: "a".into(),
                device_type: "sensor".into(),
                serial: Some("SAME".into()),
                ..Default::default()
            },
            DeviceIdentityRecord {
                id: "b".into(),
                device_type: "sensor".into(),
                serial: Some("SAME".into()),
                ..Default::default()
            },
        ],
    };
    let anomalies = detect_identity_anomalies(&registry);
    assert!(anomalies
        .iter()
        .any(|a| a.anomaly_type == "duplicate_serial"));
}

#[test]
fn quarantine_blocks_actuator_control() {
    let device = DeviceIdentityRecord {
        id: "drive-1".into(),
        device_type: "DifferentialDrive".into(),
        lifecycle_state: Some("quarantined".into()),
        endpoint_url: Some("http://10.0.0.5".into()),
        capabilities: vec!["move".into(), "stop".into()],
        ..Default::default()
    };
    let policy = evaluate_quarantine_policy(&device);
    assert!(!policy.can_control_actuators);
}

#[test]
fn assign_rejects_quarantined_device() {
    let mut registry = DeviceRegistry {
        devices: vec![DeviceIdentityRecord {
            id: "cam-1".into(),
            device_type: "camera".into(),
            lifecycle_state: Some("quarantined".into()),
            ..Default::default()
        }],
    };
    let err = registry
        .assign_device(
            "cam-1",
            &AssignDeviceOptions {
                robot_id: "rover-1".into(),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(err.contains("quarantined"));
}

#[test]
fn provision_passes_for_verified_device() {
    let registry = sample_registry();
    let report = run_provision_workflow("gps-001", &registry, Some("rover-001"));
    assert!(report.ready);
}

#[test]
fn lifecycle_active_parses_from_healthy_alias() {
    assert_eq!(
        DeviceLifecycleState::parse("healthy"),
        DeviceLifecycleState::Active
    );
}

#[test]
fn pool_summary_counts_active_devices() {
    let registry = sample_registry();
    let summary = registry.pool_summary();
    assert!(summary.active >= 1);
}
