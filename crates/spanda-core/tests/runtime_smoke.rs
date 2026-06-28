//! Smoke tests for the interpreter runtime (moved from `runtime.rs`).
//!
use std::cell::RefCell;
use std::rc::Rc;

use spanda_core::lexer;
use spanda_core::parser;
use spanda_core::runtime::{Interpreter, InterpreterOptions};
use spanda_core::simulator::{create_default_simulator, Obstacle, SimulatorConfig};
use spanda_core::{run, RobotState, RunOptions, SpandaError};

fn compile_and_run(source: &str, max_iters: usize) -> Result<RobotState, SpandaError> {
    // Description:
    //     Compile and run.
    //
    // Inputs:
    //     source: &str
    //         Caller-supplied source.
    //     ax_iters: usize
    //         Caller-supplied ax iters.
    //
    // Outputs:
    //     result: Result<RobotState, SpandaError>
    //         Return value from `compile_and_run`.
    //
    // Example:

    //     let result = spanda_core::runtime_smoke::compile_and_run(source, ax_iters);

    let tokens = lexer::tokenize(source)?;
    let program = parser::parse(tokens)?;
    let sim = create_default_simulator(SimulatorConfig {
        obstacles: vec![Obstacle {
            x: 100.0,
            y: 100.0,
            radius: 0.1,
        }],
        ..Default::default()
    });
    let mut interp = Interpreter::new(
        sim,
        InterpreterOptions {
            max_loop_iterations: max_iters,
            ..Default::default()
        },
    );
    interp.run(&program, None)
}

#[test]
fn executes_let_bindings_and_if_else() {
    // Description:
    //     Executes let bindings and if else.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::runtime_smoke::executes_let_bindings_and_if_else();

    let source = r#"
      robot R {
        sensor lidar: Lidar;
        actuator wheels: DifferentialDrive;
        behavior test() {
          let scan = lidar.read();
          if scan.nearest_distance < 0.5 m {
            wheels.stop();
          } else {
            wheels.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
          }
        }
      }
    "#;
    let state = compile_and_run(source, 1).unwrap();
    assert!(state.velocity.linear > 0.0);
}

#[test]
fn runs_deterministic_loop() {
    // Description:
    //     Runs deterministic loop.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::runtime_smoke::runs_deterministic_loop();

    let source = r#"
      robot R {
        actuator wheels: DifferentialDrive;
        behavior tick() {
          loop every 100ms {
            wheels.drive(linear: 0.1 m/s, angular: 0.0 rad/s);
          }
        }
      }
    "#;
    let state = compile_and_run(source, 5).unwrap();
    assert!(state.pose.x > 0.0);
}

#[test]
fn stops_on_close_obstacle() {
    // Description:
    //     Stops on close obstacle.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::runtime_smoke::stops_on_close_obstacle();

    let source = r#"
      robot R {
        sensor lidar: Lidar;
        actuator wheels: DifferentialDrive;
        behavior avoid() {
          loop every 50ms {
            let scan = lidar.read();
            if scan.nearest_distance < 0.5 m {
              wheels.stop();
            } else {
              wheels.drive(linear: 0.8 m/s, angular: 0.0 rad/s);
            }
          }
        }
      }
    "#;
    let tokens = lexer::tokenize(source).unwrap();
    let program = parser::parse(tokens).unwrap();
    let sim = create_default_simulator(SimulatorConfig {
        obstacles: vec![Obstacle {
            x: 0.3,
            y: 0.0,
            radius: 0.1,
        }],
        ..Default::default()
    });
    let mut interp = Interpreter::new(
        sim,
        InterpreterOptions {
            max_loop_iterations: 3,
            ..Default::default()
        },
    );
    let state = interp.run(&program, None).unwrap();
    assert_eq!(state.velocity.linear, 0.0);
}

