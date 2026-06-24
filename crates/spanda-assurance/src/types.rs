//! Core data models for mission assurance and autonomous operations.
//!
use serde::{Deserialize, Serialize};

/// Confidence level for state estimates (0.0–1.0).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Confidence(pub f64);

/// State estimate with confidence and source attribution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateEstimate {
    pub name: String,
    pub value: String,
    pub confidence: Confidence,
    pub sources: Vec<String>,
}

/// Belief state aggregating multiple estimates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeliefState {
    pub estimates: Vec<StateEstimate>,
}

/// Sensor fusion state snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorFusionState {
    pub estimator: String,
    pub inputs: Vec<String>,
    pub fused: Option<StateEstimate>,
}

/// Detected anomaly with severity and context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Anomaly {
    pub detector: String,
    pub metric: String,
    pub expected: String,
    pub observed: String,
    pub severity: AnomalySeverity,
}

/// Anomaly severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Expected behavior model from static declarations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpectedBehaviorModel {
    pub detector: String,
    pub rules: Vec<String>,
}

/// Learned behavior model placeholder (optional package backend).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LearnedBehaviorModel {
    pub detector: String,
    pub backend: String,
}

/// Root cause finding from diagnosis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RootCause {
    pub description: String,
    pub confidence: Confidence,
    pub contributing: Vec<String>,
}

/// Diagnosis report for a fault or anomaly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnosis {
    pub subject: String,
    pub root_causes: Vec<RootCause>,
    pub fault_tree: FaultTree,
}

/// Fault tree node for structured diagnosis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FaultTree {
    pub top_event: String,
    pub gates: Vec<String>,
}

/// Causal graph edge for diagnosis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausalGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>,
}

/// Prognostic model metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrognosticModel {
    pub name: String,
    pub target: String,
    pub rules: Vec<String>,
}

/// Remaining useful life prediction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemainingUsefulLife {
    pub component: String,
    pub estimate: String,
    pub confidence: Confidence,
}

/// Failure prediction from prognostics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailurePrediction {
    pub component: String,
    pub probability: f64,
    pub horizon: String,
}

/// Degradation trend observation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DegradationTrend {
    pub metric: String,
    pub direction: String,
    pub rate: String,
}

/// Mitigation plan with recovery actions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MitigationPlan {
    pub name: String,
    pub actions: Vec<RecoveryAction>,
    pub fallback: Option<FallbackMode>,
}

/// Single recovery action in a mitigation plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryAction {
    pub description: String,
    pub condition: Option<String>,
}

/// Fallback operating mode reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FallbackMode {
    pub mode: String,
}

/// Safe mode transition descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafeModeTransition {
    pub from_mode: String,
    pub to_mode: String,
    pub trigger: String,
}

/// Operating mode descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperatingMode {
    pub name: String,
    pub kind: ModeKind,
}

/// Standard operating mode kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ModeKind {
    Normal,
    Degraded,
    Safe,
    Emergency,
}

/// Mission plan summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionPlan {
    pub name: String,
    pub steps: Vec<String>,
    pub constraints: Vec<String>,
}

/// Mission execution state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionExecutionState {
    pub plan: String,
    pub current_step: Option<String>,
    pub status: String,
}

/// Reason for mission abort.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionAbortReason {
    pub reason: String,
    pub severity: AnomalySeverity,
}

/// Resilience policy descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResiliencePolicy {
    pub name: String,
    pub strategies: Vec<FaultToleranceStrategy>,
}

/// Recovery policy for fault handling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryPolicy {
    pub name: String,
    pub actions: Vec<String>,
}

/// Fault tolerance strategy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FaultToleranceStrategy {
    pub name: String,
    pub description: String,
}

/// Redundancy model for components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RedundancyModel {
    pub component: String,
    pub replicas: u32,
    pub failover: String,
}

/// System model from knowledge declarations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemModel {
    pub name: String,
    pub components: Vec<ComponentModel>,
    pub dependencies: DependencyGraph,
}

/// Component in a system model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentModel {
    pub name: String,
}

/// Dependency graph for capabilities and components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub edges: Vec<(String, Vec<String>)>,
}

/// Mission knowledge base aggregating models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionKnowledgeBase {
    pub models: Vec<SystemModel>,
}

/// Capability ontology entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityOntology {
    pub capability: String,
    pub requires: Vec<String>,
}

/// Assurance case with linked evidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssuranceCase {
    pub name: String,
    pub evidence: Vec<EvidenceRecord>,
}

/// Single evidence record in an assurance case.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub source: String,
    pub kind: EvidenceKind,
    pub status: String,
}

/// Evidence classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Verification,
    Safety,
    Traceability,
    Health,
    Replay,
    Hardware,
    Capability,
}

/// Verification evidence summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationEvidence {
    pub compatible: bool,
    pub items: Vec<String>,
}

/// Safety evidence summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafetyEvidence {
    pub rules: Vec<String>,
    pub kill_switches: Vec<String>,
}

/// Traceability evidence summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceabilityEvidence {
    pub rows: Vec<String>,
}
