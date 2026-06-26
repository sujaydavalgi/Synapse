//! Device health, calibration, and readiness-blocking rules.
//!
use crate::device_identity::{DeviceIdentityRecord, DeviceRegistry};
use crate::device_pool::DeviceLifecycleState;
use serde::{Deserialize, Serialize};

/// Calibration posture for a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationStatus {
    #[default]
    Unknown,
    Valid,
    Expired,
    Required,
}

impl CalibrationStatus {
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "valid" => Self::Valid,
            "expired" => Self::Expired,
            "required" => Self::Required,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Valid => "valid",
            Self::Expired => "expired",
            Self::Required => "required",
        }
    }
}

/// Health rollup for a single device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceHealthStatus {
    pub device_id: String,
    pub health_status: String,
    pub calibration_status: CalibrationStatus,
    pub calibration_expired: bool,
    pub firmware_supported: bool,
    pub identity_verified: bool,
    pub lifecycle_ok: bool,
    pub readiness_blocked: bool,
    pub blockers: Vec<String>,
}

/// Evaluate whether a device blocks mission readiness.
pub fn evaluate_device_readiness(device: &DeviceIdentityRecord, now_ms: f64) -> DeviceHealthStatus {
    let mut blockers = Vec::new();
    let calibration = device
        .calibration_status
        .as_deref()
        .map(CalibrationStatus::parse)
        .unwrap_or(CalibrationStatus::Unknown);
    let calibration_expired = calibration == CalibrationStatus::Expired
        || device.calibration_expiry_ms.is_some_and(|exp| now_ms > exp);
    if calibration_expired {
        blockers.push("calibration_expired".into());
    }
    let firmware_supported = !firmware_unsupported(device);
    if !firmware_supported {
        blockers.push("firmware_unsupported".into());
    }
    let lifecycle = device
        .lifecycle_state
        .as_deref()
        .map(DeviceLifecycleState::parse)
        .unwrap_or(DeviceLifecycleState::Discovered);
    let lifecycle_ok = !matches!(
        lifecycle,
        DeviceLifecycleState::Quarantined
            | DeviceLifecycleState::Failed
            | DeviceLifecycleState::Retired
    );
    if !lifecycle_ok {
        blockers.push(format!("lifecycle_{}", lifecycle.as_str()));
    }
    let identity_verified = device.trust_level_enum().is_operational();
    if !identity_verified {
        blockers.push("identity_unverified".into());
    }
    let health_status = device
        .health_status
        .as_deref()
        .unwrap_or("unknown")
        .to_string();
    if health_status == "critical" {
        blockers.push("health_critical".into());
    }
    let readiness_blocked = !blockers.is_empty();
    DeviceHealthStatus {
        device_id: device.id.clone(),
        health_status,
        calibration_status: calibration,
        calibration_expired,
        firmware_supported,
        identity_verified,
        lifecycle_ok,
        readiness_blocked,
        blockers,
    }
}

/// Roll up readiness impact across the device pool.
pub fn readiness_impact(registry: &DeviceRegistry, now_ms: f64) -> ReadinessImpactReport {
    let statuses: Vec<DeviceHealthStatus> = registry
        .devices
        .iter()
        .map(|d| evaluate_device_readiness(d, now_ms))
        .collect();
    let blocked: Vec<_> = statuses
        .iter()
        .filter(|s| s.readiness_blocked)
        .cloned()
        .collect();
    ReadinessImpactReport {
        total_devices: registry.devices.len(),
        blocked_count: blocked.len(),
        blocked_devices: blocked,
    }
}

/// Report describing readiness impact from device health.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadinessImpactReport {
    pub total_devices: usize,
    pub blocked_count: usize,
    pub blocked_devices: Vec<DeviceHealthStatus>,
}

fn firmware_unsupported(device: &DeviceIdentityRecord) -> bool {
    let Some(ref min) = device.min_firmware_version else {
        return false;
    };
    let Some(ref current) = device.firmware_version else {
        return true;
    };
    compare_semver(current, min) < 0
}

fn compare_semver(current: &str, minimum: &str) -> i32 {
    let parse = |s: &str| -> Vec<u32> { s.split('.').filter_map(|p| p.parse().ok()).collect() };
    let cur = parse(current);
    let min = parse(minimum);
    for i in 0..cur.len().max(min.len()) {
        let c = cur.get(i).copied().unwrap_or(0);
        let m = min.get(i).copied().unwrap_or(0);
        if c != m {
            return (c as i32) - (m as i32);
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device_identity::DeviceIdentityRecord;

    #[test]
    fn blocks_when_calibration_expired() {
        let device = DeviceIdentityRecord {
            id: "gps-1".into(),
            device_type: "gps".into(),
            calibration_expiry_ms: Some(1000.0),
            trust_level: Some("verified".into()),
            firmware_version: Some("2.0.0".into()),
            lifecycle_state: Some("active".into()),
            ..Default::default()
        };
        let status = evaluate_device_readiness(&device, 2000.0);
        assert!(status.readiness_blocked);
        assert!(status.blockers.iter().any(|b| b == "calibration_expired"));
    }

    #[test]
    fn blocks_unsupported_firmware() {
        let device = DeviceIdentityRecord {
            id: "cam-1".into(),
            device_type: "camera".into(),
            firmware_version: Some("1.0.0".into()),
            min_firmware_version: Some("2.0.0".into()),
            trust_level: Some("verified".into()),
            lifecycle_state: Some("active".into()),
            ..Default::default()
        };
        let status = evaluate_device_readiness(&device, 0.0);
        assert!(status.readiness_blocked);
    }
}
