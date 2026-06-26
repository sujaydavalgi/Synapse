//! Quarantine rules — unknown or untrusted devices cannot affect safety-critical paths.
//!
use crate::device_identity::{DeviceIdentityRecord, TrustLevel};
use crate::device_pool::DeviceLifecycleState;
use serde::{Deserialize, Serialize};

/// Outcome of evaluating quarantine policy for a device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuarantinePolicyResult {
    pub device_id: String,
    pub quarantined: bool,
    pub can_control_actuators: bool,
    pub can_publish_trusted_safety_data: bool,
    pub can_satisfy_mission_capabilities: bool,
    pub requires_operator_approval: bool,
    pub reasons: Vec<String>,
}

/// Evaluate quarantine policy for a device record.
pub fn evaluate_quarantine_policy(device: &DeviceIdentityRecord) -> QuarantinePolicyResult {
    let lifecycle = device
        .lifecycle_state
        .as_deref()
        .map(DeviceLifecycleState::parse)
        .unwrap_or(DeviceLifecycleState::Discovered);
    let trust = device.trust_level_enum();
    let mut reasons = Vec::new();
    let quarantined = lifecycle == DeviceLifecycleState::Quarantined
        || trust == TrustLevel::Unknown
        || trust == TrustLevel::Unverified
        || trust == TrustLevel::Restricted;
    if lifecycle == DeviceLifecycleState::Quarantined {
        reasons.push("lifecycle_quarantined".into());
    }
    if matches!(trust, TrustLevel::Unknown | TrustLevel::Unverified) {
        reasons.push("trust_not_verified".into());
    }
    if trust == TrustLevel::Restricted {
        reasons.push("trust_restricted".into());
    }
    if device.endpoint_is_insecure() && device.certificate_fingerprint.is_none() {
        reasons.push("insecure_endpoint".into());
    }
    let requires_operator_approval = quarantined;
    let actuator_blocked = quarantined && device.is_remote_actuator();
    QuarantinePolicyResult {
        device_id: device.id.clone(),
        quarantined,
        can_control_actuators: !actuator_blocked,
        can_publish_trusted_safety_data: !quarantined && trust.is_operational(),
        can_satisfy_mission_capabilities: !quarantined
            && trust.is_operational()
            && lifecycle != DeviceLifecycleState::Failed,
        requires_operator_approval,
        reasons,
    }
}

/// Whether a quarantined device may control actuators.
pub fn can_control_actuators(device: &DeviceIdentityRecord) -> bool {
    evaluate_quarantine_policy(device).can_control_actuators
}

/// Whether a quarantined device may publish trusted safety data.
pub fn can_publish_trusted_safety_data(device: &DeviceIdentityRecord) -> bool {
    evaluate_quarantine_policy(device).can_publish_trusted_safety_data
}

/// Whether a quarantined device may satisfy mission capabilities.
pub fn can_satisfy_mission_capabilities(device: &DeviceIdentityRecord) -> bool {
    evaluate_quarantine_policy(device).can_satisfy_mission_capabilities
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device_identity::DeviceIdentityRecord;

    #[test]
    fn quarantined_device_cannot_control_actuators() {
        let device = DeviceIdentityRecord {
            id: "drive-1".into(),
            device_type: "DifferentialDrive".into(),
            lifecycle_state: Some("quarantined".into()),
            endpoint_url: Some("http://192.168.1.10".into()),
            capabilities: vec!["move".into(), "stop".into()],
            ..Default::default()
        };
        let policy = evaluate_quarantine_policy(&device);
        assert!(policy.quarantined);
        assert!(!policy.can_control_actuators);
        assert!(!policy.can_satisfy_mission_capabilities);
    }

    #[test]
    fn trusted_device_passes_quarantine() {
        let device = DeviceIdentityRecord {
            id: "lidar-1".into(),
            device_type: "lidar".into(),
            lifecycle_state: Some("active".into()),
            trust_level: Some("trusted".into()),
            ..Default::default()
        };
        let policy = evaluate_quarantine_policy(&device);
        assert!(!policy.quarantined);
        assert!(policy.can_satisfy_mission_capabilities);
    }
}
