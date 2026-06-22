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
fn runtime_shim_reexports_spanda_interpreter() {
    let source = fs::read_to_string(runtime_shim_path()).expect("runtime.rs shim");
    let lines = source.lines().count();
    assert!(
        lines <= 8,
        "runtime.rs should be a thin re-export shim (got {lines} lines)"
    );
    assert!(
        source.contains("spanda_interpreter::runtime"),
        "runtime shim should re-export spanda_interpreter::runtime"
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
fn providers_bootstrap_shim_reexports_spanda_providers() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/providers.rs");
    let source = fs::read_to_string(&path).expect("providers.rs");
    assert!(
        source.lines().count() <= 22,
        "providers.rs should be a facade re-export shim"
    );
    assert!(
        source.contains("spanda_providers"),
        "providers facade should re-export from spanda-providers"
    );
    assert!(
        source.contains("spanda_runtime::providers"),
        "providers facade should re-export runtime provider traits"
    );
}

#[test]
fn concurrency_shim_reexports_spanda_concurrency() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/concurrency.rs");
    let source = fs::read_to_string(&path).expect("concurrency.rs");
    assert!(source.lines().count() <= 8);
    assert!(source.contains("spanda_concurrency"));
}

#[test]
fn debug_shim_reexports_spanda_debug() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/debug.rs");
    let source = fs::read_to_string(&path).expect("debug.rs");
    assert!(source.lines().count() <= 8);
    assert!(source.contains("spanda_debug"));
}

#[test]
fn final_phase8_shims_reexport_workspace_crates() {
    for (module, crate_name) in [
        ("regex_lang.rs", "spanda_regex_lang"),
        ("lib_registry.rs", "spanda_lib_registry"),
        ("connectivity_positioning.rs", "spanda_connectivity_runtime"),
        ("runtime_host.rs", "spanda_runtime_host"),
        ("nav2_adapter.rs", "spanda_runtime_host"),
        ("slam_adapter.rs", "spanda_runtime_host"),
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(module);
        let source = fs::read_to_string(&path).expect(module);
        assert!(
            source.lines().count() <= 8,
            "{module} should be a thin re-export shim"
        );
        assert!(
            source.contains(crate_name),
            "{module} shim should re-export from {crate_name}"
        );
    }
}

#[test]
fn ffi_shim_reexports_spanda_ffi_with_core_bridges() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ffi.rs");
    let source = fs::read_to_string(&path).expect("ffi.rs");
    assert!(source.contains("spanda_ffi"));
    assert!(
        source.contains("spanda_bridge") && source.contains("new_with_core_bridges"),
        "ffi shim should delegate bridge wiring to spanda-bridge"
    );
    assert!(
        source.lines().count() <= 6,
        "ffi.rs should be a thin re-export shim"
    );
}

#[test]
fn interpreter_runtime_has_no_crate_colon_imports() {
    let dir = interpreter_runtime_dir();
    for entry in fs::read_dir(&dir).expect("runtime dir") {
        let path = entry.expect("entry").path();
        if path.extension().is_some_and(|e| e == "rs") {
            let source = fs::read_to_string(&path).expect("runtime source");
            assert!(
                !source.contains("crate::"),
                "{} should not import via crate:: after Phase 8 routing",
                path.file_name().unwrap().to_string_lossy()
            );
        }
    }
}

