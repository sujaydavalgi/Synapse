//! Guardrails for lean-core shim deprecation in `spanda-core`.
//!
use std::fs;
use std::path::{Path, PathBuf};

fn interpreter_runtime_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../spanda-interpreter/src/runtime")
}

fn runtime_shim_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime.rs")
}

fn orchestrator_path() -> PathBuf {
    interpreter_runtime_dir().join("orchestrator.rs")
}

#[test]
fn runtime_shim_stays_thin() {
    let source = fs::read_to_string(runtime_shim_path()).expect("runtime.rs shim");
    let lines = source.lines().count();
    assert!(
        lines <= 12,
        "runtime.rs should be a thin include shim (got {lines} lines)"
    );
    assert!(
        source.contains("spanda-interpreter/src/runtime/orchestrator.rs"),
        "runtime shim should include orchestrator from spanda-interpreter"
    );
}

#[test]
fn interpreter_sources_live_in_interpreter_crate() {
    let orchestrator = orchestrator_path();
    assert!(
        orchestrator.exists(),
        "orchestrator.rs should live under crates/spanda-interpreter/src/runtime/"
    );
    let eval = interpreter_runtime_dir().join("runtime_eval.rs");
    assert!(eval.exists(), "runtime_eval.rs should live in spanda-interpreter tree");
}

#[test]
fn interpreter_runtime_uses_workspace_ast_paths() {
    let orchestrator = fs::read_to_string(orchestrator_path()).expect("orchestrator.rs");
    assert!(
        orchestrator.contains("spanda_ast::nodes::"),
        "orchestrator should import AST nodes from spanda-ast"
    );
    assert!(
        orchestrator.contains("spanda_ast::foundations::"),
        "orchestrator should import foundation decls from spanda-ast"
    );
    assert!(
        !orchestrator.contains("crate::ast::"),
        "orchestrator should not use crate::ast after Phase 8 routing"
    );
}

#[test]
fn error_shim_reexports_spanda_error() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/error.rs");
    let source = fs::read_to_string(&path).expect("error.rs");
    assert!(
        source.contains("spanda_error"),
        "error.rs should re-export SpandaError from spanda-error"
    );
    assert!(
        source.contains("RunOptions"),
        "error.rs should retain RunOptions and related run API types in core"
    );
}

#[test]
fn hal_shim_reexports_spanda_hal() {
    for module in ["hal.rs", "hardware_monitor.rs", "soc.rs"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(module);
        let source = fs::read_to_string(&path).expect(module);
        assert!(
            source.lines().count() <= 8,
            "{module} should be a thin re-export shim"
        );
        assert!(
            source.contains("spanda_hal"),
            "{module} shim should re-export from spanda-hal"
        );
    }
}

#[test]
fn safety_shim_reexports_spanda_safety() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/safety.rs");
    let source = fs::read_to_string(&path).expect("safety.rs");
    assert!(
        source.lines().count() <= 8,
        "safety.rs should be a thin re-export shim"
    );
    assert!(
        source.contains("spanda_safety"),
        "safety shim should re-export from spanda-safety"
    );
}

#[test]
fn comm_shim_reexports_spanda_comm() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/comm.rs");
    let source = fs::read_to_string(&path).expect("comm.rs");
    assert!(
        source.lines().count() <= 8,
        "comm.rs should be a thin re-export shim (got {} lines)",
        source.lines().count()
    );
    assert!(
        source.contains("spanda_comm"),
        "comm shim should re-export from spanda-comm"
    );
}

#[test]
fn runtime_kernel_modules_reexport_from_spanda_runtime() {
    for (module, export) in [
        ("telemetry.rs", "spanda_runtime::telemetry"),
        ("replay.rs", "spanda_runtime::replay"),
        ("twin.rs", "spanda_runtime::twin"),
        ("events.rs", "spanda_runtime::events"),
        ("state_machine.rs", "spanda_runtime::state_machine"),
        ("reliability_runtime.rs", "spanda_runtime::reliability_runtime"),
        ("serialize.rs", "spanda_runtime::serialize"),
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(module);
        let source = fs::read_to_string(&path).expect(module);
        assert!(
            source.lines().count() <= 8,
            "{module} should stay a thin re-export shim"
        );
        assert!(
            source.contains(export),
            "{module} should re-export from spanda-runtime"
        );
    }
}

#[test]
fn triggers_shim_reexports_spanda_runtime() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/triggers.rs");
    let source = fs::read_to_string(&path).expect("triggers.rs");
    let lines = source.lines().count();
    assert!(lines <= 8, "triggers.rs should be a thin re-export shim (got {lines} lines)");
    assert!(
        source.contains("spanda_runtime::triggers"),
        "triggers shim should re-export from spanda-runtime"
    );
}

