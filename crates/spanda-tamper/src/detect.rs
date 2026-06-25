//! Verify-time tamper detection composing threat, audit, and security engines.

use serde::{Deserialize, Serialize};
use spanda_ast::foundations::DeployDecl;
use spanda_ast::nodes::{ImportDecl, Program, RobotDecl};
use spanda_readiness::{audit_program, ReadinessSeverity};
use spanda_security::{security_analyze_program, SecuritySeverity};
use spanda_threat::{analyze_threat_model, ThreatRisk};

/// Output format for tamper reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TamperFormat {
    #[default]
    Text,
    Json,
}

/// Tamper severity aligned with platform maturity taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TamperSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Overall trust posture for a program artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TamperStatus {
    Trusted,
    Suspicious,
    Tampered,
    Compromised,
    Unknown,
}

/// One tamper or integrity finding with supporting context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TamperFinding {
    pub category: String,
    pub severity: TamperSeverity,
    pub message: String,
    pub evidence: Option<String>,
    pub line: Option<u32>,
}

/// Full verify-time tamper analysis report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TamperReport {
    pub program: String,
    pub status: TamperStatus,
    pub trust_score: u32,
    pub findings: Vec<TamperFinding>,
    pub passed: bool,
}

/// Run verify-time tamper analysis on a parsed program.
pub fn generate_tamper_check(program: &Program, source_label: &str) -> TamperReport {
    // Compose threat, audit, security, and structural integrity signals.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    // - `source_label` — file label for the report
    //
    // Returns:
    // Tamper report with status, trust score, and findings.
    //
    // Options:
    // None.
    //
    // Example:
    // let report = generate_tamper_check(&program, "rover.sd");

    let mut findings = Vec::new();
    let threat = analyze_threat_model(program, source_label);
    let audit = audit_program(program, source_label);
    let security = security_analyze_program(program);

    for assessment in &threat.assessments {
        if assessment.risk >= ThreatRisk::Medium {
            findings.push(TamperFinding {
                category: "threat_model".into(),
                severity: threat_risk_to_severity(assessment.risk),
                message: assessment.description.clone(),
                evidence: Some(format!("{} — {}", assessment.id, assessment.mitigation)),
                line: None,
            });
        }
    }

    for finding in &audit.findings {
        if matches!(
            finding.severity,
            ReadinessSeverity::Critical | ReadinessSeverity::High | ReadinessSeverity::Medium
        ) {
            findings.push(TamperFinding {
                category: "safety_audit".into(),
                severity: readiness_to_severity(finding.severity),
                message: finding.message.clone(),
                evidence: Some(finding.category.clone()),
                line: Some(finding.line),
            });
        }
    }

    for finding in &security.findings {
        findings.push(TamperFinding {
            category: "security".into(),
            severity: security_to_severity(finding.severity),
            message: finding.message.clone(),
            evidence: None,
            line: Some(finding.line),
        });
    }

    collect_structural_findings(program, &mut findings);

    let trust_score = compute_trust_score(&findings);
    let status = derive_status(&findings, trust_score);
    let passed = matches!(
        status,
        TamperStatus::Trusted | TamperStatus::Suspicious
    );

    TamperReport {
        program: source_label.into(),
        status,
        trust_score,
        findings,
        passed,
    }
}

/// Format a tamper report for CLI output.
pub fn format_tamper_report(report: &TamperReport, format: TamperFormat) -> String {
    // Render tamper report as JSON or human-readable text.
    //
    // Parameters:
    // - `report` — tamper analysis report
    // - `format` — text or JSON output
    //
    // Returns:
    // Formatted report string.
    //
    // Options:
    // None.
    //
    // Example:
    // let text = format_tamper_report(&report, TamperFormat::Text);

    if format == TamperFormat::Json {
        return serde_json::to_string_pretty(report).unwrap_or_else(|e| e.to_string());
    }

    let mut lines = vec![
        format!("Tamper check: {} — {:?}", report.program, report.status),
        format!("Trust score: {}/100", report.trust_score),
        if report.passed {
            "Result: PASS".into()
        } else {
            "Result: FAIL".into()
        },
    ];
    if report.findings.is_empty() {
        lines.push("No tamper indicators.".into());
    } else {
        lines.push("Findings:".into());
        for finding in &report.findings {
            let evidence = finding
                .evidence
                .as_deref()
                .map(|value| format!(" ({value})"))
                .unwrap_or_default();
            let location = finding
                .line
                .map(|line| format!(" @ line {line}"))
                .unwrap_or_default();
            lines.push(format!(
                "  [{:?}] {} — {}{}{}",
                finding.severity, finding.category, finding.message, evidence, location
            ));
        }
    }
    lines.join("\n")
}

