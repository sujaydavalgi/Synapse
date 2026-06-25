//! Configuration drift detection between resolved system baselines.
//!
use crate::device_identity::DeviceIdentityRecord;
use crate::mapping::{ActuatorMapping, LogicalPhysicalMap, SensorMapping};
use crate::resolver::diff_configs;
use crate::resolved::ResolvedSystemConfig;
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use std::collections::{BTreeSet, HashMap};

/// Severity tier for a drift finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Configuration area where drift was observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftDimension {
    Configuration,
    Fleet,
    Device,
    Provider,
    Package,
    Mapping,
    Program,
}

/// Single drift delta between baseline and current configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriftFinding {
    pub dimension: DriftDimension,
    pub severity: DriftSeverity,
    pub message: String,
    pub path: Option<String>,
}

/// Structured drift report for baseline vs current resolved configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigDriftReport {
    pub findings: Vec<DriftFinding>,
    pub passed: bool,
    pub baseline_project: String,
    pub current_project: String,
}

impl ConfigDriftReport {
    pub fn push(&mut self, finding: DriftFinding) {
        if finding.severity >= DriftSeverity::Medium {
            self.passed = false;
        }
        self.findings.push(finding);
    }
}

/// Compare two resolved configurations and emit structured drift findings.
pub fn detect_config_drift(
    baseline: &ResolvedSystemConfig,
    current: &ResolvedSystemConfig,
) -> ConfigDriftReport {
    // Compare baseline and current resolved configs for operational drift.
    //
    // Parameters:
    // - `baseline` — approved or expected configuration
    // - `current` — configuration under inspection
    //
    // Returns:
    // Structured drift report with severity-tagged findings.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = detect_config_drift(&approved, &live);

    let mut report = ConfigDriftReport {
        findings: Vec::new(),
        passed: true,
        baseline_project: baseline.project_name().to_string(),
        current_project: current.project_name().to_string(),
    };

    // Flag raw TOML key deltas under the configuration dimension.
    for line in diff_configs(&baseline.raw, &current.raw) {
        report.push(DriftFinding {
            dimension: DriftDimension::Configuration,
            severity: DriftSeverity::Medium,
            message: line,
            path: None,
        });
    }

    // Compare fleet identifiers when both sides declare a fleet.
    if baseline.fleet_id() != current.fleet_id() {
        report.push(DriftFinding {
            dimension: DriftDimension::Fleet,
            severity: DriftSeverity::High,
            message: format!(
                "fleet.id changed: {:?} -> {:?}",
                baseline.fleet_id(),
                current.fleet_id()
            ),
            path: Some("fleet.id".into()),
        });
    }

    // Detect robots added or removed from the fleet tree.
    diff_string_sets(
        &mut report,
        DriftDimension::Fleet,
        "robot",
        &baseline.robot_ids().into_iter().map(str::to_string).collect::<Vec<_>>(),
        &current.robot_ids().into_iter().map(str::to_string).collect::<Vec<_>>(),
    );

    // Compare provider and package manifests.
    diff_string_sets(
        &mut report,
        DriftDimension::Provider,
        "provider",
        &baseline.providers,
        &current.providers,
    );
    diff_string_sets(
        &mut report,
        DriftDimension::Package,
        "package",
        &baseline.packages,
        &current.packages,
    );

    // Compare device identity records field-by-field.
    diff_device_registry(&mut report, baseline, current);

    // Compare logical-to-physical mappings derived from each config.
    diff_logical_map(&mut report, &baseline.logical_map, &current.logical_map);

    report
}

/// Append program alignment findings against the current resolved configuration.
pub fn append_program_drift(
    report: &mut ConfigDriftReport,
    program: &Program,
    current: &ResolvedSystemConfig,
) {
    // Add program-vs-config mapping drift to an existing report.
    //
    // Parameters:
    // - `report` — drift report to extend in place
    // - `program` — parsed `.sd` program
    // - `current` — live resolved configuration
    //
    // Returns:
    // Nothing; findings are appended to `report`.
    //
    // Options:
    // None.
    //
    // Example:
    // append_program_drift(&mut report, &program, &current);

    for issue in current
        .logical_map
        .verify_against_program(program, &current.device_registry)
    {
        report.push(DriftFinding {
            dimension: DriftDimension::Program,
            severity: DriftSeverity::High,
            message: issue,
            path: None,
        });
    }
}

