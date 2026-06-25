//! Threat model builders from program structure and security analysis.

use spanda_ast::comm_decl::TransportKind;
use spanda_ast::foundations::{DeployDecl, KillSwitchDecl, PermissionsDecl, SecureCommPolicyDecl};
use spanda_ast::nodes::{Program, RobotDecl};
use spanda_security::{security_analyze_program, SecurityFinding, SecuritySeverity};
use serde::{Deserialize, Serialize};

/// Threat category aligned with platform maturity taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreatCategory {
    Connectivity,
    Transport,
    Ota,
    RemoteCommand,
    AgentPermissions,
    ProviderPermissions,
    SecurityControl,
}

/// Risk tier for a modeled threat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreatRisk {
    Low,
    Medium,
    High,
    Critical,
}

/// One attack-surface element discovered in source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttackSurfaceItem {
    pub category: ThreatCategory,
    pub detail: String,
    pub line: Option<u32>,
}

/// Per-threat assessment with risk and mitigation hint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreatAssessment {
    pub id: String,
    pub category: ThreatCategory,
    pub risk: ThreatRisk,
    pub description: String,
    pub mitigation: String,
}

/// Full static threat model report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreatReport {
    pub program: String,
    pub attack_surface: Vec<AttackSurfaceItem>,
    pub assessments: Vec<ThreatAssessment>,
    pub mitigations: Vec<String>,
    pub risk_score: u32,
    pub passed: bool,
}

/// Analyze attack surface and threats for a parsed program.
pub fn analyze_threat_model(program: &Program, source_label: &str) -> ThreatReport {
    // Build attack surface and threat assessments from AST and security findings.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    // - `source_label` — file label for the report
    //
    // Returns:
    // Threat model report with surface, assessments, and mitigations.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = analyze_threat_model(&program, "rover.sd");

    let security = security_analyze_program(program);
    let mut attack_surface = Vec::new();
    let mut assessments = Vec::new();
    let mut mitigations = Vec::new();

    let Program::Program {
        imports,
        deployments,
        requires_network,
        connectivity_policies,
        kill_switches,
        robots,
        ..
    } = program;

    if requires_network.is_some() {
        attack_surface.push(surface(
            ThreatCategory::Connectivity,
            "program declares requires_network",
            None,
        ));
        assessments.push(assessment(
            "NET-001",
            ThreatCategory::Connectivity,
            ThreatRisk::Medium,
            "Network dependency expands remote attack surface",
            "Enforce secure_comm, TLS, and trust boundaries on all external links",
        ));
    }

    for policy in connectivity_policies {
        let spanda_ast::foundations::ConnectivityPolicyDecl::ConnectivityPolicyDecl {
            preferred,
            fallback,
            emergency,
            ..
        } = policy;
        attack_surface.push(surface(
            ThreatCategory::Connectivity,
            format!("connectivity policy preferred={preferred} fallback={fallback}"),
            None,
        ));
        if emergency.is_some() {
            assessments.push(assessment(
                "NET-002",
                ThreatCategory::Connectivity,
                ThreatRisk::Low,
                "Emergency connectivity channel may bypass primary security controls",
                "Apply identical encryption and authentication on emergency links",
            ));
        }
    }

    for import in imports {
        let spanda_ast::nodes::ImportDecl::ImportDecl { path, .. } = import;
        attack_surface.push(surface(
            ThreatCategory::ProviderPermissions,
            format!("imports provider package `{path}`"),
            None,
        ));
        assessments.push(assessment(
            "PKG-001",
            ThreatCategory::ProviderPermissions,
            ThreatRisk::Medium,
            format!("Third-party package `{path}` extends runtime privileges"),
            "Run `spanda trust` and verify adapter capabilities before install",
        ));
    }

    for deployment in deployments {
        let DeployDecl::DeployDecl {
            robot_name,
            targets,
            ..
        } = deployment;
        attack_surface.push(surface(
            ThreatCategory::Ota,
            format!("deploy {robot_name} to {}", targets.join(", ")),
            None,
        ));
        assessments.push(assessment(
            "OTA-001",
            ThreatCategory::Ota,
            ThreatRisk::High,
            "OTA/deploy channel can deliver tampered mission or firmware",
            "Use signed bundles, `spanda deploy gate`, and registry trust scoring",
        ));
        mitigations.push("Require signed OTA artifacts and deployment gates before rollout".into());
    }

    for ks in kill_switches {
        collect_kill_switch_threat(ks, "program", &mut attack_surface, &mut assessments);
    }

    for robot in robots {
        collect_robot_surface(robot, &mut attack_surface, &mut assessments);
    }

    for finding in &security.findings {
        map_security_finding(finding, &mut assessments);
    }

    if security.has_errors() {
        mitigations.push("Resolve security check errors before production deploy".into());
    }
    if !assessments.iter().any(|a| a.category == ThreatCategory::Transport) {
        mitigations.push("Declare secure_comm and trust_boundary for networked robots".into());
    }

    let risk_score = score_assessments(&assessments);
    let passed = !security.has_errors()
        && !assessments
            .iter()
            .any(|a| matches!(a.risk, ThreatRisk::Critical | ThreatRisk::High) && a.id.starts_with("SEC-ERR"));
    ThreatReport {
        program: source_label.into(),
        attack_surface,
        assessments,
        mitigations,
        risk_score,
        passed,
    }
}

