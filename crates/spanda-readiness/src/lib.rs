//! Operational readiness, mission verification, failure analysis, and safety reporting.
//!
//! Composes hardware verification, capability verification, health framework,
//! fleet support, simulation/replay, safety validation, and traceability into
//! unified operational intelligence.

pub mod agent;
pub mod approval;
pub mod auditor;
pub mod config;
pub mod dashboard;
pub mod diagnostics;
pub mod engine;
pub mod failure;
pub mod fleet;
pub mod fleet_verify;
pub mod gates;
pub mod mission;
pub mod report;
pub mod root_cause;
pub mod runtime;
pub mod safety_coverage;
pub mod safety_report;
pub mod spans;
pub mod target;
pub mod traceability;
pub mod twin;
pub mod types;

pub use agent::{evaluate_agent_readiness, evaluate_agent_readiness_json};
pub use approval::{verify_approvals, verify_approvals_source, ApprovalVerifyReport};
pub use auditor::{audit_program, audit_program_source, SafetyAuditReport};
pub use dashboard::{FleetDashboard, HealthDashboard, MissionDashboard, ReadinessDashboard};
pub use diagnostics::collect_readiness_diagnostics;
pub use engine::{evaluate_readiness, evaluate_readiness_source, evaluate_readiness_with_runtime};
pub use failure::{analyze_failure, analyze_failure_source, FailureAnalysisReport};
pub use fleet::evaluate_fleet_readiness;
pub use fleet_verify::{verify_fleet, verify_fleet_source, FleetVerifyReport};
pub use gates::{
    evaluate_deployment_gates, DeploymentGate, DeploymentGatePolicy, DeploymentGateReport,
};
pub use mission::{verify_mission, verify_mission_source, MissionVerificationReport};
pub use report::{
    format_audit, format_failure_analysis, format_fleet_readiness, format_mission_verification,
    format_readiness, format_root_cause, format_safety_report, format_twin_readiness,
};
pub use root_cause::{diagnose_trace, RootCauseReport};
pub use runtime::{
    build_runtime_context, build_runtime_context_with_config, seed_hardware_monitor,
    RuntimeReadinessContext,
};
pub use safety_coverage::{
    evaluate_safety_coverage, format_safety_coverage, CoverageStatus as SafetyCoverageStatus,
    SafetyCoverageReport, SafetyScenarioReport,
};
pub use safety_report::{generate_safety_report, generate_safety_report_source, SafetyCaseReport};
pub use target::{default_deploy_target, readiness_options_from_flags};
pub use traceability::{readiness_traceability, ReadinessTraceRow};
pub use twin::evaluate_twin_readiness;
pub use types::{
    FleetReadinessReport, ReadinessIssue, ReadinessOptions, ReadinessPolicy, ReadinessReport,
    ReadinessScore, ReadinessSeverity, ReadinessStatus, ReportFormat, TwinReadinessStatus,
};