/// Render drift findings as human-readable lines (legacy text consumers).
pub fn format_drift_lines(report: &ConfigDriftReport) -> Vec<String> {
    report
        .findings
        .iter()
        .map(|finding| {
            let path = finding
                .path
                .as_deref()
                .map(|p| format!(" @ {p}"))
                .unwrap_or_default();
            format!(
                "[{:?}/{:?}] {}{}",
                finding.dimension, finding.severity, finding.message, path
            )
        })
        .collect()
}

fn diff_string_sets(
    report: &mut ConfigDriftReport,
    dimension: DriftDimension,
    label: &str,
    baseline: &[String],
    current: &[String],
) {
    let base: BTreeSet<&str> = baseline.iter().map(String::as_str).collect();
    let live: BTreeSet<&str> = current.iter().map(String::as_str).collect();
    for item in base.difference(&live) {
        report.push(DriftFinding {
            dimension,
            severity: DriftSeverity::High,
            message: format!("removed {label} '{item}'"),
            path: Some(item.to_string()),
        });
    }
    for item in live.difference(&base) {
        report.push(DriftFinding {
            dimension,
            severity: DriftSeverity::Medium,
            message: format!("added {label} '{item}'"),
            path: Some(item.to_string()),
        });
    }
}

fn diff_device_registry(
    report: &mut ConfigDriftReport,
    baseline: &ResolvedSystemConfig,
    current: &ResolvedSystemConfig,
) {
    let base: HashMap<&str, &DeviceIdentityRecord> = baseline
        .device_registry
        .devices
        .iter()
        .map(|d| (d.id.as_str(), d))
        .collect();
    let live: HashMap<&str, &DeviceIdentityRecord> = current
        .device_registry
        .devices
        .iter()
        .map(|d| (d.id.as_str(), d))
        .collect();

    for id in base.keys() {
        if !live.contains_key(id) {
            report.push(DriftFinding {
                dimension: DriftDimension::Device,
                severity: DriftSeverity::Critical,
                message: format!("device '{id}' removed"),
                path: Some(format!("devices.{id}")),
            });
        }
    }
    for id in live.keys() {
        if !base.contains_key(id) {
            report.push(DriftFinding {
                dimension: DriftDimension::Device,
                severity: DriftSeverity::High,
                message: format!("device '{id}' added"),
                path: Some(format!("devices.{id}")),
            });
        }
    }
    for (id, base_device) in &base {
        let Some(live_device) = live.get(id) else {
            continue;
        };
        diff_optional_field(report, id, "ip", &base_device.ip_address, &live_device.ip_address);
        diff_optional_field(
            report,
            id,
            "endpoint",
            &base_device.endpoint_url,
            &live_device.endpoint_url,
        );
        diff_optional_field(
            report,
            id,
            "provider",
            &base_device.provider,
            &live_device.provider,
        );
        diff_optional_field(
            report,
            id,
            "firmware",
            &base_device.firmware_version,
            &live_device.firmware_version,
        );
        diff_optional_field(
            report,
            id,
            "trust_level",
            &base_device.trust_level,
            &live_device.trust_level,
        );
        diff_optional_field(
            report,
            id,
            "security_identity",
            &base_device.security_identity,
            &live_device.security_identity,
        );
        if base_device.capabilities != live_device.capabilities {
            report.push(DriftFinding {
                dimension: DriftDimension::Device,
                severity: DriftSeverity::Medium,
                message: format!(
                    "device '{id}' capabilities changed: {:?} -> {:?}",
                    base_device.capabilities, live_device.capabilities
                ),
                path: Some(format!("devices.{id}.capabilities")),
            });
        }
    }
}

fn diff_optional_field(
    report: &mut ConfigDriftReport,
    device_id: &str,
    field: &str,
    baseline: &Option<String>,
    current: &Option<String>,
) {
    if baseline == current {
        return;
    }
    report.push(DriftFinding {
        dimension: DriftDimension::Device,
        severity: if field == "security_identity" || field == "trust_level" {
            DriftSeverity::High
        } else {
            DriftSeverity::Medium
        },
        message: format!(
            "device '{device_id}' {field}: {:?} -> {:?}",
            baseline, current
        ),
        path: Some(format!("devices.{device_id}.{field}")),
    });
}

fn diff_logical_map(
    report: &mut ConfigDriftReport,
    baseline: &LogicalPhysicalMap,
    current: &LogicalPhysicalMap,
) {
    diff_sensor_map(report, baseline, current);
    diff_actuator_map(report, baseline, current);
}

