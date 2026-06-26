//! Device provisioning workflow — discover through ready gates.
//!
use crate::device_health::{evaluate_device_readiness, CalibrationStatus};
use crate::device_identity::{
    detect_identity_anomalies, DeviceIdentityRecord, DeviceRegistry, TrustLevel,
};
use crate::device_pool::DeviceLifecycleState;
use serde::{Deserialize, Serialize};

/// A single gate in the provisioning pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvisionStep {
    Discover,
    VerifyIdentity,
    TrustValidation,
    FirmwareValidation,
    HealthValidation,
    CapabilityValidation,
    Assign,
    Ready,
}

impl ProvisionStep {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Discover => "discover",
            Self::VerifyIdentity => "verify_identity",
            Self::TrustValidation => "trust_validation",
            Self::FirmwareValidation => "firmware_validation",
            Self::HealthValidation => "health_validation",
            Self::CapabilityValidation => "capability_validation",
            Self::Assign => "assign",
            Self::Ready => "ready",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Discover,
            Self::VerifyIdentity,
            Self::TrustValidation,
            Self::FirmwareValidation,
            Self::HealthValidation,
            Self::CapabilityValidation,
            Self::Assign,
            Self::Ready,
        ]
    }
}

/// Result of one provisioning gate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProvisionStepResult {
    pub step: String,
    pub passed: bool,
    pub message: String,
}

/// Full provisioning report for a device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProvisionReport {
    pub device_id: String,
    pub ready: bool,
    pub steps: Vec<ProvisionStepResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_robot: Option<String>,
}

/// Run the discover → ready provisioning workflow for one device.
pub fn run_provision_workflow(
    device_id: &str,
    registry: &DeviceRegistry,
    assign_robot: Option<&str>,
) -> ProvisionReport {
    let mut steps = Vec::new();
    let device = registry.get(device_id);
    let mut ready = true;

    for step in ProvisionStep::all() {
        let result = evaluate_step(*step, device, assign_robot, registry);
        if !result.passed {
            ready = false;
        }
        steps.push(result);
    }

    ProvisionReport {
        device_id: device_id.to_string(),
        ready,
        steps,
        assigned_robot: assign_robot.map(str::to_string),
    }
}

