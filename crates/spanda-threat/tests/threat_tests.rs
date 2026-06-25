//! Integration tests for static threat modeling.

use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_threat::{analyze_threat_model, ThreatCategory, ThreatRisk};
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
fn readiness_rover_has_connectivity_surface() {
    let program = parse_file(repo_path(&["examples", "showcase", "readiness", "rover.sd"]));
    let report = analyze_threat_model(&program, "rover.sd");
    assert!(!report.attack_surface.is_empty());
    assert!(report
        .assessments
        .iter()
        .any(|a| a.category == ThreatCategory::Ota));
}

#[test]
fn remote_signed_kill_switch_flags_high_risk() {
    let program = parse_file(repo_path(&[
        "examples",
        "security",
        "remote_signed_kill_switch.sd",
    ]));
    let report = analyze_threat_model(&program, "remote_signed_kill_switch.sd");
    assert!(report.assessments.iter().any(|a| {
        a.category == ThreatCategory::RemoteCommand && a.risk == ThreatRisk::High
    }));
}
