use spanda_core::{
    check, codegen, run, run_debug, wasm_deploy_manifest, CodegenTarget, DebugOptions, FfiRegistry,
    RunOptions,
};
use std::collections::HashSet;

#[test]
fn extern_fn_parses_and_runs_with_stub() {
    let source = r#"
module ffi;

extern fn stub_add(a: Int, b: Int) -> Int;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let sum = stub_add(2, 3);
    let _ = sum;
    wheels.stop();
  }
}
"#;
    check(source).expect("extern fn should type-check");
    run(source, RunOptions::default()).expect("extern stub should run");
}

#[test]
fn codegen_native_emits_c_header() {
    let source = r#"
module demo;

export fn ping() -> Int {
  return 1;
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}
"#;
    let output = codegen(source, CodegenTarget::Native).expect("codegen");
    assert!(output.contains("spanda_main"));
    assert!(output.contains("ping"));
}

#[test]
fn codegen_wasm_emits_module() {
    let source = "module m;\nexport fn f() -> Int { return 1; }\n";
    let output = codegen(source, CodegenTarget::Wasm).expect("wasm codegen");
    assert!(output.contains("(module"));
    assert!(output.contains("spanda_main"));
}

#[test]
fn wasm_deploy_manifest_json() {
    let source = r#"
module web;

export fn handler() -> Int { return 0; }

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}
"#;
    let manifest = wasm_deploy_manifest(source).expect("manifest");
    assert!(manifest.contains("\"target\": \"wasm\""));
    assert!(manifest.contains("handler"));
}

#[test]
fn debug_session_records_breakpoint() {
    let source = r#"
module dbg;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    wheels.stop();
  }
}
"#;
    let session = run_debug(
        source,
        DebugOptions {
            breakpoints: HashSet::new(),
            step: true,
        },
    )
    .expect("debug run");
    assert!(!session.pauses.is_empty());
}

#[test]
fn ffi_registry_stub_echo() {
    let registry = FfiRegistry::new();
    let decl = spanda_core::foundations::ExternFnDecl {
        name: "stub_echo".into(),
        library: None,
        bridge: spanda_core::foundations::BridgeKind::Native,
        params: vec![],
        return_type: spanda_core::SpandaType::Int,
        span: spanda_core::Span {
            start: spanda_core::SourceLocation {
                line: 1,
                column: 1,
                offset: 0,
            },
            end: spanda_core::SourceLocation {
                line: 1,
                column: 1,
                offset: 0,
            },
        },
    };
    let out = registry
        .call(
            &decl,
            &[spanda_core::runtime::RuntimeValue::Number {
                value: 42.0,
                unit: spanda_core::UnitKind::None,
            }],
        )
        .expect("echo");
    assert!(matches!(
        out,
        spanda_core::runtime::RuntimeValue::Number { value, .. } if (value - 42.0).abs() < f64::EPSILON
    ));
}

#[test]
fn ffi_bridge_imports_type_check() {
    let source = r#"
import python.torch;
import cpp.ros2;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}
"#;
    check(source).expect("planned FFI bridge imports should type-check");
}

#[test]
fn extern_python_fn_parses_with_bridge_kind() {
    let source = r#"
extern python fn py_version() -> Int;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}
"#;
    check(source).expect("extern python fn should parse and type-check");
    let sir = spanda_core::lower_to_sir(source).expect("sir");
    assert_eq!(
        sir.externs[0].bridge,
        spanda_core::foundations::BridgeKind::Python
    );
}

#[test]
fn extern_python_fn_fails_at_runtime_without_link() {
    let source = r#"
extern python fn py_missing() -> Int;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let _ = py_missing();
    wheels.stop();
  }
}
"#;
    let err = run(source, RunOptions::default()).unwrap_err();
    assert!(
        err.to_string().contains("Unknown python extern")
            || err.to_string().contains("not found")
            || err.to_string().contains("Python")
    );
}

#[test]
fn extern_python_fn_runs_via_subprocess_bridge() {
    if !spanda_core::bridge::python::python_available()
        || spanda_core::bridge::python::bridge_script_path().is_none()
    {
        return;
    }
    let source = r#"
extern python fn py_add(a: Int, b: Int) -> Int;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let sum = py_add(10, 32);
    let _ = sum;
    wheels.stop();
  }
}
"#;
    run(source, RunOptions::default()).expect("py_add via subprocess bridge");
}

#[test]
fn extern_cpp_fn_parses_with_bridge_kind() {
    let source = r#"
extern cpp fn cpp_version() -> Int;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}
"#;
    check(source).expect("extern cpp fn should parse and type-check");
    let sir = spanda_core::lower_to_sir(source).expect("sir");
    assert_eq!(
        sir.externs[0].bridge,
        spanda_core::foundations::BridgeKind::Cpp
    );
}

#[test]
fn extern_cpp_fn_fails_at_runtime_without_handler() {
    let source = r#"
extern cpp fn cpp_missing() -> Int;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let _ = cpp_missing();
    wheels.stop();
  }
}
"#;
    let err = run(source, RunOptions::default()).unwrap_err();
    assert!(
        err.to_string().contains("Unknown cpp extern")
            || err.to_string().contains("not found")
            || err.to_string().contains("C++")
    );
}

#[test]
fn extern_cpp_fn_runs_via_subprocess_bridge() {
    if !spanda_core::bridge::cpp::bridge_available() {
        return;
    }
    let source = r#"
extern cpp fn cpp_add(a: Int, b: Int) -> Int;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let sum = cpp_add(10, 32);
    let _ = sum;
    wheels.stop();
  }
}
"#;
    run(source, RunOptions::default()).expect("cpp_add via subprocess bridge");
}
