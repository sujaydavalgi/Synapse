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
        budget: Option<ResourceBudgetDecl>,
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

/// First-class hardware profile for deployment compatibility verification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum HardwareDecl {
    HardwareDecl {
        name: String,
        cpu: Option<String>,
        memory_mb: Option<f64>,
        storage_mb: Option<f64>,
        gpu_tops: Option<f64>,
        gpu_required: bool,
        sensors: Vec<String>,
        actuators: Vec<String>,
        battery_wh: Option<f64>,
        network_bandwidth_mbps: Option<f64>,
        network_latency_ms: Option<f64>,
        min_control_period_ms: Option<f64>,
        power_draw_w: Option<f64>,
        span: Span,
    },
}

/// Deployment target binding: `deploy <robot> to <hardware>;`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum DeployDecl {
    DeployDecl {
        robot_name: String,
        targets: Vec<String>,
        span: Span,
    },
}

/// Program- or robot-level hardware requirements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RequiresHardwareDecl {
    RequiresHardwareDecl {
        memory_mb_min: Option<f64>,
        storage_mb_min: Option<f64>,
        gpu_tops_min: Option<f64>,
        gpu_required: bool,
        sensors: Vec<String>,
        actuators: Vec<String>,
        span: Span,
    },
}

/// Network connectivity requirements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RequiresNetworkDecl {
    RequiresNetworkDecl {
        bandwidth_mbps_min: Option<f64>,
        latency_ms_max: Option<f64>,
        span: Span,
    },
}

/// Per-task resource budget constraints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ResourceBudgetDecl {
    ResourceBudgetDecl {
        battery_pct_max: Option<f64>,
        memory_mb_max: Option<f64>,
        cpu_pct_max: Option<f64>,
        network_mbps_max: Option<f64>,
        storage_mb_max: Option<f64>,
        span: Span,
    },
}

/// Mission duration for power budgeting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum MissionDecl {
    MissionDecl { duration_hours: f64, span: Span },
}

/// Fault injection scenario for simulation-time compatibility checks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimFaultDecl {
    pub fault_type: String,
    pub span: Span,
}

/// `simulate_compatibility { fault CameraFailure; }`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SimulateCompatibilityDecl {
    SimulateCompatibilityDecl {
        faults: Vec<SimFaultDecl>,
        span: Span,
    },
}

/// Sensor fusion configuration listing sensors to combine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ObserveDecl {
    ObserveDecl { sensors: Vec<String>, span: Span },
}

/// System-level verification assertions checked after behavior/task execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum VerifyDecl {
    VerifyDecl { rules: Vec<Expr>, span: Span },
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
            | "std.time"
            | "std.units"
            | "std.spatial"
            | "std.ai"
            | "std.robotics"
            | "std.sensors"
            | "std.safety"
            | "std.twin"
            | "std.hri"
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
