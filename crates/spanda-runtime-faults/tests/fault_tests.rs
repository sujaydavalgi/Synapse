//! Integration tests for runtime fault detection.

use spanda_lexer::tokenize;
use spanda_parser::parse;
use spanda_readiness::{evaluate_readiness, ReadinessOptions};
use spanda_runtime::replay::MissionTrace;
use spanda_runtime_faults::{
    apply_fault_readiness_impact, diagnose_fault_report, evaluate_memory_watches,
    faults_from_trace, parse_memory_watch_threshold, record_fault_in_trace, scan_program_faults,
    FaultScanOptions, RuntimeFault, RuntimeFaultKind, RuntimeHealthStatus,
};

fn parse_source(source: &str) -> spanda_ast::nodes::Program {
    parse(tokenize(source).unwrap()).unwrap()
}

const HEARTBEAT_SOURCE: &str = r#"
robot Rover {
  sensor lidar: Lidar;
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  heartbeat RoverRuntime every 1s timeout 5s {
    on_missed { mark Degraded; }
  }
  behavior b() { loop every 50ms { } }
}
"#;

const MEMORY_WATCH_SOURCE: &str = r#"
robot Rover {
  sensor lidar: Lidar;
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  memory_watch RoverRuntime {
    threshold growth > 100 MB over 10 min;
    action { mark Warning; }
  }
  behavior b() { loop every 50ms { } }
}
"#;

const CRASH_SOURCE: &str = r#"
robot Rover {
  sensor lidar: Lidar;
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  restart_policy ProviderRuntime {
    max_restarts: 3 within 5 min;
    on_exceeded { enter degraded_mode; }
  }
  behavior b() { loop every 50ms { } }
}
on runtime crash { diagnose root_cause; recover using RestartRuntime; }
"#;

const REBOOT_SOURCE: &str = r#"
robot Rover {
  sensor gps: GPS;
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  behavior b() { loop every 50ms { } }
}
on reboot unexpected { run post_reboot_diagnostics; }
"#;

const RESOURCE_SOURCE: &str = r#"
robot Rover {
  sensor lidar: Lidar;
  actuator wheels: DifferentialDrive;
  safety { max_speed = 1.0 m/s; }
  resource_watch {
    memory > 85%;
    cpu > 90% for 30s;
    disk_free < 500 MB;
  }
  behavior b() { loop every 50ms { } }
}
"#;

#[test]
fn heartbeat_parsing() {
    let program = parse_source(HEARTBEAT_SOURCE);
    let report = scan_program_faults(&program, "test.sd", &FaultScanOptions::default());
    assert_eq!(report.heartbeats_configured, 1);
}

#[test]
fn memory_watch_parsing() {
    let program = parse_source(MEMORY_WATCH_SOURCE);
    let report = scan_program_faults(&program, "test.sd", &FaultScanOptions::default());
    assert_eq!(report.memory_watches_configured, 1);
    let (mb, ms) = parse_memory_watch_threshold("100 MB", "10 min");
    assert!((mb - 100.0).abs() < f64::EPSILON);
    assert!((ms - 600_000.0).abs() < f64::EPSILON);
}

#[test]
fn crash_event_creation() {
    let program = parse_source(CRASH_SOURCE);
    let options = FaultScanOptions {
        inject_crash: true,
        ..Default::default()
    };
    let report = scan_program_faults(&program, "test.sd", &options);
    assert!(report
        .faults
        .iter()
        .any(|f| f.kind == RuntimeFaultKind::ProcessCrash));
    assert!(report
        .faults
        .iter()
        .any(|f| f.status == RuntimeHealthStatus::Crashed));
}

#[test]
fn reboot_event_creation() {
    let program = parse_source(REBOOT_SOURCE);
    let options = FaultScanOptions {
        inject_reboot: true,
        ..Default::default()
    };
    let report = scan_program_faults(&program, "test.sd", &options);
    assert!(report
        .faults
        .iter()
        .any(|f| f.kind == RuntimeFaultKind::UnexpectedReboot));
}