#[test]
fn interpreter_runtime_uses_workspace_security_and_scheduler() {
    let orchestrator = fs::read_to_string(orchestrator_path()).expect("orchestrator.rs");
    assert!(
        orchestrator.contains("spanda_security::SecurityContext"),
        "orchestrator should import security from spanda-security"
    );
    assert!(
        orchestrator.contains("spanda_runtime::scheduler::SchedulerClock"),
        "orchestrator should import scheduler from spanda-runtime"
    );
    assert!(
        orchestrator.contains("spanda_runtime::robot_state::"),
        "orchestrator should import robot state from spanda-runtime"
    );
}

#[test]
fn transport_live_shim_stays_thin() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/transport_live.rs");
    let source = fs::read_to_string(&path).expect("transport_live.rs");
    let lines = source.lines().count();
    assert!(
        lines <= 80,
        "transport_live.rs should remain a thin shim (got {lines} lines); move logic to spanda-transport-*"
    );
    assert!(
        source.contains("spanda_transport_ros2::live_bridge"),
        "transport_live shim should delegate ROS2 live hooks to spanda-transport-ros2"
    );
    assert!(
        source.contains("spanda_transport_mqtt"),
        "transport_live shim should delegate MQTT live hooks to spanda-transport-mqtt"
    );
}

#[test]
fn transport_routing_shim_reexports_spanda_transport() {
    for module in ["transport.rs", "transport_wire.rs"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(module);
        let source = fs::read_to_string(&path).expect(module);
        assert!(
            source.lines().count() <= 15,
            "{module} should be a thin re-export shim"
        );
        assert!(
            source.contains("spanda_transport_routing") || source.contains("spanda_transport"),
            "{module} shim should re-export from spanda-transport routing stack"
        );
    }
}

#[test]
fn transport_no_inline_adapter_impls() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/transport.rs");
    let source = fs::read_to_string(&path).expect("transport.rs");
    assert!(
        !source.contains("impl TransportAdapter for Ros2"),
        "transport.rs must not define TransportAdapter impls; use spanda-transport-* crates"
    );
}

#[test]
fn transport_live_no_direct_python_bridge() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/transport_live.rs");
    let source = fs::read_to_string(&path).expect("transport_live.rs");
    assert!(
        !source.contains("call_subprocess_bridge"),
        "transport_live should not invoke the Python bridge directly"
    );
    assert!(
        !source.contains("bridge_script_path"),
        "transport_live should not resolve bridge script paths directly"
    );
}

#[test]
fn runtime_connectivity_logic_is_extracted() {
    let orchestrator = fs::read_to_string(orchestrator_path()).expect("orchestrator.rs");
    let connectivity =
        fs::read_to_string(interpreter_runtime_dir().join("runtime_connectivity.rs"))
            .expect("runtime_connectivity.rs");
    assert!(connectivity.contains("fn run_geofence_triggers"));
    assert!(!orchestrator.contains("fn run_geofence_triggers"));
    assert!(!orchestrator.contains("connectivity_positioning::apply_gps_reading_faults"));
}

#[test]
fn runtime_navigation_and_robot_logic_is_extracted() {
    let orchestrator = fs::read_to_string(orchestrator_path()).expect("orchestrator.rs");
    let navigation = fs::read_to_string(interpreter_runtime_dir().join("runtime_navigation.rs"))
        .expect("runtime_navigation.rs");
    let robot = fs::read_to_string(interpreter_runtime_dir().join("runtime_robot.rs"))
        .expect("runtime_robot.rs");
    assert!(navigation.contains("fn eval_navigation_method"));
    assert!(navigation.contains("invoke_nav2_bridge"));
    assert!(robot.contains("fn eval_robot_method"));
    assert!(!orchestrator.contains("fn eval_navigation_method"));
    assert!(!orchestrator.contains("fn eval_robot_method"));
}

#[test]
fn runtime_trigger_logic_is_extracted() {
    let orchestrator = fs::read_to_string(orchestrator_path()).expect("orchestrator.rs");
    let triggers = fs::read_to_string(interpreter_runtime_dir().join("runtime_triggers.rs"))
        .expect("runtime_triggers.rs");
    assert!(triggers.contains("fn run_trigger_maintenance"));
    assert!(triggers.contains("fn dispatch_system_trigger"));
    assert!(!orchestrator.contains("fn run_trigger_maintenance"));
    assert!(!orchestrator.contains("fn dispatch_system_trigger"));
}

#[test]
fn runtime_robotics_sensors_and_twin_logic_is_extracted() {
    let orchestrator = fs::read_to_string(orchestrator_path()).expect("orchestrator.rs");
    let dir = interpreter_runtime_dir();
    let robotics = fs::read_to_string(dir.join("runtime_robotics.rs")).expect("runtime_robotics.rs");
    let sensors = fs::read_to_string(dir.join("runtime_sensors.rs")).expect("runtime_sensors.rs");
    let twin = fs::read_to_string(dir.join("runtime_twin.rs")).expect("runtime_twin.rs");
    assert!(robotics.contains("fn eval_ai_method"));
    assert!(robotics.contains("fn eval_safety_validate"));
    assert!(sensors.contains("fn read_sensor_value"));
    assert!(sensors.contains("fn read_fused_observation"));
    assert!(twin.contains("fn eval_twin_method"));
    assert!(!orchestrator.contains("fn eval_ai_method"));
    assert!(!orchestrator.contains("fn read_sensor_value"));
    assert!(!orchestrator.contains("fn eval_safety_validate"));
    assert!(!orchestrator.contains("fn eval_twin_method"));
}