fn diff_sensor_map(
    report: &mut ConfigDriftReport,
    baseline: &LogicalPhysicalMap,
    current: &LogicalPhysicalMap,
) {
    let base_keys: BTreeSet<&str> = baseline.sensors.keys().map(String::as_str).collect();
    let live_keys: BTreeSet<&str> = current.sensors.keys().map(String::as_str).collect();
    for key in base_keys.difference(&live_keys) {
        report.push(DriftFinding {
            dimension: DriftDimension::Mapping,
            severity: DriftSeverity::High,
            message: format!("sensor mapping '{key}' removed"),
            path: Some(format!("mapping.sensors.{key}")),
        });
    }
    for key in live_keys.difference(&base_keys) {
        report.push(DriftFinding {
            dimension: DriftDimension::Mapping,
            severity: DriftSeverity::Medium,
            message: format!("sensor mapping '{key}' added"),
            path: Some(format!("mapping.sensors.{key}")),
        });
    }
    for key in base_keys.intersection(&live_keys) {
        diff_sensor_fields(report, key, baseline.sensors.get(*key).unwrap(), current.sensors.get(*key).unwrap());
    }
}

fn diff_actuator_map(
    report: &mut ConfigDriftReport,
    baseline: &LogicalPhysicalMap,
    current: &LogicalPhysicalMap,
) {
    let base_keys: BTreeSet<&str> = baseline.actuators.keys().map(String::as_str).collect();
    let live_keys: BTreeSet<&str> = current.actuators.keys().map(String::as_str).collect();
    for key in base_keys.difference(&live_keys) {
        report.push(DriftFinding {
            dimension: DriftDimension::Mapping,
            severity: DriftSeverity::High,
            message: format!("actuator mapping '{key}' removed"),
            path: Some(format!("mapping.actuators.{key}")),
        });
    }
    for key in live_keys.difference(&base_keys) {
        report.push(DriftFinding {
            dimension: DriftDimension::Mapping,
            severity: DriftSeverity::Medium,
            message: format!("actuator mapping '{key}' added"),
            path: Some(format!("mapping.actuators.{key}")),
        });
    }
    for key in base_keys.intersection(&live_keys) {
        diff_actuator_fields(
            report,
            key,
            baseline.actuators.get(*key).unwrap(),
            current.actuators.get(*key).unwrap(),
        );
    }
}

fn diff_sensor_fields(
    report: &mut ConfigDriftReport,
    key: &str,
    baseline: &SensorMapping,
    current: &SensorMapping,
) {
    if baseline.physical_device_id != current.physical_device_id {
        report.push(DriftFinding {
            dimension: DriftDimension::Mapping,
            severity: DriftSeverity::High,
            message: format!(
                "sensor '{key}' physical device: {} -> {}",
                baseline.physical_device_id, current.physical_device_id
            ),
            path: Some(format!("mapping.sensors.{key}.physical_device_id")),
        });
    }
    diff_optional_field(report, key, "ip", &baseline.ip_address, &current.ip_address);
    diff_optional_field(report, key, "endpoint", &baseline.endpoint_url, &current.endpoint_url);
}

fn diff_actuator_fields(
    report: &mut ConfigDriftReport,
    key: &str,
    baseline: &ActuatorMapping,
    current: &ActuatorMapping,
) {
    if baseline.physical_device_id != current.physical_device_id {
        report.push(DriftFinding {
            dimension: DriftDimension::Mapping,
            severity: DriftSeverity::High,
            message: format!(
                "actuator '{key}' physical device: {} -> {}",
                baseline.physical_device_id, current.physical_device_id
            ),
            path: Some(format!("mapping.actuators.{key}.physical_device_id")),
        });
    }
    if baseline.has_emergency_stop != current.has_emergency_stop {
        report.push(DriftFinding {
            dimension: DriftDimension::Mapping,
            severity: DriftSeverity::Critical,
            message: format!(
                "actuator '{key}' emergency_stop capability: {} -> {}",
                baseline.has_emergency_stop, current.has_emergency_stop
            ),
            path: Some(format!("mapping.actuators.{key}.emergency_stop")),
        });
    }
    diff_optional_field(report, key, "ip", &baseline.ip_address, &current.ip_address);
    diff_optional_field(report, key, "endpoint", &baseline.endpoint_url, &current.endpoint_url);
}