fn collect_robot_surface(
    robot: &RobotDecl,
    attack_surface: &mut Vec<AttackSurfaceItem>,
    assessments: &mut Vec<ThreatAssessment>,
) {
    let RobotDecl::RobotDecl {
        name,
        secure_comm,
        permissions,
        buses,
        kill_switches,
        requires_network,
        ..
    } = robot;

    if let Some(net) = requires_network {
        let _ = net;
        attack_surface.push(surface(
            ThreatCategory::Connectivity,
            format!("robot {name} requires network"),
            None,
        ));
    }

    if let Some(sc) = secure_comm {
        let SecureCommPolicyDecl::SecureCommPolicyDecl {
            encryption,
            authentication,
            ..
        } = sc;
        let enc = encryption.as_deref().unwrap_or("unspecified");
        let auth = authentication.as_deref().unwrap_or("unspecified");
        attack_surface.push(surface(
            ThreatCategory::Transport,
            format!("robot {name} secure_comm encryption={enc} authentication={auth}"),
            None,
        ));
        if enc == "none" || enc == "optional" {
            assessments.push(assessment(
                "TRN-001",
                ThreatCategory::Transport,
                ThreatRisk::High,
                format!("Robot {name} allows unencrypted transport"),
                "Set secure_comm encryption to required",
            ));
        }
    }

    for bus in buses {
        let spanda_ast::comm_decl::BusDecl::BusDecl { transport, .. } = bus;
        let detail = match transport {
            TransportKind::Mqtt => "MQTT bus",
            TransportKind::Ros2 => "ROS 2 bus",
            TransportKind::Dds => "DDS bus",
            TransportKind::Websocket => "WebSocket bus",
            TransportKind::Local => "local bus",
            TransportKind::Sim => "simulated bus",
        };
        attack_surface.push(surface(
            ThreatCategory::Transport,
            format!("robot {name} uses {detail}"),
            None,
        ));
    }

    if let Some(perms) = permissions {
        let PermissionsDecl::PermissionsDecl { capabilities, .. } = perms;
        attack_surface.push(surface(
            ThreatCategory::AgentPermissions,
            format!("robot {name} permissions [{}]", capabilities.join(", ")),
            None,
        ));
        if capabilities.iter().any(|c| c.contains("actuator") || c.contains("deploy")) {
            assessments.push(assessment(
                "AGT-001",
                ThreatCategory::AgentPermissions,
                ThreatRisk::High,
                format!("Robot {name} grants high-impact actuator or deploy capabilities"),
                "Scope permissions to least privilege and audit operator actions",
            ));
        }
    }

    for ks in kill_switches {
        collect_kill_switch_threat(ks, &format!("robot {name}"), attack_surface, assessments);
    }
}

