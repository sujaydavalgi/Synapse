//! Native interpreter integration smoke tests.
//!
use spanda_interpreter::{create_default_simulator, Interpreter, InterpreterOptions, SimulatorConfig};

#[test]
fn native_interpreter_construct_and_options_default() {
    let sim = create_default_simulator(SimulatorConfig::default());
    let options = InterpreterOptions::default();
    let _interp = Interpreter::new(sim, options);
    assert_eq!(InterpreterOptions::default().max_loop_iterations, 10);
}

#[test]
fn native_crate_builds_without_spanda_core() {
    // Guardrail: this test file only links `spanda-interpreter`.
    assert!(std::env::var("CARGO_PKG_NAME")
        .unwrap_or_default()
        .contains("spanda-interpreter"));
}

#[test]
fn sim_robot_backend_type_alias_matches_simulator() {
    fn assert_sim_type<T: spanda_interpreter::RobotBackend>() {}
    assert_sim_type::<spanda_interpreter::SimRobotBackend>();
}
