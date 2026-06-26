//! Integration tests for compliance profile evaluation.

use spanda_compliance::{
    evaluate_compliance_profile, generate_accreditation_report, list_compliance_profiles,
};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use std::path::PathBuf;

fn repo_path(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../..");
    for part in parts {
        path.push(part);
    }
    path
}

fn parse_file(path: PathBuf) -> spanda_ast::nodes::Program {
    let source = std::fs::read_to_string(&path).unwrap();
    let tokens = tokenize(&source).unwrap();
    parse(tokens).unwrap()
}

#[test]
fn lists_builtin_profiles() {
    let profiles = list_compliance_profiles();
    assert!(profiles.iter().any(|name| name == "warehouse"));
    assert!(profiles.iter().any(|name| name == "medical"));
}

#[test]
fn warehouse_profile_passes_showcase_program() {
    let program = parse_file(repo_path(&[
        "examples",
        "showcase",
        "policy",
        "warehouse.sd",
    ]));
    let report =
        evaluate_compliance_profile(&program, "warehouse", "warehouse.sd").unwrap();
    assert!(report.passed, "{:?}", report.violations);
}

#[test]
fn medical_profile_fails_readiness_rover_without_assurance_case() {
    let program = parse_file(repo_path(&["examples", "showcase", "readiness", "rover.sd"]));
    let report = evaluate_compliance_profile(&program, "medical", "rover.sd").unwrap();
    assert!(!report.passed);
    assert!(report
        .violations
        .iter()
        .any(|violation| violation.requirement == "requires_assurance_case"));
}

#[test]
fn research_profile_passes_with_warnings_only() {
    let program = parse_file(repo_path(&["examples", "showcase", "readiness", "rover.sd"]));
    let report = evaluate_compliance_profile(&program, "research", "rover.sd").unwrap();
    assert!(report.passed);
}

#[test]
fn defense_profile_requires_secure_boot_contract() {
    let program = parse_file(repo_path(&[
        "examples",
        "showcase",
        "policy",
        "warehouse.sd",
    ]));
    let report = evaluate_compliance_profile(&program, "defense", "warehouse.sd").unwrap();
    assert!(!report.passed);
    assert!(report.violations.iter().any(|violation| {
        violation.requirement == "requires_secure_boot"
    }));
}

#[test]
fn defense_showcase_passes_profile() {
    let registry = repo_path(&["registry"]);
    std::env::set_var(
        "SPANDA_REGISTRY_URL",
        format!("file://{}", registry.display()),
    );
    let program = parse_file(repo_path(&[
        "examples",
        "showcase",
        "compliance",
        "defense_rover.sd",
    ]));
    let report =
        evaluate_compliance_profile(&program, "defense", "compliance/defense_rover.sd").unwrap();
    assert!(report.passed, "{:?}", report.violations);
    std::env::remove_var("SPANDA_REGISTRY_URL");
}

#[test]
fn medical_showcase_passes_profile() {
    let registry = repo_path(&["registry"]);
    std::env::set_var(
        "SPANDA_REGISTRY_URL",
        format!("file://{}", registry.display()),
    );
    let program = parse_file(repo_path(&[
        "examples",
        "showcase",
        "compliance",
        "medical_rover.sd",
    ]));
    let report =
        evaluate_compliance_profile(&program, "medical", "compliance/medical_rover.sd").unwrap();
    assert!(report.passed, "{:?}", report.violations);
    std::env::remove_var("SPANDA_REGISTRY_URL");
}

#[test]
fn secure_boot_showcase_satisfies_secure_boot_requirement() {
    let registry = repo_path(&["registry"]);
    std::env::set_var(
        "SPANDA_REGISTRY_URL",
        format!("file://{}", registry.display()),
    );
    let program = parse_file(repo_path(&[
        "examples",
        "showcase",
        "secure_boot",
        "rover.sd",
    ]));
    let report =
        evaluate_compliance_profile(&program, "defense", "secure_boot/rover.sd").unwrap();
    assert!(!report
        .violations
        .iter()
        .any(|violation| violation.requirement == "requires_secure_boot"));
    std::env::remove_var("SPANDA_REGISTRY_URL");
}

#[test]
fn defense_accreditation_export_includes_template_notice() {
    let registry = repo_path(&["registry"]);
    std::env::set_var(
        "SPANDA_REGISTRY_URL",
        format!("file://{}", registry.display()),
    );
    let program = parse_file(repo_path(&[
        "examples",
        "showcase",
        "compliance",
        "defense_rover.sd",
    ]));
    let report =
        generate_accreditation_report(&program, "defense", "defense_rover.sd").unwrap();
    assert_eq!(report.accreditation_status, "template_only");
    assert!(report.evidence_checklist.iter().any(|item| item.id == "secure_boot"));
    assert!(!report.audit_export_id.is_empty());
    std::env::remove_var("SPANDA_REGISTRY_URL");
}
