//! Assurance, mission, recovery, health, and provider settings from resolved config.
//!
use crate::resolved::ResolvedSystemConfig;
use spanda_package::adapter::framework_packages;
use std::collections::HashSet;

/// Scoring thresholds for mission assurance reports.
#[derive(Debug, Clone, PartialEq)]
pub struct AssurancePolicy {
    pub minimum_score: u32,
    pub require_recovery: bool,
    pub require_resilience: bool,
}

impl Default for AssurancePolicy {
    fn default() -> Self {
        Self {
            minimum_score: 70,
            require_recovery: true,
            require_resilience: true,
        }
    }
}

/// Mission planning requirements from config.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MissionPolicy {
    pub require_plans: bool,
    pub required_capabilities: Vec<String>,
}

/// Diagnosis sensitivity from config.
#[derive(Debug, Clone, PartialEq)]
pub struct DiagnosisPolicy {
    pub require_mitigations: bool,
    pub require_anomaly_handlers: bool,
}

impl Default for DiagnosisPolicy {
    fn default() -> Self {
        Self {
            require_mitigations: false,
            require_anomaly_handlers: false,
        }
    }
}

const DEFAULT_RECOVERY_FAILURES: &[&str] = &[
    "gps_loss",
    "battery_critical",
    "connectivity_loss",
    "sensor_failure",
    "actuator_failure",
    "provider_timeout",
    "fleet_peer_loss",
    "swarm_member_loss",
    "package_unavailable",
    "human_approval_timeout",
    "robot_failed",
];

const DEFAULT_HEALTH_FAULTS: &[&str] = &["GPSDegraded", "CameraOffline", "RobotHealthCritical"];

/// Parse `[assurance]` policy from resolved config.
pub fn assurance_policy(cfg: &ResolvedSystemConfig) -> AssurancePolicy {
    let mut policy = AssurancePolicy::default();
    let Some(section) = cfg.assurance_config() else {
        return policy;
    };
    if let Some(score) = section.get("minimum_score").and_then(|v| v.as_integer()) {
        policy.minimum_score = score as u32;
    }
    if let Some(v) = section.get("require_recovery").and_then(|v| v.as_bool()) {
        policy.require_recovery = v;
    }
    if let Some(v) = section.get("require_resilience").and_then(|v| v.as_bool()) {
        policy.require_resilience = v;
    }
    policy
}

/// Parse `[mission]` policy from resolved config.
pub fn mission_policy(cfg: &ResolvedSystemConfig) -> MissionPolicy {
    let Some(section) = cfg.mission_config() else {
        return MissionPolicy::default();
    };
    let require_plans = section
        .get("require_plans")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let required_capabilities = section
        .get("required_capabilities")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    MissionPolicy {
        require_plans,
        required_capabilities,
    }
}

/// Parse diagnosis policy from `[assurance]` or `[diagnosis]` sections.
pub fn diagnosis_policy(cfg: &ResolvedSystemConfig) -> DiagnosisPolicy {
    let mut policy = DiagnosisPolicy::default();
    if let Some(section) = cfg.raw.get("diagnosis").and_then(|v| v.as_table()) {
        if let Some(v) = section.get("require_mitigations").and_then(|v| v.as_bool()) {
            policy.require_mitigations = v;
        }
        if let Some(v) = section
            .get("require_anomaly_handlers")
            .and_then(|v| v.as_bool())
        {
            policy.require_anomaly_handlers = v;
        }
        return policy;
    }
    if let Some(section) = cfg.assurance_config().and_then(|v| v.as_table()) {
        if let Some(v) = section.get("require_mitigations").and_then(|v| v.as_bool()) {
            policy.require_mitigations = v;
        }
        if let Some(v) = section
            .get("require_anomaly_handlers")
            .and_then(|v| v.as_bool())
        {
            policy.require_anomaly_handlers = v;
        }
    }
    policy
}

/// Failure catalog for recovery coverage (`[recovery].known_failures` or built-in list).
pub fn recovery_failure_catalog(cfg: &ResolvedSystemConfig) -> Vec<String> {
    if let Some(section) = cfg.recovery_config() {
        if let Some(list) = section.get("known_failures").and_then(|v| v.as_array()) {
            let custom: Vec<String> = list
                .iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect();
            if !custom.is_empty() {
                return custom;
            }
        }
    }
    DEFAULT_RECOVERY_FAILURES
        .iter()
        .map(|s| (*s).to_string())
        .collect()
}

/// Health faults to inject for a robot during runtime readiness simulation.
pub fn health_inject_faults(cfg: &ResolvedSystemConfig, robot_id: &str) -> Vec<String> {
    if let Some(robot) = cfg.health_policy_for(robot_id) {
        if let Some(list) = robot.get("inject_faults").and_then(|v| v.as_array()) {
            let faults: Vec<String> = list
                .iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect();
            if !faults.is_empty() {
                return faults;
            }
        }
    }
    if let Some(section) = cfg.raw.get("health") {
        if let Some(list) = section.get("inject_faults").and_then(|v| v.as_array()) {
            let faults: Vec<String> = list
                .iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect();
            if !faults.is_empty() {
                return faults;
            }
        }
    }
    DEFAULT_HEALTH_FAULTS
        .iter()
        .map(|s| (*s).to_string())
        .collect()
}

/// Provider package names for runtime bootstrap (lockfile + device tree providers).
pub fn provider_packages_for_runtime(cfg: &ResolvedSystemConfig) -> Vec<String> {
    let mut names: HashSet<String> = crate::integration::official_packages_from_resolved(cfg)
        .into_iter()
        .collect();
    for (_robot, _compute, device) in cfg.device_tree.all_devices() {
        if let Some(ref provider) = device.provider {
            names.insert(provider.clone());
        }
    }
    names.extend(cfg.providers.clone());
    let known: HashSet<&str> = framework_packages().iter().map(|p| p.name).collect();
    let mut out: Vec<String> = names
        .into_iter()
        .filter(|n| known.contains(n.as_str()))
        .collect();
    out.sort();
    out
}

/// Compute assurance score (0–100) from a mission assurance summary pass/fail factors.
pub fn assurance_score_from_flags(passed_flags: &[bool]) -> u32 {
    if passed_flags.is_empty() {
        return 0;
    }
    let ok = passed_flags.iter().filter(|p| **p).count();
    ((ok * 100) / passed_flags.len()) as u32
}
