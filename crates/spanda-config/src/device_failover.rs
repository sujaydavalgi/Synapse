//! Redundant device failover chains for recovery and continuity integration.
//!
use crate::device_identity::{redundant_groups, DeviceIdentityRecord, DeviceRegistry};
use serde::{Deserialize, Serialize};

/// Ordered failover chain for a redundant device group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailoverChain {
    pub group: String,
    pub logical_name: Option<String>,
    pub members: Vec<FailoverMember>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailoverMember {
    pub device_id: String,
    pub priority: u32,
    pub device_type: String,
    pub lifecycle_state: Option<String>,
}

/// Build failover chains from registry redundant groups sorted by priority.
pub fn failover_chains(registry: &DeviceRegistry) -> Vec<FailoverChain> {
    redundant_groups(registry)
        .into_iter()
        .map(|(group, members)| {
            let mut sorted: Vec<&DeviceIdentityRecord> = members;
            sorted.sort_by_key(|d| d.failover_priority.unwrap_or(u32::MAX));
            let logical_name = sorted.first().and_then(|d| d.logical_name.clone());
            FailoverChain {
                group: group.clone(),
                logical_name,
                members: sorted
                    .into_iter()
                    .map(|d| FailoverMember {
                        device_id: d.id.clone(),
                        priority: d.failover_priority.unwrap_or(u32::MAX),
                        device_type: d.device_type.clone(),
                        lifecycle_state: d.lifecycle_state.clone(),
                    })
                    .collect(),
            }
        })
        .collect()
}

/// Suggest the next device in a failover chain after a failure.
pub fn next_failover_device(
    registry: &DeviceRegistry,
    failed_device_id: &str,
) -> Option<FailoverMember> {
    for chain in failover_chains(registry) {
        let idx = chain
            .members
            .iter()
            .position(|m| m.device_id == failed_device_id)?;
        for member in chain.members.iter().skip(idx + 1) {
            if let Some(device) = registry.get(&member.device_id) {
                let lifecycle = device.lifecycle_state.as_deref().unwrap_or("discovered");
                if lifecycle != "failed" && lifecycle != "quarantined" && lifecycle != "retired" {
                    return Some(member.clone());
                }
            }
        }
    }
    None
}

/// Recovery actions suggested from device failover configuration.
pub fn recovery_failover_actions(registry: &DeviceRegistry, failed_device_id: &str) -> Vec<String> {
    let mut actions = Vec::new();
    if let Some(next) = next_failover_device(registry, failed_device_id) {
        actions.push(format!("switch_redundant_hardware {}", next.device_id));
        if let Some(ref logical) = registry
            .get(failed_device_id)
            .and_then(|d| d.logical_name.clone())
        {
            actions.push(format!("remap logical {logical} -> {}", next.device_id));
        }
    }
    actions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device_identity::DeviceIdentityRecord;

    #[test]
    fn selects_next_failover_by_priority() {
        let registry = DeviceRegistry {
            devices: vec![
                DeviceIdentityRecord {
                    id: "gps-001".into(),
                    device_type: "GPS".into(),
                    redundant_group: Some("gps".into()),
                    failover_priority: Some(1),
                    lifecycle_state: Some("failed".into()),
                    ..Default::default()
                },
                DeviceIdentityRecord {
                    id: "gps-002".into(),
                    device_type: "GPS".into(),
                    redundant_group: Some("gps".into()),
                    failover_priority: Some(2),
                    lifecycle_state: Some("active".into()),
                    ..Default::default()
                },
            ],
        };
        let next = next_failover_device(&registry, "gps-001").expect("backup");
        assert_eq!(next.device_id, "gps-002");
    }
}
