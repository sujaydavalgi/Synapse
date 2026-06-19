use crate::ast::{Expr, Span, Stmt};
use serde::{Deserialize, Serialize};

/// Top-level struct declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum StructDecl {
    StructDecl {
        name: String,
        fields: Vec<FieldDecl>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDecl {
    pub name: String,
    pub type_name: String,
    pub span: Span,
}

/// Top-level enum declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum EnumDecl {
    EnumDecl {
        name: String,
        variants: Vec<String>,
        span: Span,
    },
}

/// Top-level trait declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TraitDecl {
    TraitDecl {
        name: String,
        methods: Vec<TraitMethodDecl>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitMethodDecl {
    pub name: String,
    pub params: Vec<TraitParamDecl>,
    pub return_type: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitParamDecl {
    pub name: String,
    pub type_name: String,
    pub span: Span,
}

/// Trait implementation bound to an agent inside a robot block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TraitImplDecl {
    TraitImplDecl {
        trait_name: String,
        agent_name: String,
        methods: Vec<TraitImplMethodDecl>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitImplMethodDecl {
    pub name: String,
    pub params: Vec<TraitParamDecl>,
    pub return_type: String,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// Pattern-matching expression arm.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub variant: String,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// Executable state machine inside a robot block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum StateMachineDecl {
    StateMachineDecl {
        name: String,
        states: Vec<String>,
        transitions: Vec<TransitionDecl>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransitionDecl {
    pub from: String,
    pub to: String,
    pub span: Span,
}

/// Deterministic periodic task (distinct from legacy `behavior`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TaskDecl {
    TaskDecl {
        name: String,
        interval_ms: f64,
        requires: Option<Expr>,
        ensures: Option<Expr>,
        invariant: Option<Expr>,
        body: Vec<Stmt>,
        span: Span,
    },
}

/// Event declaration and handler.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum EventDecl {
    EventDecl { name: String, span: Span },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum EventHandlerDecl {
    EventHandlerDecl {
        event_name: String,
        body: Vec<Stmt>,
        span: Span,
    },
}

/// Digital twin shadow configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TwinDecl {
    TwinDecl {
        name: String,
        mirrors: Vec<String>,
        replay: bool,
        span: Span,
    },
}

/// Capability granted to an agent (`can [ read(lidar), propose_motion ]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityDecl {
    pub action: String,
    pub target: Option<String>,
    pub span: Span,
}

/// Known code-module import paths (Phase 1 module system).
pub fn resolve_module_import(path: &str) -> bool {
    matches!(
        path,
        "sensors.lidar"
            | "sensors.camera"
            | "sensors.imu"
            | "motion.drive"
            | "motion.arm"
            | "navigation.planning"
            | "navigation.localize"
            | "safety.validate"
            | "ai.reasoning"
    )
}

/// Map user-facing type aliases to physical units / builtin types.
pub fn resolve_type_alias(name: &str) -> Option<&'static str> {
    match name {
        "Distance" | "meter" | "Meter" => Some("distance"),
        "Angle" | "radian" | "Radian" => Some("angle"),
        "Path" => Some("path"),
        "Velocity" => Some("velocity"),
        "Pose" => Some("pose"),
        _ => None,
    }
}