fn collect_structural_findings(program: &Program, findings: &mut Vec<TamperFinding>) {
    let Program::Program {
        imports,
        deployments,
        certifications,
        robots,
        ..
    } = program;

    if !deployments.is_empty() && certifications.is_empty() {
        findings.push(TamperFinding {
            category: "integrity".into(),
            severity: TamperSeverity::Medium,
            message: "Deploy targets declared without certify block for baseline hashing".into(),
            evidence: Some("mission/config tampering risk".into()),
            line: None,
        });
    }

    for deployment in deployments {
        let DeployDecl::DeployDecl {
            robot_name,
            targets,
            span,
            ..
        } = deployment;
        if targets.is_empty() {
            findings.push(TamperFinding {
                category: "integrity".into(),
                severity: TamperSeverity::High,
                message: format!("Deploy for robot '{robot_name}' has no signed targets"),
                evidence: Some("unsigned OTA channel".into()),
                line: Some(span.start.line),
            });
        }
    }

    for import in imports {
        let ImportDecl::ImportDecl { path, span } = import;
        findings.push(TamperFinding {
            category: "package".into(),
            severity: TamperSeverity::Medium,
            message: format!("Third-party package `{path}` requires trust verification"),
            evidence: Some("run spanda trust before deploy".into()),
            line: Some(span.start.line),
        });
    }

    for robot in robots {
        collect_robot_integrity(robot, findings);
    }
}

fn collect_robot_integrity(robot: &RobotDecl, findings: &mut Vec<TamperFinding>) {
    let RobotDecl::RobotDecl {
        name,
        audit,
        signed_records,
        secure_comm,
        requires_network,
        ..
    } = robot;

    if requires_network.is_some() && secure_comm.is_none() {
        findings.push(TamperFinding {
            category: "integrity".into(),
            severity: TamperSeverity::High,
            message: format!("Robot '{name}' requires network without secure_comm policy"),
            evidence: Some("remote command tampering risk".into()),
            line: None,
        });
    }

    if audit.is_none() && signed_records.is_empty() {
        findings.push(TamperFinding {
            category: "integrity".into(),
            severity: TamperSeverity::Low,
            message: format!("Robot '{name}' has no audit baseline or signed records"),
            evidence: Some("config drift detection unavailable".into()),
            line: None,
        });
    }
}

fn threat_risk_to_severity(risk: ThreatRisk) -> TamperSeverity {
    match risk {
        ThreatRisk::Low => TamperSeverity::Low,
        ThreatRisk::Medium => TamperSeverity::Medium,
        ThreatRisk::High => TamperSeverity::High,
        ThreatRisk::Critical => TamperSeverity::Critical,
    }
}

fn readiness_to_severity(severity: ReadinessSeverity) -> TamperSeverity {
    match severity {
        ReadinessSeverity::Info => TamperSeverity::Info,
        ReadinessSeverity::Low => TamperSeverity::Low,
        ReadinessSeverity::Medium => TamperSeverity::Medium,
        ReadinessSeverity::High => TamperSeverity::High,
        ReadinessSeverity::Critical => TamperSeverity::Critical,
    }
}

fn security_to_severity(severity: SecuritySeverity) -> TamperSeverity {
    match severity {
        SecuritySeverity::Info => TamperSeverity::Info,
        SecuritySeverity::Warning => TamperSeverity::Medium,
        SecuritySeverity::Error => TamperSeverity::High,
    }
}

fn compute_trust_score(findings: &[TamperFinding]) -> u32 {
    let mut score = 100u32;
    for finding in findings {
        let penalty = match finding.severity {
            TamperSeverity::Info => 1,
            TamperSeverity::Low => 3,
            TamperSeverity::Medium => 8,
            TamperSeverity::High => 18,
            TamperSeverity::Critical => 35,
        };
        score = score.saturating_sub(penalty);
    }
    score
}

fn derive_status(findings: &[TamperFinding], trust_score: u32) -> TamperStatus {
    if findings.is_empty() {
        return TamperStatus::Trusted;
    }

    let has_critical = findings
        .iter()
        .any(|finding| finding.severity == TamperSeverity::Critical);
    let high_count = findings
        .iter()
        .filter(|finding| finding.severity == TamperSeverity::High)
        .count();

    if has_critical {
        return TamperStatus::Compromised;
    }
    if high_count >= 3 || trust_score < 40 {
        return TamperStatus::Tampered;
    }
    if high_count > 0 || trust_score < 75 {
        return TamperStatus::Suspicious;
    }
    TamperStatus::Trusted
}
