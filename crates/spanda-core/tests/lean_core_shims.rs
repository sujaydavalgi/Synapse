//! Guardrails for lean-core shim deprecation in `spanda-core`.
//!
use std::fs;
use std::path::Path;

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
    let runtime = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime.rs");
    let connectivity = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime_connectivity.rs");
    let runtime_source = fs::read_to_string(&runtime).expect("runtime.rs");
    let connectivity_source = fs::read_to_string(&connectivity).expect("runtime_connectivity.rs");
    assert!(
        connectivity_source.contains("fn run_geofence_triggers"),
        "runtime_connectivity.rs should own geofence trigger dispatch"
    );
    assert!(
        !runtime_source.contains("fn run_geofence_triggers"),
        "runtime.rs should delegate geofence triggers to runtime_connectivity.rs"
    );
    assert!(
        !runtime_source.contains("connectivity_positioning::apply_gps_reading_faults"),
        "runtime.rs should route GPS reading faults through RuntimeHost"
    );
}

#[test]
fn runtime_navigation_and_robot_logic_is_extracted() {
    let runtime = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime.rs");
    let navigation = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime_navigation.rs");
    let robot = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime_robot.rs");
    let runtime_source = fs::read_to_string(&runtime).expect("runtime.rs");
    let navigation_source = fs::read_to_string(&navigation).expect("runtime_navigation.rs");
    let robot_source = fs::read_to_string(&robot).expect("runtime_robot.rs");
    assert!(navigation_source.contains("fn eval_navigation_method"));
    assert!(navigation_source.contains("invoke_nav2_bridge"));
    assert!(robot_source.contains("fn eval_robot_method"));
    assert!(!runtime_source.contains("fn eval_navigation_method"));
    assert!(!runtime_source.contains("fn eval_robot_method"));
}

#[test]
fn runtime_trigger_logic_is_extracted() {
    let runtime = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime.rs");
    let triggers = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime_triggers.rs");
    let runtime_source = fs::read_to_string(&runtime).expect("runtime.rs");
    let triggers_source = fs::read_to_string(&triggers).expect("runtime_triggers.rs");
    assert!(triggers_source.contains("fn run_trigger_maintenance"));
    assert!(triggers_source.contains("fn dispatch_system_trigger"));
    assert!(!runtime_source.contains("fn run_trigger_maintenance"));
    assert!(!runtime_source.contains("fn dispatch_system_trigger"));
}

#[test]
fn runtime_robotics_sensors_and_twin_logic_is_extracted() {
    let runtime = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime.rs");
    let robotics = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime_robotics.rs");
    let sensors = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime_sensors.rs");
    let twin = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime_twin.rs");
    let runtime_source = fs::read_to_string(&runtime).expect("runtime.rs");
    let robotics_source = fs::read_to_string(&robotics).expect("runtime_robotics.rs");
    let sensors_source = fs::read_to_string(&sensors).expect("runtime_sensors.rs");
    let twin_source = fs::read_to_string(&twin).expect("runtime_twin.rs");
    assert!(robotics_source.contains("fn eval_ai_method"));
    assert!(robotics_source.contains("fn eval_safety_validate"));
    assert!(sensors_source.contains("fn read_sensor_value"));
    assert!(sensors_source.contains("fn read_fused_observation"));
    assert!(twin_source.contains("fn eval_twin_method"));
    assert!(!runtime_source.contains("fn eval_ai_method"));
    assert!(!runtime_source.contains("fn read_sensor_value"));
    assert!(!runtime_source.contains("fn eval_safety_validate"));
    assert!(!runtime_source.contains("fn eval_twin_method"));
}

#[test]
fn runtime_builtins_audit_and_actuator_logic_is_extracted() {
    let runtime = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime.rs");
    let builtins = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime_builtins.rs");
    let audit = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime_audit.rs");
    let actuators = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime_actuators.rs");
    let helpers = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime_helpers.rs");
    let runtime_source = fs::read_to_string(&runtime).expect("runtime.rs");
    let builtins_source = fs::read_to_string(&builtins).expect("runtime_builtins.rs");
    let audit_source = fs::read_to_string(&audit).expect("runtime_audit.rs");
    let actuators_source = fs::read_to_string(&actuators).expect("runtime_actuators.rs");
    let helpers_source = fs::read_to_string(&helpers).expect("runtime_helpers.rs");
    assert!(builtins_source.contains("fn eval_builtin_function"));
    assert!(audit_source.contains("fn eval_audit_method"));
    assert!(audit_source.contains("fn eval_ledger_method"));
    assert!(actuators_source.contains("fn execute_actuator_method"));
    assert!(helpers_source.contains("fn runtime_value_payload"));
    assert!(!runtime_source.contains("fn eval_builtin_function"));
    assert!(!runtime_source.contains("fn eval_audit_method"));
    assert!(!runtime_source.contains("fn execute_actuator_method"));
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
