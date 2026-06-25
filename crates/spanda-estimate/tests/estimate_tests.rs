//! Integration tests for mission resource estimation.

use spanda_estimate::{estimate_mission, EstimateOptions};
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
fn mission_duration_feature_estimates_half_hour() {
    let program = parse_file(repo_path(&["examples", "features", "mission_duration.sd"]));
    let report = estimate_mission(&program, "mission_duration.sd", &EstimateOptions::default());
    let duration = report
        .resources
        .iter()
        .find(|resource| resource.resource == "duration")
        .expect("duration estimate");
    assert!((duration.value - 0.5).abs() < 0.01);
}

#[test]
fn hardware_compatibility_showcase_within_battery_budget() {
    let program = parse_file(repo_path(&[
        "examples",
        "showcase",
        "hardware_compatibility.sd",
    ]));
    let report = estimate_mission(
        &program,
        "hardware_compatibility.sd",
        &EstimateOptions {
            target: Some("RoverV1".into()),
        },
    );
    assert_eq!(report.target.as_deref(), Some("RoverV1"));
    assert!(report.within_budget, "{:?}", report.resources);
}
