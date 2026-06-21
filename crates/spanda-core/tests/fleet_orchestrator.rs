//! Fleet orchestrator integration tests.

use spanda_core::{check, compile, orchestrate_fleets};

#[test]
fn orchestrates_robotics_fleet_example() {
    let source = include_str!("../../../examples/robotics/fleet_management.sd");
    check(source).expect("fleet example should type-check");
    let program = compile(source).expect("compile").program;
    let result = orchestrate_fleets(&program, "fleet_management.sd");
    assert!(result.success);
    assert_eq!(result.fleets.len(), 1);
    assert_eq!(result.fleets[0].fleet_name, "Warehouse");
    assert_eq!(result.fleets[0].members.len(), 2);
}
