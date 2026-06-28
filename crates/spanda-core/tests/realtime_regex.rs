//! Tests for real-time reliability and regex language features.

use spanda_core::{check, lexer, parser, types};

#[test]
fn parses_task_deadline_and_jitter() {
    // Description:
    //     Parses task deadline and jitter.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::realtime_regex::parses_task_deadline_and_jitter();

    let source = r#"
robot R {
    task safety_monitor critical every 5ms deadline 2ms jitter <= 1ms {
        emergency_stop;
    }
}
"#;
    let tokens = lexer::tokenize(source).expect("tokenize");
    let program = parser::parse(tokens).expect("parse");
    types::check(&program).expect("type check");
}

#[test]
fn rejects_deadline_greater_than_period() {
    // Description:
    //     Rejects deadline greater than period.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::realtime_regex::rejects_deadline_greater_than_period();

    let source = r#"
robot R {
    task bad every 10ms deadline 20ms {
        emergency_stop;
    }
}
"#;
    let tokens = lexer::tokenize(source).expect("tokenize");
    let program = parser::parse(tokens).expect("parse");
    let err = types::check(&program).expect_err("expected type error");
    let msg = err.diagnostics()[0].message.to_lowercase();
    assert!(msg.contains("deadline"));
}

#[test]
fn parses_watchdog_and_pipeline() {
    // Description:
    //     Parses watchdog and pipeline.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::realtime_regex::parses_watchdog_and_pipeline();

    let source = r#"
robot R {
    task SafetyMonitor critical every 10ms {
        emergency_stop;
    }

    watchdog SafetyMonitor timeout 50ms {
        stop_all_actuators();
    }

    pipeline obstacle_avoidance budget 30ms {
        emergency_stop;
    }
}
"#;
    let tokens = lexer::tokenize(source).expect("tokenize");
    let program = parser::parse(tokens).expect("parse");
    types::check(&program).expect("type check");
}

#[test]
fn parses_regex_literal_and_validate_rule() {
    // Description:
    //     Parses regex literal and validate rule.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::realtime_regex::parses_regex_literal_and_validate_rule();

    let source = r#"
validate RobotId {
    value matches /^robot-[0-9]{3}$/;
}

robot R {
    task t every 10ms {
        let id_pattern = /^robot-[0-9]+$/;
        let text = "robot-123";
        let ok = text.matches(id_pattern);
        let _ = ok;
    }
}
"#;
    check(source).expect("check should pass");
}

