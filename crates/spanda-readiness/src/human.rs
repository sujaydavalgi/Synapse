//! Operator, team, and mission readiness for human–robot collaboration.
//!
use crate::types::{ReadinessIssue, ReadinessSeverity};
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use spanda_config::{is_operator_capability, HumanEntity, HumanRegistry, ResolvedSystemConfig};

/// Per-dimension human readiness score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HumanDimensionScore {
    pub dimension: String,
    pub score: u32,
    pub weight_percent: u32,
    pub passed: bool,
    pub detail: String,
}

/// Human collaboration readiness report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HumanReadinessReport {
    pub profile: String,
    pub operator_ready: bool,
    pub team_ready: bool,
    pub mission_ready: bool,
    pub total_score: u32,
    pub minimum_score: u32,
    pub dimensions: Vec<HumanDimensionScore>,
    pub issues: Vec<ReadinessIssue>,
}

/// Weighted dimensions for human collaboration readiness.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HumanReadinessWeights {
    pub certification: u32,
    pub capability: u32,
    pub availability: u32,
    pub trust: u32,
    pub location: u32,
    pub permissions: u32,
    pub wearable_connectivity: u32,
}

impl Default for HumanReadinessWeights {
    fn default() -> Self {
        Self {
            certification: 25,
            capability: 20,
            availability: 15,
            trust: 15,
            location: 10,
            permissions: 10,
            wearable_connectivity: 5,
        }
    }
}

/// Evaluate operator, team, and mission readiness from resolved config.
pub fn evaluate_human_collaboration(
    cfg: &ResolvedSystemConfig,
    program: &Program,
) -> HumanReadinessReport {
    // Score human entities against collaboration profile gates and program operator capabilities.
    //
    // Parameters:
    // - `cfg` — resolved system config with human registry
    // - `program` — parsed mission program
    //
    // Returns:
    // Human readiness report with dimension breakdown.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = evaluate_human_collaboration(&cfg, &program);

    let profile = human_profile_name(cfg);
    let minimum_score = human_minimum_score(cfg);
    let weights = human_weights(cfg);
    let today = chrono::Utc::now().date_naive().to_string();
    let required_caps = operator_capabilities_required(program);
    let mut issues = Vec::new();
    let mut dimensions = Vec::new();

    if cfg.human_registry.humans.is_empty() {
        issues.push(ReadinessIssue {
            factor: "Operator".into(),
            severity: ReadinessSeverity::High,
            message: "no human operators configured in device tree".into(),
            suggested_action: Some("Add [[fleet.humans]] entries to spanda.devices.toml".into()),
        });
        return HumanReadinessReport {
            profile,
            operator_ready: false,
            team_ready: false,
            mission_ready: false,
            total_score: 0,
            minimum_score,
            dimensions,
            issues,
        };
    }

    let operator_scores: Vec<u32> = cfg
        .human_registry
        .humans
        .iter()
        .map(|human| {
            score_operator(
                human,
                &cfg.human_registry,
                &required_caps,
                &weights,
                &today,
                &mut issues,
            )
        })
        .collect();

    let operator_avg = average_score(&operator_scores);
    dimensions.push(HumanDimensionScore {
        dimension: "Operator".into(),
        score: operator_avg,
        weight_percent: 100,
        passed: operator_avg >= minimum_score,
        detail: format!("{} operator(s) evaluated", cfg.human_registry.humans.len()),
    });

    let supervisor_present = cfg.human_registry.humans.iter().any(|h| {
        h.role == "supervisor" && h.is_available() && h.trust_level_enum().is_operational()
    });
    let require_supervisor = human_require_supervisor(cfg);
    if require_supervisor && !supervisor_present {
        issues.push(ReadinessIssue {
            factor: "Team".into(),
            severity: ReadinessSeverity::High,
            message: "no available supervisor with operational trust".into(),
            suggested_action: Some("Assign supervisor role with trusted availability".into()),
        });
    }

    let team_ready = operator_avg >= minimum_score && (!require_supervisor || supervisor_present);
    let caps_covered = required_caps.iter().all(|cap| {
        cfg.human_registry
            .humans
            .iter()
            .any(|h| h.is_available() && h.has_capability(cap))
    });
    if !required_caps.is_empty() && !caps_covered {
        issues.push(ReadinessIssue {
            factor: "Mission".into(),
            severity: ReadinessSeverity::High,
            message: format!(
                "no available operator provides required capabilities: {}",
                required_caps.join(", ")
            ),
            suggested_action: Some("Update human capabilities or operator assignments".into()),
        });
    }

    let mission_ready = team_ready && (required_caps.is_empty() || caps_covered);

    HumanReadinessReport {
        profile,
        operator_ready: operator_avg >= minimum_score,
        team_ready,
        mission_ready,
        total_score: operator_avg,
        minimum_score,
        dimensions,
        issues,
    }
}