#[test]
fn runtime_builtins_audit_and_actuator_logic_is_extracted() {
    let orchestrator = fs::read_to_string(orchestrator_path()).expect("orchestrator.rs");
    let dir = interpreter_runtime_dir();
    let builtins = fs::read_to_string(dir.join("runtime_builtins.rs")).expect("runtime_builtins.rs");
    let audit = fs::read_to_string(dir.join("runtime_audit.rs")).expect("runtime_audit.rs");
    let actuators =
        fs::read_to_string(dir.join("runtime_actuators.rs")).expect("runtime_actuators.rs");
    let helpers = fs::read_to_string(dir.join("runtime_helpers.rs")).expect("runtime_helpers.rs");
    assert!(builtins.contains("fn eval_builtin_function"));
    assert!(audit.contains("fn eval_audit_method"));
    assert!(audit.contains("fn eval_ledger_method"));
    assert!(actuators.contains("fn execute_actuator_method"));
    assert!(helpers.contains("fn runtime_value_payload"));
    assert!(!orchestrator.contains("fn eval_builtin_function"));
    assert!(!orchestrator.contains("fn eval_audit_method"));
    assert!(!orchestrator.contains("fn execute_actuator_method"));
}

#[test]
fn runtime_eval_logic_is_extracted() {
    let orchestrator = fs::read_to_string(orchestrator_path()).expect("orchestrator.rs");
    let eval = fs::read_to_string(interpreter_runtime_dir().join("runtime_eval.rs"))
        .expect("runtime_eval.rs");
    assert!(eval.contains("fn eval_expr"));
    assert!(eval.contains("fn eval_call"));
    assert!(eval.contains("fn eval_binary"));
    assert!(eval.contains("fn get_named_arg_value"));
    assert!(!orchestrator.contains("fn eval_expr"));
    assert!(!orchestrator.contains("fn eval_call"));
    assert!(!orchestrator.contains("fn eval_binary"));
}

#[test]
fn runtime_spawn_logic_is_extracted() {
    let orchestrator = fs::read_to_string(orchestrator_path()).expect("orchestrator.rs");
    let spawn = fs::read_to_string(interpreter_runtime_dir().join("runtime_spawn.rs"))
        .expect("runtime_spawn.rs");
    assert!(spawn.contains("fn resolve_future"));
    assert!(spawn.contains("fn process_spawn_queue"));
    assert!(spawn.contains("fn eval_spawn_target"));
    assert!(!orchestrator.contains("fn resolve_future"));
    assert!(!orchestrator.contains("fn process_spawn_queue"));
}

#[test]
fn runtime_execute_and_scheduler_logic_is_extracted() {
    let orchestrator = fs::read_to_string(orchestrator_path()).expect("orchestrator.rs");
    let dir = interpreter_runtime_dir();
    let execute = fs::read_to_string(dir.join("runtime_execute.rs")).expect("runtime_execute.rs");
    let scheduler =
        fs::read_to_string(dir.join("runtime_scheduler.rs")).expect("runtime_scheduler.rs");
    let setup = fs::read_to_string(dir.join("runtime_setup.rs")).expect("runtime_setup.rs");
    assert!(execute.contains("fn execute_stmt"));
    assert!(scheduler.contains("fn execute_multiplexed_tasks"));
    assert!(setup.contains("fn setup_robot"));
    assert!(!orchestrator.contains("fn execute_stmt"));
    assert!(!orchestrator.contains("fn execute_multiplexed_tasks"));
    assert!(!orchestrator.contains("fn setup_robot("));
    let lines = orchestrator.lines().count();
    assert!(
        lines <= 1850,
        "orchestrator.rs should stay orchestration-only (got {lines} lines)"
    );
}

#[test]
fn interpreter_accepts_injected_runtime_host() {
    use spanda_core::runtime::{Interpreter, InterpreterOptions};
    use spanda_core::simulator::{create_default_simulator, SimulatorConfig};
    use spanda_runtime::RuntimeHost;

    struct StubHost;

    impl RuntimeHost for StubHost {
        fn slam_import_known(&self, _path: &str) -> bool {
            false
        }

        fn navigation_import_known(&self, _path: &str) -> bool {
            false
        }
    }

    static STUB: StubHost = StubHost;
    let interp = Interpreter::new(
        create_default_simulator(SimulatorConfig::default()),
        InterpreterOptions {
            runtime_host: Some(&STUB),
            ..Default::default()
        },
    );
    assert!(std::ptr::eq(
        interp.runtime_host() as *const dyn RuntimeHost,
        &STUB as *const StubHost as *const dyn RuntimeHost,
    ));
}
