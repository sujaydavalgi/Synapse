//! Device pool operations — assign, unassign, quarantine, retire, and mapping updates.
//!
use crate::device_identity::{DeviceIdentityRecord, DeviceRegistry};
use crate::device_pool::DeviceLifecycleState;
use crate::mapping::LogicalPhysicalMap;
use serde::{Deserialize, Serialize};

/// Options for assigning a device to a robot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AssignDeviceOptions {
    pub robot_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logical_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redundant_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failover_priority: Option<u32>,
}

/// Result of a device pool mutation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceOperationResult {
    pub ok: bool,
    pub device_id: String,
    pub lifecycle_state: String,
    pub message: String,
}

impl DeviceRegistry {
    /// Assign a device to a robot and optional logical entity.
    pub fn assign_device(
        &mut self,
        device_id: &str,
        options: &AssignDeviceOptions,
    ) -> Result<DeviceOperationResult, String> {
        let device = self
            .devices
            .iter_mut()
            .find(|d| d.id == device_id)
            .ok_or_else(|| format!("device '{device_id}' not found"))?;
        let current = device
            .lifecycle_state
            .as_deref()
            .map(DeviceLifecycleState::parse)
            .unwrap_or(DeviceLifecycleState::Discovered);
        if matches!(
            current,
            DeviceLifecycleState::Quarantined | DeviceLifecycleState::Retired
        ) {
            return Err(format!(
                "cannot assign device in state '{}'",
                current.as_str()
            ));
        }
        device.assigned_robot = Some(options.robot_id.clone());
        device.robot_id = Some(options.robot_id.clone());
        if let Some(ref logical) = options.logical_name {
            device.logical_name = Some(logical.clone());
        }
        if let Some(ref group) = options.redundant_group {
            device.redundant_group = Some(group.clone());
        }
        if let Some(priority) = options.failover_priority {
            device.failover_priority = Some(priority);
        }
        let next = if current == DeviceLifecycleState::Verified
            || current == DeviceLifecycleState::Discovered
        {
            DeviceLifecycleState::Assigned
        } else {
            current
        };
        device.lifecycle_state = Some(next.as_str().to_string());
        Ok(DeviceOperationResult {
            ok: true,
            device_id: device_id.to_string(),
            lifecycle_state: next.as_str().to_string(),
            message: format!("assigned to robot '{}'", options.robot_id),
        })
    }

    /// Remove robot assignment from a device.
    pub fn unassign_device(&mut self, device_id: &str) -> Result<DeviceOperationResult, String> {
        let device = self
            .devices
            .iter_mut()
            .find(|d| d.id == device_id)
            .ok_or_else(|| format!("device '{device_id}' not found"))?;
        device.assigned_robot = None;
        device.robot_id = None;
        let next = DeviceLifecycleState::Verified;
        device.lifecycle_state = Some(next.as_str().to_string());
        Ok(DeviceOperationResult {
            ok: true,
            device_id: device_id.to_string(),
            lifecycle_state: next.as_str().to_string(),
            message: "assignment removed".into(),
        })
    }

    /// Move a device into quarantine.
    pub fn quarantine_device(&mut self, device_id: &str) -> Result<DeviceOperationResult, String> {
        self.set_lifecycle(device_id, DeviceLifecycleState::Quarantined)?;
        Ok(DeviceOperationResult {
            ok: true,
            device_id: device_id.to_string(),
            lifecycle_state: DeviceLifecycleState::Quarantined.as_str().to_string(),
            message: "device quarantined — operator approval required to trust".into(),
        })
    }

    /// Retire a device from the active pool.
    pub fn retire_device(&mut self, device_id: &str) -> Result<DeviceOperationResult, String> {
        self.set_lifecycle(device_id, DeviceLifecycleState::Retired)?;
        Ok(DeviceOperationResult {
            ok: true,
            device_id: device_id.to_string(),
            lifecycle_state: DeviceLifecycleState::Retired.as_str().to_string(),
            message: "device retired".into(),
        })
    }

    /// Trust a device after operator approval (quarantine → verified).
    pub fn trust_device(&mut self, device_id: &str) -> Result<DeviceOperationResult, String> {
        let device = self
            .devices
            .iter_mut()
            .find(|d| d.id == device_id)
            .ok_or_else(|| format!("device '{device_id}' not found"))?;
        device.trust_level = Some("verified".into());
        self.set_lifecycle(device_id, DeviceLifecycleState::Verified)?;
        Ok(DeviceOperationResult {
            ok: true,
            device_id: device_id.to_string(),
            lifecycle_state: DeviceLifecycleState::Verified.as_str().to_string(),
            message: "device trusted by operator".into(),
        })
    }

    /// Register a newly discovered device in the pool.
    pub fn register_discovered(&mut self, record: DeviceIdentityRecord) -> DeviceOperationResult {
        let id = record.id.clone();
        if let Some(existing) = self.devices.iter_mut().find(|d| d.id == id) {
            existing.last_seen_ms = record.last_seen_ms;
            if existing.lifecycle_state.is_none() {
                existing.lifecycle_state =
                    Some(DeviceLifecycleState::Discovered.as_str().to_string());
            }
            return DeviceOperationResult {
                ok: true,
                device_id: id,
                lifecycle_state: existing
                    .lifecycle_state
                    .clone()
                    .unwrap_or_else(|| DeviceLifecycleState::Discovered.as_str().to_string()),
                message: "device refreshed".into(),
            };
        }
        let mut record = record;
        record.lifecycle_state = Some(DeviceLifecycleState::Discovered.as_str().to_string());
        if record.trust_level.is_none() {
            record.trust_level = Some("unknown".into());
        }
        self.devices.push(record);
        DeviceOperationResult {
            ok: true,
            device_id: id,
            lifecycle_state: DeviceLifecycleState::Discovered.as_str().to_string(),
            message: "device registered".into(),
        }
    }
}

/// Export logical-to-physical mapping as JSON-friendly structure.
pub fn export_device_mapping_json(
    registry: &DeviceRegistry,
    map: &LogicalPhysicalMap,
) -> serde_json::Value {
    let redundancy: Vec<serde_json::Value> = crate::device_identity::redundant_groups(registry)
        .into_iter()
        .map(|(group, members)| {
            let mut sorted = members;
            sorted.sort_by_key(|d| d.failover_priority.unwrap_or(u32::MAX));
            serde_json::json!({
                "group": group,
                "members": sorted.iter().map(|d| serde_json::json!({
                    "device_id": d.id,
                    "failover_priority": d.failover_priority,
                    "logical_name": d.logical_name,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    serde_json::json!({
        "sensors": map.sensors,
        "actuators": map.actuators,
        "robots": map.robots,
        "redundancy": redundancy,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device_identity::DeviceIdentityRecord;

    #[test]
    fn assign_updates_robot_and_lifecycle() {
        let mut registry = DeviceRegistry {
            devices: vec![DeviceIdentityRecord {
                id: "gps-1".into(),
                device_type: "gps".into(),
                lifecycle_state: Some("verified".into()),
                trust_level: Some("verified".into()),
                ..Default::default()
            }],
        };
        let result = registry
            .assign_device(
                "gps-1",
                &AssignDeviceOptions {
                    robot_id: "rover-1".into(),
                    logical_name: Some("gps".into()),
                    ..Default::default()
                },
            )
            .expect("assign");
        assert!(result.ok);
        assert_eq!(
            registry.devices[0].assigned_robot.as_deref(),
            Some("rover-1")
        );
        assert_eq!(registry.devices[0].logical_name.as_deref(), Some("gps"));
    }

    #[test]
    fn cannot_assign_quarantined_device() {
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
}