fn evaluate_step(
    step: ProvisionStep,
    device: Option<&DeviceIdentityRecord>,
    assign_robot: Option<&str>,
    registry: &DeviceRegistry,
) -> ProvisionStepResult {
    let step_name = step.as_str().to_string();
    match step {
        ProvisionStep::Discover => {
            if let Some(d) = device {
                ok(step_name, format!("device '{}' discovered", d.id))
            } else {
                fail(step_name, "device not found in registry".into())
            }
        }
        ProvisionStep::VerifyIdentity => {
            let Some(d) = device else {
                return fail(step_name, "device missing".into());
            };
            if d.id.is_empty() {
                return fail(step_name, "device id is empty".into());
            }
            let dup_ip = registry
                .devices
                .iter()
                .filter(|x| x.ip_address == d.ip_address && x.ip_address.is_some())
                .count()
                > 1;
            if dup_ip {
                return fail(step_name, "duplicate IP in registry".into());
            }
            let anomalies = detect_identity_anomalies(registry)
                .into_iter()
                .filter(|a| a.device_id == d.id)
                .collect::<Vec<_>>();
            if anomalies
                .iter()
                .any(|a| a.anomaly_type == "duplicate_serial" || a.anomaly_type == "duplicate_mac")
            {
                return fail(step_name, "duplicate identity detected".into());
            }
            ok(step_name, "identity fields valid".into())
        }
        ProvisionStep::TrustValidation => {
            let Some(d) = device else {
                return fail(step_name, "device missing".into());
            };
            match d.trust_level_enum() {
                TrustLevel::Trusted | TrustLevel::Verified => {
                    ok(step_name, format!("trust level {:?}", d.trust_level_enum()))
                }
                TrustLevel::Restricted => fail(step_name, "device trust restricted".into()),
                _ => fail(step_name, "device not verified/trusted".into()),
            }
        }
        ProvisionStep::FirmwareValidation => {
            let Some(d) = device else {
                return fail(step_name, "device missing".into());
            };
            if d.firmware_version.as_deref().unwrap_or("").is_empty() {
                fail(step_name, "firmware_version not declared".into())
            } else {
                ok(step_name, "firmware declared".into())
            }
        }
        ProvisionStep::HealthValidation => {
            let Some(d) = device else {
                return fail(step_name, "device missing".into());
            };
            let lifecycle = d
                .lifecycle_state
                .as_deref()
                .map(DeviceLifecycleState::parse)
                .unwrap_or(DeviceLifecycleState::Discovered);
            if matches!(
                lifecycle,
                DeviceLifecycleState::Quarantined
                    | DeviceLifecycleState::Failed
                    | DeviceLifecycleState::Retired
            ) {
                return fail(step_name, format!("lifecycle state {}", lifecycle.as_str()));
            }
            let health = evaluate_device_readiness(d, 0.0);
            if health.health_status == "critical" {
                return fail(step_name, "health status critical".into());
            }
            if health.calibration_expired {
                return fail(step_name, "calibration expired".into());
            }
            if d.calibration_status
                .as_deref()
                .map(CalibrationStatus::parse)
                == Some(CalibrationStatus::Expired)
            {
                return fail(step_name, "calibration status expired".into());
            }
            ok(step_name, format!("lifecycle {}", lifecycle.as_str()))
        }
        ProvisionStep::CapabilityValidation => {
            let Some(d) = device else {
                return fail(step_name, "device missing".into());
            };
            if d.is_remote_actuator() && d.capabilities.is_empty() {
                fail(step_name, "remote actuator missing capabilities".into())
            } else {
                ok(step_name, "capabilities satisfied".into())
            }
        }
        ProvisionStep::Assign => {
            let robot = assign_robot
                .map(str::to_string)
                .or_else(|| device.and_then(|d| d.assigned_robot.clone()))
                .or_else(|| device.and_then(|d| d.robot_id.clone()));
            if let Some(ref robot) = robot {
                ok(step_name, format!("assigned to {robot}"))
            } else {
                fail(step_name, "no robot assignment".into())
            }
        }
        ProvisionStep::Ready => {
            if device.is_some() && assign_robot.is_some() {
                ok(step_name, "ready for operations".into())
            } else {
                fail(step_name, "provisioning incomplete".into())
            }
        }
    }
}

fn ok(step: String, message: String) -> ProvisionStepResult {
    ProvisionStepResult {
        step,
        passed: true,
        message,
    }
}

fn fail(step: String, message: String) -> ProvisionStepResult {
    ProvisionStepResult {
        step,
        passed: false,
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device_identity::DeviceIdentityRecord;

    fn sample_device() -> DeviceIdentityRecord {
        DeviceIdentityRecord {
            id: "lidar-1".into(),
            device_type: "lidar".into(),
            firmware_version: Some("1.2.0".into()),
            trust_level: Some("verified".into()),
            robot_id: Some("rover-1".into()),
            capabilities: vec!["sense".into()],
            lifecycle_state: Some("verified".into()),
            ..Default::default()
        }
    }

    #[test]
    fn provision_passes_for_valid_device() {
        let registry = DeviceRegistry {
            devices: vec![sample_device()],
        };
        let report = run_provision_workflow("lidar-1", &registry, Some("rover-1"));
        assert!(report.ready);
    }

    #[test]
    fn provision_fails_without_firmware() {
        let mut device = sample_device();
        device.firmware_version = None;
        let registry = DeviceRegistry {
            devices: vec![device],
        };
        let report = run_provision_workflow("lidar-1", &registry, Some("rover-1"));
        assert!(!report.ready);
    }
}
