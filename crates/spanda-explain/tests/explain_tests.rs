//! Integration tests for program explain reports.

use spanda_explain::{explain_program_with_options, ExplainProgramOptions};
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

#[test]
fn defense_showcase_includes_secure_boot_section() {
    let registry = repo_path(&["registry"]);
    std::env::set_var(
        "SPANDA_REGISTRY_URL",
        format!("file://{}", registry.display()),
    );
    let path = repo_path(&[
        "examples",
        "showcase",
        "compliance",
        "defense_rover.sd",
    ]);
    let source = std::fs::read_to_string(&path).unwrap();
    let program = parse(tokenize(&source).unwrap()).unwrap();
    let options = ExplainProgramOptions {
        source: Some(&source),
        ..ExplainProgramOptions::default()
    };
    let report = explain_program_with_options(
        &program,
        "compliance/defense_rover.sd",
        &options,
    );
    assert!(
        report
            .sections
            .iter()
            .any(|section| section.topic == "secure_boot"),
        "expected secure_boot section, got {:?}",
        report.sections.iter().map(|s| &s.topic).collect::<Vec<_>>()
    );
    std::env::remove_var("SPANDA_REGISTRY_URL");
}
