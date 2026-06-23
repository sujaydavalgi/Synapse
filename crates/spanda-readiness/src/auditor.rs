//! Autonomous safety auditor for deployment readiness gaps.

use serde::{Deserialize, Serialize};
use spanda_ast::foundations::KillSwitchDecl;
use spanda_ast::nodes::Program;
use spanda_capability::{check_minimum_capabilities, collect_verification_diagnostics};
use spanda_security::validate::security_check;

use crate::types::ReadinessSeverity;

/// Single audit finding with severity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditFinding {
    pub severity: ReadinessSeverity,
    pub category: String,
    pub message: String,
    pub line: u32,
    pub column: u32,
}

/// Safety audit report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafetyAuditReport {
    pub findings: Vec<AuditFinding>,
    pub critical_count: u32,
    pub high_count: u32,
    pub medium_count: u32,
    pub low_count: u32,
}

/// Run autonomous safety audit on a program.
pub fn audit_program(program: &Program, source: &str) -> SafetyAuditReport {
    let mut findings = Vec::new();
    let Program::Program {
        robots,
        kill_switches,
        health_checks,
        ..
    } = program;

    if kill_switches.is_empty() {
        findings.push(finding(
            ReadinessSeverity::Critical,
            "kill-switch",
            "Missing kill switch declaration",
            1,
            1,
        ));
    }
    for ks in kill_switches {
        let KillSwitchDecl::KillSwitchDecl {
            remote_signed,
            name,
            span,
            ..
        } = ks;
        if !remote_signed {
            findings.push(finding(
                ReadinessSeverity::High,
                "kill-switch",
                format!("Kill switch '{name}' is not signed"),
                span.start.line,
                span.start.column,
            ));
        }
    }

    if health_checks.is_empty() {
        findings.push(finding(
            ReadinessSeverity::Medium,
            "health",
            "Missing health check declarations",
            1,
            1,
        ));
    }

    let minimum = check_minimum_capabilities(program);
    if !minimum.compatible {
        for err in &minimum.errors {
            findings.push(finding(
                ReadinessSeverity::High,
                "capability",
                err.clone(),
                1,
                1,
            ));
        }
    }

    for robot in robots {
        let spanda_ast::nodes::RobotDecl::RobotDecl {
            name,
            safety,
            modes,
            ..
        } = robot;
        if safety.is_none() {
            findings.push(finding(
                ReadinessSeverity::High,
                "safety",
                format!("Robot '{name}' missing safety block"),
                1,
                1,
            ));
        }
        if modes.is_empty() {
            findings.push(finding(
                ReadinessSeverity::Medium,
                "fallback",
                format!("Robot '{name}' has no fallback/degraded mode"),
                1,
                1,
            ));
        }
    }

    for diag in collect_verification_diagnostics(program) {
        let sev = match diag.severity.as_str() {
            "error" => ReadinessSeverity::Critical,
            "warning" => ReadinessSeverity::Medium,
            _ => ReadinessSeverity::Low,
        };
        findings.push(finding(
            sev,
            diag.category,
            diag.message,
            diag.line,
            diag.column,
        ));
    }

    if let Ok(sec) = security_check(source) {
        for f in sec.findings {
            let sev = match f.severity {
                spanda_security::validate::SecuritySeverity::Error => ReadinessSeverity::Critical,
                spanda_security::validate::SecuritySeverity::Warning => ReadinessSeverity::High,
                spanda_security::validate::SecuritySeverity::Info => ReadinessSeverity::Low,
            };
            findings.push(finding(sev, "security", f.message, f.line, f.column));
        }
    }

    let critical_count = findings
        .iter()
        .filter(|f| f.severity == ReadinessSeverity::Critical)
        .count() as u32;
    let high_count = findings
        .iter()
        .filter(|f| f.severity == ReadinessSeverity::High)
        .count() as u32;
    let medium_count = findings
        .iter()
        .filter(|f| f.severity == ReadinessSeverity::Medium)
        .count() as u32;
    let low_count = findings
        .iter()
        .filter(|f| f.severity == ReadinessSeverity::Low)
        .count() as u32;

    SafetyAuditReport {
        findings,
        critical_count,
        high_count,
        medium_count,
        low_count,
    }
}

fn finding(
    severity: ReadinessSeverity,
    category: impl Into<String>,
    message: impl Into<String>,
    line: u32,
    column: u32,
) -> AuditFinding {
    AuditFinding {
        severity,
        category: category.into(),
        message: message.into(),
        line,
        column,
    }
}

/// Audit from source.
pub fn audit_program_source(source: &str) -> Result<SafetyAuditReport, spanda_error::SpandaError> {
    let tokens = spanda_lexer::tokenize(source)?;
    let program = spanda_parser::parse(tokens)?;
    Ok(audit_program(&program, source))
}
