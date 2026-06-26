//! Device pool reporting — inventory, trust, calibration, and readiness impact.
//!
use crate::device_health::{readiness_impact, DeviceHealthStatus};
use crate::device_identity::{detect_identity_anomalies, DeviceRegistry, IdentityAnomaly};
use crate::device_pool::DevicePoolSummary;
use crate::device_quarantine::evaluate_quarantine_policy;
use crate::mapping::LogicalPhysicalMap;
use serde::{Deserialize, Serialize};

/// Full device operations report bundle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceReportBundle {
    pub inventory: DeviceInventoryReport,
    pub assignments: AssignmentReport,
    pub trust: DeviceTrustReport,
    pub capabilities: CapabilityCoverageReport,
    pub calibration: CalibrationReport,
    pub readiness_impact: crate::device_health::ReadinessImpactReport,
    pub anomalies: Vec<IdentityAnomaly>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceInventoryReport {
    pub summary: DevicePoolSummary,
    pub devices: Vec<DeviceInventoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceInventoryEntry {
    pub id: String,
    pub device_type: String,
    pub lifecycle_state: String,
    pub provider: Option<String>,
    pub assigned_robot: Option<String>,
    pub logical_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignmentReport {
    pub assigned: Vec<AssignmentEntry>,
    pub unassigned: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignmentEntry {
    pub device_id: String,
    pub robot_id: String,
    pub logical_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceTrustReport {
    pub trusted: Vec<String>,
    pub unverified: Vec<String>,
    pub quarantined: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityCoverageReport {
    pub mapped_sensors: usize,
    pub mapped_actuators: usize,
    pub devices_without_capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationReport {
    pub valid: Vec<String>,
    pub expired: Vec<String>,
    pub unknown: Vec<String>,
}

/// Generate the full device report bundle.
pub fn generate_device_reports(
    registry: &DeviceRegistry,
    map: &LogicalPhysicalMap,
    now_ms: f64,
) -> DeviceReportBundle {
    let summary = registry.pool_summary();
    let devices: Vec<DeviceInventoryEntry> = registry
        .devices
        .iter()
        .map(|d| DeviceInventoryEntry {
            id: d.id.clone(),
            device_type: d.device_type.clone(),
            lifecycle_state: d
                .lifecycle_state
                .clone()
                .unwrap_or_else(|| "discovered".into()),
            provider: d.provider.clone(),
            assigned_robot: d.assigned_robot.clone().or(d.robot_id.clone()),
            logical_name: d.logical_name.clone(),
        })
        .collect();
    let mut assigned = Vec::new();
    let mut unassigned = Vec::new();
    for device in &registry.devices {
        if let Some(robot) = device.assigned_robot.clone().or(device.robot_id.clone()) {
            assigned.push(AssignmentEntry {
                device_id: device.id.clone(),
                robot_id: robot,
                logical_name: device.logical_name.clone(),
            });
        } else {
            unassigned.push(device.id.clone());
        }
    }
    let mut trusted = Vec::new();
    let mut unverified = Vec::new();
    let mut quarantined = Vec::new();
    for device in &registry.devices {
        let policy = evaluate_quarantine_policy(device);
        if policy.quarantined {
            quarantined.push(device.id.clone());
        } else if device.trust_level_enum().is_operational() {
            trusted.push(device.id.clone());
        } else {
            unverified.push(device.id.clone());
        }
    }
    let devices_without_capabilities: Vec<String> = registry
        .devices
        .iter()
        .filter(|d| d.capabilities.is_empty())
        .map(|d| d.id.clone())
        .collect();
    let mut cal_valid = Vec::new();
    let mut cal_expired = Vec::new();
    let mut cal_unknown = Vec::new();
    for device in &registry.devices {
        let health: DeviceHealthStatus =
            crate::device_health::evaluate_device_readiness(device, now_ms);
        if health.calibration_expired {
            cal_expired.push(device.id.clone());
        } else if health.calibration_status == crate::device_health::CalibrationStatus::Valid {
            cal_valid.push(device.id.clone());
        } else {
            cal_unknown.push(device.id.clone());
        }
    }
    DeviceReportBundle {
        inventory: DeviceInventoryReport { summary, devices },
        assignments: AssignmentReport {
            assigned,
            unassigned,
        },
        trust: DeviceTrustReport {
            trusted,
            unverified,
            quarantined,
        },
        capabilities: CapabilityCoverageReport {
            mapped_sensors: map.sensors.len(),
            mapped_actuators: map.actuators.len(),
            devices_without_capabilities,
        },
        calibration: CalibrationReport {
            valid: cal_valid,
            expired: cal_expired,
            unknown: cal_unknown,
        },
        readiness_impact: readiness_impact(registry, now_ms),
        anomalies: detect_identity_anomalies(registry),
    }
}
