use spanda_core::{check, check_with_registry, compile, run, ModuleRegistry, RunOptions};

#[test]
fn export_fn_in_module_type_checks() {
    let source = r#"
module navigation.path_planning;

export fn plan_path() -> Path {
  return trajectory(from: pose(x: 0.0 m, y: 0.0 m), to: pose(x: 1.0 m, y: 0.0 m), steps: 3);
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let route = plan_path();
    let _ = route;
    wheels.stop();
  }
}
"#;
    check(source).expect("export fn should type-check in same module");
    run(source, RunOptions::default()).expect("export fn should run");
}

#[test]
fn cross_module_import_resolves_export() {
    let planning = r#"
module navigation.path_planning;

export fn plan_path() -> Path {
  return trajectory(from: pose(x: 0.0 m, y: 0.0 m), to: pose(x: 2.0 m, y: 0.0 m), steps: 4);
}
"#;
    let main = r#"
module navigation;

import navigation.path_planning;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let route = plan_path();
    let _ = route;
    wheels.stop();
  }
}
"#;
    let planning_program = compile(planning).expect("planning module").program;
    let mut registry = ModuleRegistry::new();
    registry.register("navigation.path_planning", &planning_program);
    check_with_registry(main, &registry).expect("import should resolve exported fn");
    let mut opts = RunOptions::default();
    opts.module_registry = Some(registry);
    run(main, opts).expect("imported fn should run");
}

#[test]
fn private_fn_not_exported_to_importer() {
    let planning = r#"
module navigation.path_planning;

private fn helper() -> Path {
  return trajectory(from: pose(x: 0.0 m, y: 0.0 m), to: pose(x: 1.0 m, y: 0.0 m), steps: 2);
}
"#;
    let main = r#"
module navigation;
import navigation.path_planning;

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    helper();
    wheels.stop();
  }
}
"#;
    let planning_program = compile(planning).expect("planning").program;
    let mut registry = ModuleRegistry::new();
    registry.register("navigation.path_planning", &planning_program);
    let err = check_with_registry(main, &registry).expect_err("private fn should not import");
    assert!(
        err.diagnostics()
            .iter()
            .any(|d| d.message.contains("Unknown function") || d.message.contains("Undefined")),
        "got {:?}",
        err.diagnostics()
    );
}

#[test]
fn generic_export_fn_with_type_param() {
    let source = r#"
module std.collections;

export fn identity<T>(value: T) -> T {
  return value;
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let n: Int = identity(3);
    let _ = n;
    wheels.stop();
  }
}
"#;
    check(source).expect("generic export fn should type-check");
}

#[test]
fn result_ok_err_match() {
    let source = r#"
module navigation;

enum NavError { Blocked, Timeout }

export fn navigate() -> Result<Path, NavError> {
  return Err(Blocked);
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let outcome = navigate();
    match outcome {
      Ok => wheels.stop();
      Err => wheels.stop();
    };
  }
}
"#;
    check(source).expect("Result match should type-check");
    run(source, RunOptions::default()).expect("Result match should run");
}

#[test]
fn option_some_none_match() {
    let source = r#"
module sensors;

export fn latest_scan() -> Option<Scan> {
  return None();
}

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    match latest_scan() {
      Some => wheels.stop();
      None => wheels.stop();
    };
  }
}
"#;
    check(source).expect("Option match should type-check");
}

#[test]
fn result_generic_type_annotation() {
    let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let ok: Result<Path, Error> = Ok(trajectory(from: pose(x: 0.0 m, y: 0.0 m), to: pose(x: 1.0 m, y: 0.0 m), steps: 2));
    let _ = ok;
    wheels.stop();
  }
}
"#;
    check(source).expect("Result<T,E> annotation should parse");
}