#[test]
fn ai_shim_reexports_spanda_ai() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ai.rs");
    let source = fs::read_to_string(&path).expect("ai.rs");
    assert!(
        source.lines().count() <= 8,
        "ai.rs should be a thin re-export shim"
    );
    assert!(
        source.contains("spanda_ai"),
        "ai shim should re-export from spanda-ai"
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
    let lib = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs"))
        .expect("lib.rs");
    assert!(
        lib.contains("RunOptions"),
        "core facade should re-export RunOptions from spanda-driver"
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
        lines <= 8,
        "transport_live.rs should remain a thin shim (got {lines} lines)"
    );
    assert!(
        source.contains("spanda_transport_routing"),
        "transport_live shim should re-export from spanda-transport-routing"
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
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../spanda-transport-routing/src/transport_live.rs");
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

#[test]
fn parser_shim_reexports_spanda_parser() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/parser.rs");
    let source = fs::read_to_string(path).expect("parser.rs shim");
    assert!(source.lines().count() <= 5, "parser.rs should be a thin re-export shim");
    assert!(source.contains("spanda_parser::parse"));
}

#[test]
fn compile_pipeline_lives_in_spanda_driver() {
    let driver = Path::new(env!("CARGO_MANIFEST_DIR")).join("../spanda-driver/src/compile.rs");
    assert!(driver.exists(), "compile pipeline should live in spanda-driver");
    let source = fs::read_to_string(driver).expect("spanda-driver compile.rs");
    assert!(source.contains("spanda_lexer::tokenize"));
    assert!(source.contains("spanda_parser::parse"));
    assert!(source.contains("spanda_typecheck::"));
}

#[test]
fn facade_pipeline_lives_in_spanda_driver() {
    for module in [
        "../spanda-driver/src/verify.rs",
        "../spanda-driver/src/pipeline.rs",
        "../spanda-driver/src/replay.rs",
        "../spanda-driver/src/debug_run.rs",
        "../spanda-driver/src/type_check.rs",
        "../spanda-ota/src/plan.rs",
        "../spanda-driver/src/deploy_plan.rs",
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(module);
        assert!(path.exists(), "{module} should exist in workspace crate");
    }
}

#[test]
fn phase13_extractions_use_thin_shims() {
    for (module, crate_name) in [
        ("deploy_service.rs", "spanda_driver"),
        ("reliability.rs", "spanda_typecheck"),
        ("robotics_platform.rs", "spanda_runtime"),
        ("types.rs", "spanda_driver"),
        ("lexer.rs", "spanda_driver"),
        ("ffi.rs", "spanda_bridge"),
        ("transport_live.rs", "spanda_transport_routing"),
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(module);
        let source = fs::read_to_string(&path).expect(module);
        assert!(
            source.lines().count() <= 12,
            "{module} should be a thin re-export shim"
        );
        assert!(
            source.contains(crate_name),
            "{module} shim should re-export from {crate_name}"
        );
    }
}

#[test]
fn phase14_providers_facade_reexports_workspace_crates() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/providers.rs");
    let source = fs::read_to_string(path).expect("providers.rs");
    assert!(source.lines().count() <= 22);
    assert!(source.contains("spanda_providers"));
    assert!(source.contains("spanda_runtime::classification"));
}

#[test]
fn compatibility_shims_stay_thin() {
    for module in [
        "deploy_agent.rs",
        "deploy_bundle.rs",
        "deploy_http.rs",
        "deploy_remote.rs",
        "fleet_agent.rs",
        "fleet_mesh.rs",
        "fleet_orchestrator.rs",
        "fleet_remote.rs",
        "nav2_adapter.rs",
        "slam_adapter.rs",
        "connectivity_positioning.rs",
        "ffi_registry.rs",
        "transport_wire.rs",
        "transport_security.rs",
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(module);
        let source = fs::read_to_string(&path).expect(module);
        assert!(
            source.lines().count() <= 8,
            "{module} should remain a compatibility shim"
        );
    }
    for module in [
        "transport_mqtt.rs",
        "transport_dds.rs",
        "transport_websocket.rs",
        "transport_rclrs.rs",
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(module);
        let source = fs::read_to_string(&path).expect(module);
        assert!(
            source.lines().count() <= 40,
            "{module} should remain a thin transport compatibility shim"
        );
        assert!(
            source.contains("spanda_transport_routing")
                || source.contains("spanda_transport"),
            "{module} should delegate to spanda-transport-* crates"
        );
    }
}

#[test]
fn run_pipeline_lives_in_spanda_driver() {
    let run_rs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../spanda-driver/src/run.rs");
    assert!(run_rs.exists(), "run(source) should live in spanda-driver");
    let source = fs::read_to_string(run_rs).expect("spanda-driver run.rs");
    assert!(source.contains("spanda_certify::"));
    assert!(source.contains("spanda_bridge::default_ffi_registry"));
    assert!(
        source.contains("interpreter_run_program") || source.contains("spanda_interpreter::"),
        "run.rs should delegate execution to spanda-interpreter"
    );
}

#[test]
fn sir_shim_reexports_spanda_sir() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/sir.rs");
    let source = fs::read_to_string(path).expect("sir.rs shim");
    assert!(source.lines().count() <= 5, "sir.rs should be a thin re-export shim");
    assert!(source.contains("spanda_sir"));
}

#[test]
fn phase11_extractions_use_thin_shims() {
    for (module, crate_name) in [
        ("hardware.rs", "spanda_hardware"),
        ("adapter_verify.rs", "spanda_hardware"),
        ("format.rs", "spanda_format"),
        ("pretty.rs", "spanda_format"),
        ("lint.rs", "spanda_lint"),
        ("codegen.rs", "spanda_codegen"),
        ("modules.rs", "spanda_modules"),
        ("security_validate.rs", "spanda_security"),
        ("debug_session.rs", "spanda_driver"),
        ("docs.rs", "spanda_docs"),
        ("language_reference.rs", "spanda_docs"),
        ("swarm_coordinator.rs", "spanda_fleet"),
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(module);
        let source = fs::read_to_string(&path).expect(module);
        assert!(
            source.lines().count() <= 8,
            "{module} should be a thin re-export shim"
        );
        assert!(
            source.contains(crate_name),
            "{module} shim should re-export from {crate_name}"
        );
    }
}
