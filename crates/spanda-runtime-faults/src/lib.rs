//! Runtime fault detection for Spanda autonomous systems.
//!
//! Detects memory leaks, crashes, reboots, hangs, restarts, and resource
//! exhaustion. Integrates with health, readiness, diagnosis, recovery, replay,
//! audit, and assurance systems.

pub mod detection;
pub mod engine;
pub mod integrate;
pub mod replay;
pub mod report;
pub mod types;

pub use detection::{
    collect_configured_monitors, detect_from_runtime_signals, evaluate_memory_watches,
    evaluate_resource_pressure, evaluate_restart_loops, faults_from_hardware_signals,
    parse_memory_watch_threshold, resource_pressure_from_condition, static_fault_scan,
};
pub use engine::{
    build_runtime_health, build_timeline, extract_reliability_evidence, scan_program_faults,
    FaultScanOptions,
};
pub use integrate::{
    apply_fault_readiness_impact, diagnose_fault, diagnose_fault_report, fault_to_crash_event,
    recommend_recovery,
};
pub use replay::{
    fault_frames, faults_from_trace, format_trace_faults, record_fault_in_trace,
    record_faults_in_trace, FAULT_EVENTS,
};
pub use report::{format_fault_report, format_runtime_health};
pub use types::*;
