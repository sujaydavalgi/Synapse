//! Integration tests for scorecard rollup.

use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_score::{evaluate_scorecard, ScorecardOptions};
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
fn readiness_rover_produces_scorecard() {
    let program = parse_file(repo_path(&["examples", "showcase", "readiness", "rover.sd"]));
    let report = evaluate_scorecard(&program, "rover.sd", &ScorecardOptions::default());
    assert_eq!(report.categories.len(), 7);
    assert!(report.overall_score > 0);
    assert!(!report.tier.is_empty());
}