/// Format human readiness for CLI text output.
pub fn format_human_readiness(report: &HumanReadinessReport) -> String {
    let mut lines = vec![
        format!("Human collaboration profile: {}", report.profile),
        format!(
            "Score: {}/100 (minimum {}) — operator:{} team:{} mission:{}",
            report.total_score,
            report.minimum_score,
            yes_no(report.operator_ready),
            yes_no(report.team_ready),
            yes_no(report.mission_ready),
        ),
    ];
    for dim in &report.dimensions {
        lines.push(format!(
            "  {}: {} ({})",
            dim.dimension, dim.score, dim.detail
        ));
    }
    for issue in &report.issues {
        lines.push(format!(
            "  [{}] {:?}: {}",
            issue.factor, issue.severity, issue.message
        ));
    }
    lines.join("\n")
}

fn score_operator(
    human: &HumanEntity,
    registry: &HumanRegistry,
    required_caps: &[String],
    weights: &HumanReadinessWeights,
    today: &str,
    issues: &mut Vec<ReadinessIssue>,
) -> u32 {
    let cert_score = if required_role_certs_valid(human, today) {
        100
    } else {
        issues.push(ReadinessIssue {
            factor: "Operator".into(),
            severity: ReadinessSeverity::Medium,
            message: format!(
                "operator '{}' has expired or missing certifications",
                human.id
            ),
            suggested_action: None,
        });
        40
    };
    let capability_score =
        if required_caps.is_empty() || required_caps.iter().all(|cap| human.has_capability(cap)) {
            100
        } else {
            30
        };
    let availability_score = if human.is_available() { 100 } else { 0 };
    let trust_score = if human.trust_level_enum().is_operational() {
        100
    } else {
        50
    };
    let location_score = if human.location.is_some() { 100 } else { 80 };
    let permissions_score = if human.permissions.is_empty() {
        90
    } else {
        100
    };
    let wearable_score = score_wearables(human, registry);

    weighted_total(&[
        (cert_score, weights.certification),
        (capability_score, weights.capability),
        (availability_score, weights.availability),
        (trust_score, weights.trust),
        (location_score, weights.location),
        (permissions_score, weights.permissions),
        (wearable_score, weights.wearable_connectivity),
    ])
}

fn score_wearables(human: &HumanEntity, registry: &HumanRegistry) -> u32 {
    let linked = registry.wearables_for_human(&human.id);
    if linked.is_empty() {
        return 100;
    }
    let trusted = linked
        .iter()
        .filter(|w| {
            w.trust_level
                .as_deref()
                .map(spanda_config::TrustLevel::parse)
                .unwrap_or(spanda_config::TrustLevel::Unknown)
                .is_operational()
                || w.trust_level.is_none()
        })
        .count();
    ((trusted as u32) * 100 / linked.len() as u32).max(50)
}

fn required_role_certs_valid(human: &HumanEntity, today: &str) -> bool {
    human
        .certifications
        .iter()
        .all(|cert| cert_expires_on_or_after(cert.expires.as_ref(), today))
}