#[test]
fn rejects_invalid_regex_literal() {
    // Description:
    //     Rejects invalid regex literal.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::realtime_regex::rejects_invalid_regex_literal();

    let source = r#"
robot R {
    task t every 10ms {
        let bad = /[/;
        let _ = bad;
    }
}
"#;
    let tokens = lexer::tokenize(source);
    if let Ok(tokens) = tokens {
        if let Ok(program) = parser::parse(tokens) {
            let err = types::check(&program).expect_err("invalid regex should fail");
            assert!(!err.diagnostics().is_empty());
        }
    }
}

#[test]
fn parses_subscribe_filter_and_log_trigger() {
    // Description:
    //     Parses subscribe filter and log trigger.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::realtime_regex::parses_subscribe_filter_and_log_trigger();

    let source = r#"
robot R {
    on log matches /EMERGENCY_STOP|MOTOR_FAULT/ {
        stop_all_actuators();
    }

    on message.text matches /help|stop|cancel/i {
        emergency_stop;
    }
}
"#;
    check(source).expect("check should pass");
}

#[test]
fn watchdog_runtime_keeps_heartbeats_when_task_runs() {
    // Description:
    //     Watchdog runtime keeps heartbeats when task runs.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::realtime_regex::watchdog_runtime_keeps_heartbeats_when_task_runs();

    let source = include_str!("../../../examples/realtime/watchdog.sd");
    let result = spanda_core::run(
        source,
        spanda_core::RunOptions {
            max_loop_iterations: 8,
            ..Default::default()
        },
    )
    .expect("watchdog example should run");
    let timeouts = result
        .metrics
        .watchdogs
        .get("SafetyMonitor")
        .map(|m| m.timeouts)
        .unwrap_or(0);
    assert_eq!(
        timeouts, 0,
        "task heartbeats should prevent watchdog firing"
    );
}

#[test]
fn pipeline_runtime_records_execution_via_run_pipeline() {
    // Description:
    //     Pipeline runtime records execution via run pipeline.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::realtime_regex::pipeline_runtime_records_execution_via_run_pipeline();

    let source = r#"
robot R {
    pipeline obstacle_avoidance budget 30ms {
        emergency_stop;
    }

    task driver every 10ms {
        run_pipeline obstacle_avoidance;
    }
}
"#;
    let result = spanda_core::run(
        source,
        spanda_core::RunOptions {
            max_loop_iterations: 3,
            ..Default::default()
        },
    )
    .expect("pipeline run should succeed");
    let metrics = result
        .metrics
        .pipelines
        .get("obstacle_avoidance")
        .expect("pipeline metrics");
    assert!(metrics.executions >= 1);
}

#[test]
fn mission_trace_records_scheduler_frames() {
    // Description:
    //     Mission trace records scheduler frames.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::realtime_regex::mission_trace_records_scheduler_frames();

    let source = r#"
robot R {
    task sense every 10ms {
        emergency_stop;
    }
}
"#;
    let result = spanda_core::run(
        source,
        spanda_core::RunOptions {
            max_loop_iterations: 2,
            record_trace: true,
            trace_source: Some("sense.sd".into()),
            ..Default::default()
        },
    )
    .expect("run with trace");
    let trace = result.mission_trace.expect("mission trace");
    assert!(trace.frames.iter().any(|f| f.event == "scheduler_tick"));
}

#[test]
fn mission_trace_records_behavior_loop_frames() {
    // Description:
    //     Mission trace records behavior loop frames.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:
    //     let result = spanda_core::realtime_regex::mission_trace_records_behavior_loop_frames();

    let source = r#"
robot R {
    actuator wheels: DifferentialDrive;
    behavior patrol() {
        loop every 50ms {
            wheels.drive(linear: 0.5 m/s, angular: 0.0 rad/s);
        }
    }
}
"#;
    let result = spanda_core::run(
        source,
        spanda_core::RunOptions {
            max_loop_iterations: 3,
            record_trace: true,
            trace_source: Some("patrol.sd".into()),
            ..Default::default()
        },
    )
    .expect("run with trace");
    let trace = result.mission_trace.expect("mission trace");
    assert_eq!(trace.frames.len(), 3);
    assert!(trace.frames.iter().all(|f| f.event == "behavior_tick"));
}

#[test]
fn verify_traces_detects_event_mismatch() {
    // Description:
    //     Verify traces detects event mismatch.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::realtime_regex::verify_traces_detects_event_mismatch();

    use spanda_core::replay::{verify_traces, MissionTrace};
    let mut expected = MissionTrace::new("test.sd");
    expected.record(10.0, "scheduler_tick", serde_json::json!({}));
    let mut actual = MissionTrace::new("test.sd");
    actual.record(10.0, "pipeline_run", serde_json::json!({}));
    let report = verify_traces(&expected, &actual, 0.0);
    assert!(!report.ok);
    assert!(!report.mismatches.is_empty());
}

#[test]
fn mission_trace_includes_state_snapshots() {
    // Description:
    //     Mission trace includes state snapshots.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::realtime_regex::mission_trace_includes_state_snapshots();

    let source = r#"
robot R {
    task sense every 10ms {
        emergency_stop;
    }
}
"#;
    let result = spanda_core::run(
        source,
        spanda_core::RunOptions {
            max_loop_iterations: 2,
            record_trace: true,
            trace_source: Some("sense.sd".into()),
            ..Default::default()
        },
    )
    .expect("run");
    let trace = result.mission_trace.expect("trace");
    assert!(trace.frames.iter().any(|frame| frame.state.is_some()));
}

#[test]
fn playback_applies_recorded_state() {
    // Description:
    //     Playback applies recorded state.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_core::realtime_regex::playback_applies_recorded_state();

    use spanda_core::replay::playback_frames;
    use spanda_core::simulator::{create_default_simulator, SimulatorConfig};
    let source = r#"
robot R {
    task sense every 10ms {
        emergency_stop;
    }
}
"#;
    let result = spanda_core::run(
        source,
        spanda_core::RunOptions {
            max_loop_iterations: 2,
            record_trace: true,
            ..Default::default()
        },
    )
    .expect("run");
    let trace = result.mission_trace.expect("trace");
    let mut sim = create_default_simulator(SimulatorConfig::default());
    let report = playback_frames(trace.frames.as_slice(), &mut sim, false);
    assert!(report.states_applied >= 1);
}
