//! Device Pool lifecycle states and inventory helpers for enterprise operations.
//!
use crate::device_identity::{DeviceIdentityRecord, DeviceRegistry};
use serde::{Deserialize, Serialize};

/// Lifecycle state for a device in the central inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DeviceLifecycleState {
    #[default]
    Discovered,
    Quarantined,
    Verified,
    Assigned,
    Active,
    Healthy,
    Degraded,
    Offline,
    Failed,
    Retired,
}

impl DeviceLifecycleState {
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "quarantined" => Self::Quarantined,
            "verified" => Self::Verified,
            "assigned" => Self::Assigned,
            "active" => Self::Active,
            "healthy" => Self::Active,
            "degraded" => Self::Degraded,
            "offline" => Self::Offline,
            "failed" => Self::Failed,
            "retired" => Self::Retired,
            _ => Self::Discovered,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Discovered => "discovered",
            Self::Quarantined => "quarantined",
            Self::Verified => "verified",
            Self::Assigned => "assigned",
            Self::Active => "active",
            Self::Healthy => "active",
            Self::Degraded => "degraded",
            Self::Offline => "offline",
            Self::Failed => "failed",
            Self::Retired => "retired",
        }
    }

    /// Whether a transition from `from` to `to` is allowed without operator override.
    pub fn can_transition(from: Self, to: Self) -> bool {
        if from == to {
            return true;
        }
        use DeviceLifecycleState::*;
        matches!(
            (from, to),
            (Discovered, Quarantined)
                | (Discovered, Verified)
                | (Quarantined, Discovered)
                | (Quarantined, Verified)
                | (Verified, Assigned)
                | (Verified, Quarantined)
                | (Assigned, Active)
                | (Assigned, Offline)
                | (Active, Degraded)
                | (Active, Offline)
                | (Active, Failed)
                | (Degraded, Active)
                | (Degraded, Offline)
                | (Degraded, Failed)
                | (Offline, Active)
                | (Offline, Degraded)
                | (Failed, Quarantined)
                | (Failed, Retired)
                | (_, Retired)
        )
    }
}

/// Pool view of a device with lifecycle metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DevicePoolEntry {
    pub id: String,
    pub device_type: String,
    pub lifecycle_state: DeviceLifecycleState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_robot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provisioning_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logical_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_level: Option<String>,
}

impl DevicePoolEntry {
    pub fn from_record(record: &DeviceIdentityRecord) -> Self {
        let lifecycle_state = record
            .lifecycle_state
            .as_deref()
            .map(DeviceLifecycleState::parse)
            .unwrap_or(DeviceLifecycleState::Discovered);
        Self {
            id: record.id.clone(),
            device_type: record.device_type.clone(),
            lifecycle_state,
            assigned_robot: record.assigned_robot.clone().or(record.robot_id.clone()),
            last_seen_ms: record.last_seen_ms,
            provisioning_id: record.provisioning_id.clone(),
            logical_name: record.logical_name.clone(),
            trust_level: record.trust_level.clone(),
        }
    }
}

/// Summary counts for the device pool dashboard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DevicePoolSummary {
    pub total: u32,
    pub discovered: u32,
    pub quarantined: u32,
    pub verified: u32,
    pub assigned: u32,
    pub active: u32,
    pub healthy: u32,
    pub degraded: u32,
    pub offline: u32,
    pub failed: u32,
    pub retired: u32,
}

impl DeviceRegistry {
    pub fn pool_entries(&self) -> Vec<DevicePoolEntry> {
        self.devices
            .iter()
            .map(DevicePoolEntry::from_record)
            .collect()
    }

    pub fn pool_summary(&self) -> DevicePoolSummary {
        let mut summary = DevicePoolSummary::default();
        for entry in self.pool_entries() {
            summary.total += 1;
            match entry.lifecycle_state {
                DeviceLifecycleState::Discovered => summary.discovered += 1,
                DeviceLifecycleState::Quarantined => summary.quarantined += 1,
                DeviceLifecycleState::Verified => summary.verified += 1,
                DeviceLifecycleState::Assigned => summary.assigned += 1,
                DeviceLifecycleState::Active | DeviceLifecycleState::Healthy => {
                    summary.active += 1;
                    summary.healthy += 1;
                }
                DeviceLifecycleState::Degraded => summary.degraded += 1,
                DeviceLifecycleState::Offline => summary.offline += 1,
                DeviceLifecycleState::Failed => summary.failed += 1,
                DeviceLifecycleState::Retired => summary.retired += 1,
            }
        }
        summary
    }

    pub fn set_lifecycle(
        &mut self,
        device_id: &str,
        state: DeviceLifecycleState,
    ) -> Result<(), String> {
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
        if !DeviceLifecycleState::can_transition(current, state) {
            return Err(format!(
                "lifecycle transition {current:?} -> {state:?} not allowed for '{device_id}'"
            ));
        }
        device.lifecycle_state = Some(state.as_str().to_string());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_transition_allows_quarantine_to_verified() {
        assert!(DeviceLifecycleState::can_transition(
            DeviceLifecycleState::Quarantined,
            DeviceLifecycleState::Verified
        ));
    }

    #[test]
    fn pool_summary_counts_states() {
        let mut registry = DeviceRegistry::default();
        registry.devices.push(DeviceIdentityRecord {
            id: "cam-1".into(),
            device_type: "camera".into(),
            lifecycle_state: Some("healthy".into()),
            ..Default::default()
        });
        registry.devices.push(DeviceIdentityRecord {
            id: "lidar-1".into(),
            device_type: "lidar".into(),
            lifecycle_state: Some("discovered".into()),
            ..Default::default()
        });
        let summary = registry.pool_summary();
        assert_eq!(summary.total, 2);
        assert_eq!(summary.healthy, 1);
        assert_eq!(summary.active, 1);
        assert_eq!(summary.discovered, 1);
    }
}
