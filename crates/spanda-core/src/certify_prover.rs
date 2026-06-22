//! Structured certification proof artifacts for CI and audit workflows.

use crate::ast::Program;
use crate::certify_verify::verify_certification_proof;
use spanda_ota::hash_program_artifact;
use crate::foundations::DeployDecl;
use crate::hardware::{CompatItem, CompatSeverity};
use crate::robotics_platform::CertifyDecl;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CertificationEntry {
    pub standard: String,
    pub level: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeployTargetEntry {
    pub robot_name: String,
    pub hardware: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificationProofSummary {
    pub passed: bool,
    pub passed_strict: bool,
    pub summary: String,
    pub error_count: u32,
    pub warning_count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CertificationProofReport {
    pub program: String,
    pub program_hash: Option<String>,
    pub strict: bool,
    pub passed: bool,
    pub certifications: Vec<CertificationEntry>,
    pub deploy_targets: Vec<DeployTargetEntry>,
    pub checklist: Vec<CompatItem>,
    pub summary: String,
}

/// Build a structured certification proof report for a parsed program.
pub fn build_certification_proof(
    program: &Program,
    program_path: &str,
    strict: bool,
) -> CertificationProofReport {
    // Aggregate certify metadata and checklist items into an audit artifact.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    // - `program_path` — source path for hashing and reporting
    // - `strict` — treat checklist gaps as proof failures
    //
    // Returns:
    // Structured proof report suitable for JSON export.
    //
    // Options:
    // None.
    //
    // Example:
    // let proof = build_certification_proof(&program, "certified.sd", true);

    let Program::Program {
        certifications,
        deployments,
        ..
    } = program;

    let checklist = verify_certification_proof(program, strict);
    let passed = !checklist
        .iter()
        .any(|item| item.severity == CompatSeverity::Error);

    let cert_entries: Vec<CertificationEntry> = certifications
        .iter()
        .map(|cert| {
            let CertifyDecl::CertifyDecl {
                standard,
                level,
                ..
            } = cert;
            CertificationEntry {
                standard: standard.as_str().to_string(),
                level: level.clone(),
            }
        })
        .collect();

    let mut deploy_targets = Vec::new();
    for deploy in deployments {
        let DeployDecl::DeployDecl {
            robot_name,
            targets,
            ..
        } = deploy;
        for hardware in targets {
            deploy_targets.push(DeployTargetEntry {
                robot_name: robot_name.clone(),
                hardware: hardware.clone(),
            });
        }
    }

    let error_count = checklist
        .iter()
        .filter(|i| i.severity == CompatSeverity::Error)
        .count();
    let warning_count = checklist
        .iter()
        .filter(|i| i.severity == CompatSeverity::Warning)
        .count();
    let summary = if passed {
        format!(
            "Certification proof passed ({} deploy targets, {} certify blocks)",
            deploy_targets.len(),
            cert_entries.len(),
        )
    } else {
        format!(
            "Certification proof failed with {error_count} error(s) and {warning_count} warning(s)"
        )
    };

    CertificationProofReport {
        program: program_path.to_string(),
        program_hash: hash_program_artifact(program_path),
        strict,
        passed,
        certifications: cert_entries,
        deploy_targets,
        checklist,
        summary,
    }
}

/// Build a compact certification summary for deploy planning and rollout gates.
pub fn build_certification_proof_summary(
    program: &Program,
    program_path: &str,
) -> CertificationProofSummary {
    // Derive non-strict and strict proof outcomes for deploy plan reporting.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    // - `program_path` — source path for proof reporting
    //
    // Returns:
    // Compact summary with both relaxed and strict pass flags.
    //
    // Options:
    // None.
    //
    // Example:
    // let summary = build_certification_proof_summary(&program, "certified.sd");

    let proof = build_certification_proof(program, program_path, false);
    let strict = build_certification_proof(program, program_path, true);
    let error_count = strict
        .checklist
        .iter()
        .filter(|item| item.severity == CompatSeverity::Error)
        .count() as u32;
    let warning_count = strict
        .checklist
        .iter()
        .filter(|item| item.severity == CompatSeverity::Warning)
        .count() as u32;
    let summary = if strict.passed {
        proof.summary
    } else {
        strict.summary
    };
    CertificationProofSummary {
        passed: proof.passed,
        passed_strict: strict.passed,
        summary,
        error_count,
        warning_count,
    }
}
