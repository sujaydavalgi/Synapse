//! Runtime provider registry wiring tests.
//!
use spanda_core::compile;
use spanda_core::runtime::{Interpreter, InterpreterOptions};
use spanda_core::simulator::{create_default_simulator, SimulatorConfig};

#[test]
fn interpreter_bootstraps_provider_registry_by_default() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  behavior main() {
    log("ok");
  }
}
"#;
    let program = compile(source).expect("compile").program;
    let sim = create_default_simulator(SimulatorConfig::default());
    let mut interp = Interpreter::new(sim, InterpreterOptions::default());
    assert!(interp.provider_registry().transport_count() >= 2);
    let _ = interp.run(&program, None);
}
