//! Tests for real-time reliability and regex language features.

use spanda_core::{check, lexer, parser, types};

#[test]
fn parses_task_deadline_and_jitter() {
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
