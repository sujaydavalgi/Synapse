//! Runtime fault recording during interpreter simulation.

use spanda_interpreter::{run_program, RunOptions};
use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_runtime::replay::MissionTrace;

const FAULT_RUNTIME_SOURCE: &str = r#"
robot Rover {
  sensor lidar: Lidar;
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  heartbeat RoverRuntime every 1s timeout 5s {
    on_missed { mark Degraded; }
  }
  behavior patrol() {
    loop every 50ms {
      wheels.drive(linear: 0.1 m/s, angular: 0.0 rad/s);
    }
  }
}
on runtime crash { diagnose root_cause; }
"#;

#[test]
fn runtime_records_faults_during_sim_with_health_injection() {
    let trace_path =
        std::env::temp_dir().join(format!("spanda_fault_runtime_{}.trace", std::process::id()));
    let _ = std::fs::remove_file(&trace_path);
    let program = parse(tokenize(FAULT_RUNTIME_SOURCE).unwrap()).unwrap();
    let options = RunOptions {
        inject_health_faults: true,
        record_trace: true,
        trace_output: Some(trace_path.to_string_lossy().into_owned()),
        max_loop_iterations: 5,
        ..Default::default()
    };
    let result = run_program(&program, options);
    assert!(result.is_ok(), "sim failed: {:?}", result.err());
    let trace = MissionTrace::load(&trace_path).expect("load trace");
    assert!(
        trace.frames.iter().any(|f| f.event.starts_with("fault_")),
        "expected fault frames in mission trace, got: {:?}",
        trace.frames.iter().map(|f| &f.event).collect::<Vec<_>>()
    );
    let _ = std::fs::remove_file(&trace_path);
}