#[test]
fn restart_loop_detection() {
    let program = parse_source(CRASH_SOURCE);
    let options = FaultScanOptions {
        inject_crash: true,
        ..Default::default()
    };
    let report = scan_program_faults(&program, "test.sd", &options);
    assert!(report
        .faults
        .iter()
        .any(|f| f.kind == RuntimeFaultKind::RestartLoop));
    assert_eq!(report.restart_policies_configured, 1);
}

#[test]
fn resource_pressure_detection() {
    let program = parse_source(RESOURCE_SOURCE);
    let options = FaultScanOptions {
        inject_resource_pressure: true,
        ..Default::default()
    };
    let report = scan_program_faults(&program, "test.sd", &options);
    assert!(report.faults.iter().any(|f| matches!(
        f.kind,
        RuntimeFaultKind::MemoryPressure | RuntimeFaultKind::CpuOverload
    )));
    assert_eq!(report.resource_watches_configured, 1);
}

#[test]
fn readiness_impact() {
    let program = parse_source(CRASH_SOURCE);
    let options = FaultScanOptions {
        inject_crash: true,
        ..Default::default()
    };
    let fault_report = scan_program_faults(&program, "test.sd", &options);
    let mut readiness = evaluate_readiness(&program, &ReadinessOptions::default());
    apply_fault_readiness_impact(&mut readiness, &fault_report);
    assert!(!readiness.issues.is_empty());
}

#[test]
fn diagnosis_integration() {
    let program = parse_source(CRASH_SOURCE);
    let options = FaultScanOptions {
        inject_crash: true,
        ..Default::default()
    };
    let report = scan_program_faults(&program, "test.sd", &options);
    let diagnoses = diagnose_fault_report(&report);
    assert!(!diagnoses.is_empty());
    assert!(
        diagnoses[0].likely_cause.contains("exit") || diagnoses[0].likely_cause.contains("crash")
    );
}

#[test]
fn recovery_integration() {
    let program = parse_source(CRASH_SOURCE);
    let options = FaultScanOptions {
        inject_crash: true,
        ..Default::default()
    };
    let report = scan_program_faults(&program, "test.sd", &options);
    if let Some(fault) = report.faults.first() {
        let actions = spanda_runtime_faults::recommend_recovery(fault, &program);
        assert!(!actions.is_empty());
    }
}

#[test]
fn replay_recording() {
    let fault = RuntimeFault {
        kind: RuntimeFaultKind::ProcessCrash,
        target: "Rover".into(),
        status: RuntimeHealthStatus::Crashed,
        message: "test crash".into(),
        evidence: spanda_runtime_faults::FaultEvidence {
            metric: None,
            value: None,
            threshold: None,
            boot_id: None,
            exit_code: None,
            stack_trace: None,
            related_events: Vec::new(),
        },
        detected_at_ms: 1000.0,
    };
    let mut trace = MissionTrace::new("test.sd");
    record_fault_in_trace(&mut trace, &fault, 1000.0);
    let timeline = faults_from_trace(&trace);
    assert_eq!(timeline.len(), 1);
    assert_eq!(timeline[0].event, "fault_crash");
}

#[test]
fn memory_leak_detection() {
    let program = parse_source(MEMORY_WATCH_SOURCE);
    let faults = evaluate_memory_watches(&program, 250.0, 100.0);
    assert!(faults
        .iter()
        .any(|f| f.kind == RuntimeFaultKind::MemoryLeak));
}

#[test]
fn heartbeat_loss_injection() {
    let program = parse_source(HEARTBEAT_SOURCE);
    let options = FaultScanOptions {
        inject_heartbeat_loss: true,
        ..Default::default()
    };
    let report = scan_program_faults(&program, "test.sd", &options);
    assert!(report
        .faults
        .iter()
        .any(|f| f.kind == RuntimeFaultKind::HeartbeatLoss));
}