#[test]
fn enforces_safety_in_interpreter() {
    // Description:
    //     Enforces safety in interpreter.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::runtime_smoke::enforces_safety_in_interpreter();

    let source = r#"
      robot R {
        sensor lidar: Lidar;
        actuator wheels: DifferentialDrive;
        safety {
          stop_if lidar.read().nearest_distance < 1.0 m;
        }
        behavior go() {
          wheels.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
        }
      }
    "#;
    let tokens = lexer::tokenize(source).unwrap();
    let program = parser::parse(tokens).unwrap();
    let sim = create_default_simulator(SimulatorConfig {
        obstacles: vec![Obstacle {
            x: 0.5,
            y: 0.0,
            radius: 0.1,
        }],
        ..Default::default()
    });
    let blocked = Rc::new(RefCell::new(Vec::new()));
    let blocked_cb = blocked.clone();
    let mut interp = Interpreter::new(
        sim,
        InterpreterOptions {
            max_loop_iterations: 1,
            on_motion_blocked: Some(Rc::new(move |reason| {
                blocked_cb.borrow_mut().push(reason);
            })),
            ..Default::default()
        },
    );
    let state = interp.run(&program, None).unwrap();
    assert!(!blocked.borrow().is_empty());
    assert!(state.emergency_stop);
}

#[test]
fn stop_if_nearest_distance_does_not_block_clear_path() {
    // Description:
    //     Stop-if lidar.nearest_distance must not E-stop when nearest obstacle is beyond threshold.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:
    //     let result = spanda_core::runtime_smoke::stop_if_nearest_distance_does_not_block_clear_path();

    let source = r#"
      robot R {
        sensor lidar: Lidar;
        actuator wheels: DifferentialDrive;
        safety {
          stop_if lidar.nearest_distance < 0.5 m;
        }
        behavior go() {
          wheels.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
        }
      }
    "#;
    let tokens = lexer::tokenize(source).unwrap();
    let program = parser::parse(tokens).unwrap();
    let sim = create_default_simulator(SimulatorConfig::default());
    let mut interp = Interpreter::new(
        sim,
        InterpreterOptions {
            max_loop_iterations: 1,
            ..Default::default()
        },
    );
    let state = interp.run(&program, None).unwrap();
    assert!(
        state.velocity.linear > 0.0,
        "expected motion when default sim obstacles are beyond 0.5 m"
    );
    assert!(!state.emergency_stop);
}

#[test]
fn stop_if_with_mission_does_not_block_clear_path() {
    // Description:
    //     Mission-declared robots must evaluate stop_if against live lidar readings.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:
    //     let result = spanda_core::runtime_smoke::stop_if_with_mission_does_not_block_clear_path();

    let source = r#"
      robot Vehicle {
        sensor front_lidar: Lidar;
        actuator wheels: DifferentialDrive;
        safety {
          stop_if front_lidar.nearest_distance < 0.5 m;
        }
        mission M {
          drive_once;
        }
        behavior drive_once() {
          wheels.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
        }
      }
    "#;
    let tokens = lexer::tokenize(source).unwrap();
    let program = parser::parse(tokens).unwrap();
    let sim = create_default_simulator(SimulatorConfig::default());
    let mut interp = Interpreter::new(
        sim,
        InterpreterOptions {
            max_loop_iterations: 20,
            ..Default::default()
        },
    );
    let state = interp.run(&program, None).unwrap();
    assert!(
        state.velocity.linear > 0.0,
        "mission robots should drive when lidar path is clear"
    );
    assert!(!state.emergency_stop);
}

#[test]
fn stop_if_mission_via_driver_run_does_not_block_clear_path() {
    // Description:
    //     CLI/driver run path must not false-trigger stop_if on clear lidar paths.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:
    //     let result = spanda_core::runtime_smoke::stop_if_mission_via_driver_run_does_not_block_clear_path();

    let source = r#"
      robot Vehicle {
        sensor front_lidar: Lidar;
        actuator wheels: DifferentialDrive;
        safety {
          stop_if front_lidar.nearest_distance < 0.5 m;
        }
        mission M {
          drive_once;
        }
        behavior drive_once() {
          wheels.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
        }
      }
    "#;
    let result = run(
        source,
        RunOptions {
            max_loop_iterations: 20,
            official_packages: vec![],
            ..Default::default()
        },
    )
    .expect("driver run without packages");
    assert!(
        result.state.velocity.linear > 0.0,
        "driver run should move when lidar path is clear"
    );
    assert!(!result.state.emergency_stop);
}
