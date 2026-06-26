//! Fleet-scale device pool performance gate (1000+ devices).

use spanda_config::{DeviceIdentityRecord, DeviceRegistry};
use serde_json::json;
use std::time::Instant;

fn synthetic_registry(count: usize) -> DeviceRegistry {
    let devices = (0..count)
        .map(|index| {
            serde_json::from_value::<DeviceIdentityRecord>(json!({
                "id": format!("dev-{index:04}"),
                "device_type": "sensor",
                "trust_level": "verified",
                "lifecycle_state": "active",
                "assigned_robot": format!("robot-{}", index % 20),
            }))
            .expect("device record")
        })
        .collect();
    DeviceRegistry { devices }
}

#[test]
fn device_pool_lists_1000_devices_under_budget() {
    let registry = synthetic_registry(1000);
    let started = Instant::now();
    let entries = registry.pool_entries();
    let summary = registry.pool_summary();
    let elapsed = started.elapsed();
    assert_eq!(entries.len(), 1000);
    assert_eq!(summary.total, 1000);
    assert!(
        elapsed.as_millis() < 500,
        "device pool list+summary too slow: {elapsed:?}"
    );
}
