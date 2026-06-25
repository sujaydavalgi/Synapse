//! Readiness integration with resolved system configuration.
//!
use crate::types::{ReadinessPolicy, ReadinessWeights};
use spanda_config::ResolvedSystemConfig;

/// Build readiness scoring policy from merged `[readiness]` config.
pub fn policy_from_system_config(cfg: &ResolvedSystemConfig) -> Option<ReadinessPolicy> {
    let section = cfg.readiness_config()?;
    let minimum_score = section
        .get("minimum_score")
        .and_then(|v| v.as_integer())
        .map(|v| v as u32)
        .unwrap_or(80);
    let weights_table = section.get("weights").and_then(|v| v.as_table());
    let weight = |key: &str, default: u32| -> u32 {
        weights_table
            .and_then(|t| t.get(key))
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or(default)
    };
    let default_weights = ReadinessWeights::default();
    Some(ReadinessPolicy {
        minimum_score,
        weights: ReadinessWeights {
            hardware: weight("hardware", default_weights.hardware),
            capabilities: weight("capabilities", default_weights.capabilities),
            health: weight("health", default_weights.health),
            connectivity: weight("connectivity", default_weights.connectivity),
            safety: weight("safety", default_weights.safety),
            battery: weight("battery", default_weights.battery),
            storage: weight("storage", default_weights.storage),
            compute: weight("compute", default_weights.compute),
            packages: weight("packages", default_weights.packages),
            providers: weight("providers", default_weights.providers),
            mission: weight("mission", default_weights.mission),
            assurance: weight("assurance", default_weights.assurance),
        },
    })
}

/// Add readiness issues when program robots are missing from configured fleet.
pub fn config_robot_alignment_issues(
    cfg: &ResolvedSystemConfig,
    program_robots: &[String],
) -> Vec<(crate::types::ReadinessSeverity, String)> {
    let configured = spanda_config::configured_robot_ids(cfg);
    if configured.is_empty() {
        return Vec::new();
    }
    let mut issues = Vec::new();
    for robot in program_robots {
        if !configured.iter().any(|id| id == robot) {
            issues.push((
                crate::types::ReadinessSeverity::Medium,
                format!("program robot '{robot}' has no entry in resolved fleet config"),
            ));
        }
    }
    issues
}

/// Readiness issues for networked devices missing identity or endpoint metadata.
pub fn config_device_identity_issues(
    cfg: &ResolvedSystemConfig,
) -> Vec<(crate::types::ReadinessSeverity, String)> {
    let mut issues = Vec::new();
    for device in &cfg.device_registry.devices {
        if !device.is_networked() {
            continue;
        }
        if device.endpoint_url.is_none() && device.ip_address.is_none() {
            issues.push((
                crate::types::ReadinessSeverity::High,
                format!(
                    "network device '{}' missing endpoint or IP in config",
                    device.id
                ),
            ));
        }
        if device.security_identity.is_none() && device.certificate_fingerprint.is_none() {
            issues.push((
                crate::types::ReadinessSeverity::Medium,
                format!("network device '{}' missing security identity", device.id),
            ));
        }
        if device.trust_level_enum() == spanda_config::TrustLevel::Unknown {
            issues.push((
                crate::types::ReadinessSeverity::Low,
                format!("network device '{}' has unknown trust_level", device.id),
            ));
        }
    }
    issues
}

/// Readiness issues when current configuration drifts from an approved baseline.
pub fn config_drift_issues(
    baseline: &ResolvedSystemConfig,
    current: &ResolvedSystemConfig,
    program: Option<&spanda_ast::nodes::Program>,
) -> Vec<(crate::types::ReadinessSeverity, String)> {
    let mut report = spanda_config::detect_config_drift(baseline, current);
    if let Some(prog) = program {
        spanda_config::append_program_drift(&mut report, prog, current);
    }
    report
        .findings
        .into_iter()
        .filter(|f| f.severity >= spanda_config::DriftSeverity::Medium)
        .map(|f| (drift_severity_to_readiness(f.severity), f.message))
        .collect()
}

fn drift_severity_to_readiness(
    severity: spanda_config::DriftSeverity,
) -> crate::types::ReadinessSeverity {
    match severity {
        spanda_config::DriftSeverity::Info => crate::types::ReadinessSeverity::Low,
        spanda_config::DriftSeverity::Low => crate::types::ReadinessSeverity::Low,
        spanda_config::DriftSeverity::Medium => crate::types::ReadinessSeverity::Medium,
        spanda_config::DriftSeverity::High => crate::types::ReadinessSeverity::High,
        spanda_config::DriftSeverity::Critical => crate::types::ReadinessSeverity::Critical,
    }
}