fn cert_expires_on_or_after(expires: Option<&String>, today: &str) -> bool {
    match expires {
        Some(date) => date.as_str() >= today,
        None => true,
    }
}

fn operator_capabilities_required(program: &Program) -> Vec<String> {
    let spanda_ast::nodes::Program::Program {
        requires_capabilities,
        ..
    } = program;
    requires_capabilities
        .iter()
        .map(|req| req.capability.clone())
        .filter(|cap| is_operator_capability(cap))
        .collect()
}

fn human_profile_name(cfg: &ResolvedSystemConfig) -> String {
    cfg.readiness_config()
        .and_then(|r| r.get("profile"))
        .and_then(|v| v.as_str())
        .unwrap_or("human_collaboration")
        .to_string()
}

fn human_minimum_score(cfg: &ResolvedSystemConfig) -> u32 {
    cfg.readiness_config()
        .and_then(|r| r.get("min_score"))
        .and_then(|v| v.as_integer())
        .map(|v| v as u32)
        .or_else(|| {
            cfg.readiness_config()
                .and_then(|r| r.get("profiles"))
                .and_then(|p| p.get("human_collaboration"))
                .and_then(|p| p.get("min_score"))
                .and_then(|v| v.as_integer())
                .map(|v| v as u32)
        })
        .unwrap_or(85)
}

fn human_require_supervisor(cfg: &ResolvedSystemConfig) -> bool {
    cfg.readiness_config()
        .and_then(|r| r.get("profiles"))
        .and_then(|p| p.get("human_collaboration"))
        .and_then(|p| p.get("require_supervisor_approval"))
        .and_then(|v| v.as_bool())
        .or_else(|| {
            cfg.readiness_config()
                .and_then(|r| r.get("gates"))
                .and_then(|g| g.get("require_supervisor_for_mission"))
                .and_then(|v| v.as_bool())
        })
        .unwrap_or(false)
}

fn human_weights(cfg: &ResolvedSystemConfig) -> HumanReadinessWeights {
    let table = cfg
        .readiness_config()
        .and_then(|r| r.get("profiles"))
        .and_then(|p| p.get("human_collaboration"))
        .and_then(|p| p.get("weights"))
        .and_then(|w| w.as_table());
    let pct = |key: &str, default: u32| -> u32 {
        table
            .and_then(|t| t.get(key))
            .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
            .map(|v| (v * 100.0).round() as u32)
            .unwrap_or(default)
    };
    HumanReadinessWeights {
        certification: pct("certification", 25),
        capability: pct("capability", 20),
        availability: pct("availability", 15),
        trust: pct("trust", 15),
        location: pct("location", 10),
        permissions: pct("permissions", 10),
        wearable_connectivity: pct("wearable_connectivity", 5),
    }
}

fn weighted_total(parts: &[(u32, u32)]) -> u32 {
    let total_weight: u32 = parts.iter().map(|(_, w)| w).sum();
    if total_weight == 0 {
        return 100;
    }
    let sum: u32 = parts.iter().map(|(score, weight)| score * weight).sum();
    (sum / total_weight).min(100)
}

fn average_score(scores: &[u32]) -> u32 {
    if scores.is_empty() {
        return 0;
    }
    scores.iter().sum::<u32>() / scores.len() as u32
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "ready"
    } else {
        "not_ready"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_config::ConfigResolver;

    #[test]
    fn human_readiness_scores_blueprint_operator() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/solutions/spatial-computing");
        let cfg = ConfigResolver::new()
            .resolve_from_dir(&root)
            .expect("resolve spatial computing blueprint");
        assert!(cfg.human_registry.has_operators());
        let source =
            std::fs::read_to_string(root.join("warehouse-ar/pick_mission.sd")).expect("read sd");
        let tokens = spanda_lexer::tokenize(&source).expect("tokenize");
        let program = spanda_parser::parse(tokens).expect("parse");
        let report = evaluate_human_collaboration(&cfg, &program);
        assert!(report.total_score >= 80, "score={}", report.total_score);
        assert!(report.operator_ready);
    }
}
