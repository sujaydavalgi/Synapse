//! Mission assurance core: interfaces, data models, and static analysis.
//!
//! Composes with spanda-readiness, spanda-capability, and spanda-hardware
//! without duplicating health or readiness engines.

pub mod analyze;
pub mod anomaly;
pub mod diagnosis;
pub mod evidence;
pub mod knowledge;
pub mod mission;
pub mod mitigation;
pub mod modes;
pub mod prognostics;
pub mod recovery;
pub mod report;
pub mod resilience;
pub mod state;
pub mod types;

pub use analyze::{assure_program, diagnosis_report, mitigation_report, MissionAssuranceSummary};
pub use anomaly::{learned_models, scan_anomalies, AnomalyReport};
pub use diagnosis::{diagnose_from_trace, diagnose_program, DiagnosisReport};
pub use evidence::{build_assurance_report, AssuranceReport};
pub use knowledge::{capability_ontology, extract_knowledge_base, validate_knowledge_models};
pub use mission::{verify_mission_assurance, MissionAssuranceReport};
pub use mitigation::{extract_mitigations, MitigationReport};
pub use modes::{extract_operating_modes, validate_modes};
pub use prognostics::{evaluate_prognostics, PrognosticsReport};
pub use recovery::{
    analyze_failure_with_recovery, build_recovery_audit, build_recovery_knowledge,
    build_recovery_traceability, classify_failure, evaluate_recovery, evaluate_recovery_readiness,
    execute_recovery_plan, extract_recovery_policies, operational_modes_from_program,
    parse_self_correction, plan_fleet_recovery, recovery_from_diagnosis, simulate_failure_recovery,
    validate_mode_transitions, validate_recovery_plan, FailureAnalysisWithRecovery,
    FleetRecoveryPlan, OperationalMode, PlannedRecoveryAction, RecoveryAssuranceMetrics,
    RecoveryAuditRecord, RecoveryContext, RecoveryEvidence, RecoveryKnowledgeBase,
    RecoveryKnowledgeEntry, RecoveryLevel, RecoveryPlan, RecoveryPlanner, RecoveryPolicySpec,
    RecoveryReadiness, RecoveryReport, RecoveryResult, RecoveryStatus, RecoveryStrategy,
    RecoveryTraceChain, SafeRecoveryAction,
};
pub use report::{
    format_anomaly, format_assurance, format_diagnosis, format_mission_assurance,
    format_mitigation, format_prognostics, format_recovery, format_resilience, format_state,
};
pub use resilience::{check_resilience, ResilienceReport};
pub use state::{
    build_belief_state, evaluate_state_assurance, extract_sensor_fusion, validate_state_estimators,
    StateAssuranceReport,
};
pub use types::*;
