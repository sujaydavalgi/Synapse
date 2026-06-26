//! Automated failover drill — redundant chain walk-through and recovery actions.

use spanda_config::{
    failover_chains, next_failover_device, recovery_failover_actions, DeviceIdentityRecord,
    DeviceRegistry,
};

fn drill_registry() -> DeviceRegistry {
    DeviceRegistry {
        devices: vec![
            DeviceIdentityRecord {
                id: "lidar-primary".into(),
                device_type: "Lidar".into(),
                logical_name: Some("front-lidar".into()),
                redundant_group: Some("lidar-pair".into()),
                failover_priority: Some(1),
                lifecycle_state: Some("failed".into()),
                ..Default::default()
            },
            DeviceIdentityRecord {
                id: "lidar-backup".into(),
                device_type: "Lidar".into(),
                logical_name: Some("front-lidar-backup".into()),
                redundant_group: Some("lidar-pair".into()),
                failover_priority: Some(2),
                lifecycle_state: Some("active".into()),
                ..Default::default()
            },
            DeviceIdentityRecord {
                id: "lidar-tertiary".into(),
                device_type: "Lidar".into(),
                redundant_group: Some("lidar-pair".into()),
                failover_priority: Some(3),
                lifecycle_state: Some("active".into()),
                ..Default::default()
            },
        ],
    }
}

#[test]
fn failover_drill_selects_next_active_member() {
    let registry = drill_registry();
    let chains = failover_chains(&registry);
    assert_eq!(chains.len(), 1);
    assert_eq!(chains[0].members.len(), 3);

    let next = next_failover_device(&registry, "lidar-primary").expect("backup device");
    assert_eq!(next.device_id, "lidar-backup");
    assert_eq!(next.priority, 2);
}

#[test]
fn failover_drill_skips_quarantined_backup() {
    let mut registry = drill_registry();
    registry
        .devices
        .iter_mut()
        .find(|device| device.id == "lidar-backup")
        .expect("backup")
        .lifecycle_state = Some("quarantined".into());

    let next = next_failover_device(&registry, "lidar-primary").expect("tertiary");
    assert_eq!(next.device_id, "lidar-tertiary");
}

#[test]
fn failover_drill_recovery_actions_include_remap() {
    let registry = drill_registry();
    let actions = recovery_failover_actions(&registry, "lidar-primary");
    assert!(actions.iter().any(|action| action.contains("switch_redundant_hardware")));
    assert!(actions.iter().any(|action| action.contains("remap logical")));
    assert!(actions.iter().any(|action| action.contains("lidar-backup")));
}

#[test]
fn failover_drill_no_backup_when_chain_exhausted() {
    let registry = DeviceRegistry {
        devices: vec![DeviceIdentityRecord {
            id: "gps-only".into(),
            device_type: "GPS".into(),
            redundant_group: Some("gps".into()),
            failover_priority: Some(1),
            lifecycle_state: Some("failed".into()),
            ..Default::default()
        }],
    };
    assert!(next_failover_device(&registry, "gps-only").is_none());
}