fn collect_kill_switch_threat(
    ks: &KillSwitchDecl,
    scope: &str,
    attack_surface: &mut Vec<AttackSurfaceItem>,
    assessments: &mut Vec<ThreatAssessment>,
) {
    let KillSwitchDecl::KillSwitchDecl {
        name: ks_name,
        remote_signed,
        ..
    } = ks;
    if *remote_signed {
        attack_surface.push(surface(
            ThreatCategory::RemoteCommand,
            format!("{scope} kill switch `{ks_name}` accepts remote_signed commands"),
            None,
        ));
        assessments.push(assessment(
            "REM-001",
            ThreatCategory::RemoteCommand,
            ThreatRisk::High,
            format!("Remote-signed kill switch `{ks_name}` on {scope}"),
            "Require operator authentication, replay protection, and audit logging",
        ));
    }
}

fn map_security_finding(finding: &SecurityFinding, assessments: &mut Vec<ThreatAssessment>) {
    let risk = match finding.severity {
        SecuritySeverity::Error => ThreatRisk::High,
        SecuritySeverity::Warning => ThreatRisk::Medium,
        SecuritySeverity::Info => ThreatRisk::Low,
    };
    let id = if finding.severity == SecuritySeverity::Error {
        "SEC-ERR"
    } else if finding.severity == SecuritySeverity::Warning {
        "SEC-WARN"
    } else {
        "SEC-INFO"
    };
    assessments.push(assessment(
        id,
        ThreatCategory::SecurityControl,
        risk,
        finding.message.clone(),
        "Address finding via `spanda security check` recommendations",
    ));
}

fn score_assessments(assessments: &[ThreatAssessment]) -> u32 {
    let mut score = 0u32;
    for item in assessments {
        score += match item.risk {
            ThreatRisk::Low => 5,
            ThreatRisk::Medium => 15,
            ThreatRisk::High => 30,
            ThreatRisk::Critical => 50,
        };
    }
    score.min(100)
}

fn surface(category: ThreatCategory, detail: impl Into<String>, line: Option<u32>) -> AttackSurfaceItem {
    AttackSurfaceItem {
        category,
        detail: detail.into(),
        line,
    }
}

fn assessment(
    id: &str,
    category: ThreatCategory,
    risk: ThreatRisk,
    description: impl Into<String>,
    mitigation: impl Into<String>,
) -> ThreatAssessment {
    ThreatAssessment {
        id: id.into(),
        category,
        risk,
        description: description.into(),
        mitigation: mitigation.into(),
    }
}

/// Format a threat report for CLI output.
pub fn format_threat_report(report: &ThreatReport, json: bool) -> String {
    // Serialize or pretty-print a threat model report.
    //
    // Parameters:
    // - `report` — threat model report
    // - `json` — emit JSON when true
    //
    // Returns:
    // Formatted report string.
    //
    // Options:
    // None.
    //
    // Example:
    // let text = format_threat_report(&report, false);

    if json {
        return serde_json::to_string_pretty(report).unwrap_or_else(|e| e.to_string());
    }
    let mut lines = vec![
        format!("Threat model: {}", report.program),
        format!(
            "Risk score: {}/100 — {}",
            report.risk_score,
            if report.passed { "PASS" } else { "REVIEW" }
        ),
        String::from("Attack surface:"),
    ];
    if report.attack_surface.is_empty() {
        lines.push("  (none declared)".into());
    } else {
        for item in &report.attack_surface {
            lines.push(format!("  [{:?}] {}", item.category, item.detail));
        }
    }
    lines.push(String::from("Threats:"));
    if report.assessments.is_empty() {
        lines.push("  (none identified)".into());
    } else {
        for threat in &report.assessments {
            lines.push(format!(
                "  {} [{:?}] {} — {}",
                threat.id, threat.risk, threat.description, threat.mitigation
            ));
        }
    }
    if !report.mitigations.is_empty() {
        lines.push(String::from("Recommended mitigations:"));
        for mitigation in &report.mitigations {
            lines.push(format!("  - {mitigation}"));
        }
    }
    lines.join("\n")
}
