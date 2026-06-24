//! Mission assurance and autonomous operations declaration AST nodes.
//!
use crate::nodes::Span;
use serde::{Deserialize, Serialize};

/// Component entry in a knowledge model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeComponent {
    pub name: String,
    pub span: Span,
}

/// Capability dependency in a knowledge model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeDependency {
    pub capability: String,
    pub requires: Vec<String>,
    pub span: Span,
}

/// System knowledge model declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum KnowledgeModelDecl {
    KnowledgeModelDecl {
        name: String,
        components: Vec<KnowledgeComponent>,
        dependencies: Vec<KnowledgeDependency>,
        span: Span,
    },
}

/// State estimator declaration binding sensor inputs to an estimate type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum StateEstimatorDecl {
    StateEstimatorDecl {
        name: String,
        inputs: Vec<String>,
        output_type: String,
        span: Span,
    },
}

/// Expected behavior threshold for anomaly detection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpectedBehavior {
    pub metric: String,
    pub operator: String,
    pub threshold: String,
    pub span: Span,
}

/// Anomaly detector declaration with expected behavior bounds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum AnomalyDetectorDecl {
    AnomalyDetectorDecl {
        name: String,
        #[serde(default)]
        learned_backend: Option<String>,
        expected: Vec<ExpectedBehavior>,
        span: Span,
    },
}

/// Reaction handler invoked when an anomaly is detected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum AnomalyHandlerDecl {
    AnomalyHandlerDecl {
        detector: String,
        severity: String,
        actions: Vec<String>,
        span: Span,
    },
}

/// Prognostic rule (prediction or warning threshold).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrognosticRule {
    pub kind: String,
    pub target: String,
    pub threshold: Option<String>,
    pub span: Span,
}

/// Prognostics model declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PrognosticsDecl {
    PrognosticsDecl {
        name: String,
        rules: Vec<PrognosticRule>,
        span: Span,
    },
}

/// Conditional branch inside a mitigation plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MitigationBranch {
    pub condition: String,
    pub actions: Vec<String>,
    pub span: Span,
}

/// Mitigation plan declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum MitigationDecl {
    MitigationDecl {
        name: String,
        branches: Vec<MitigationBranch>,
        span: Span,
    },
}

/// Operating mode declaration (normal, degraded, safe, emergency).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum OperatingModeDecl {
    OperatingModeDecl {
        name: String,
        mode_kind: String,
        span: Span,
    },
}

/// Single step in a mission plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionStepDecl {
    pub name: String,
    pub span: Span,
}

/// Constraint on mission execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionConstraintDecl {
    pub constraint: String,
    pub span: Span,
}

/// Mission plan declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum MissionPlanDecl {
    MissionPlanDecl {
        name: String,
        steps: Vec<MissionStepDecl>,
        constraints: Vec<MissionConstraintDecl>,
        span: Span,
    },
}

/// Resilience policy declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ResiliencePolicyDecl {
    ResiliencePolicyDecl {
        name: String,
        strategies: Vec<String>,
        span: Span,
    },
}

/// Conditional branch inside a recovery policy (`on gps.failed { ... }`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryPolicyBranch {
    pub condition: String,
    pub actions: Vec<String>,
    pub span: Span,
}

/// Recovery policy declaration for self-healing workflows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RecoveryPolicyDecl {
    RecoveryPolicyDecl {
        name: String,
        branches: Vec<RecoveryPolicyBranch>,
        span: Span,
    },
}

/// Conditional branch inside a continuity policy (`on robot.failed { ... }`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContinuityPolicyBranch {
    pub condition: String,
    pub actions: Vec<String>,
    pub span: Span,
}

/// Continuity policy declaration for takeover, delegation, and succession.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ContinuityPolicyDecl {
    ContinuityPolicyDecl {
        name: String,
        branches: Vec<ContinuityPolicyBranch>,
        span: Span,
    },
}

/// Assurance case linking evidence sources.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum AssuranceCaseDecl {
    AssuranceCaseDecl {
        name: String,
        evidence: Vec<String>,
        span: Span,
    },
}
