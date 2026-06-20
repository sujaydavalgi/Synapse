use crate::ast::{Expr, Span, SpandaType, Stmt};
use serde::{Deserialize, Serialize};

/// Symbol visibility for module-level items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Private,
    Public,
    Export,
}

/// Module-level function declaration (`export fn plan_path() { ... }`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleFnDecl {
    pub name: String,
    pub visibility: Visibility,
    pub type_params: Vec<String>,
    pub params: Vec<ModuleParamDecl>,
    pub return_type: SpandaType,
    #[serde(default)]
    pub is_async: bool,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleParamDecl {
    pub name: String,
    pub type_ann: SpandaType,
    pub span: Span,
}

/// Foreign function bridge target (orchestration layer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BridgeKind {
    #[default]
    Native,
    Python,
    Cpp,
}

impl BridgeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Python => "python",
            Self::Cpp => "cpp",
        }
    }
}

/// Foreign function interface declaration (`extern fn read_sensor() -> Int;`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternFnDecl {
    pub name: String,
    pub library: Option<String>,
    #[serde(default)]
    pub bridge: BridgeKind,
    pub params: Vec<ModuleParamDecl>,
    pub return_type: SpandaType,
    pub span: Span,
}

/// In-language test block: `test "name" { ... }`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestDecl {
    pub name: String,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// Select arm: `recv(ch) => { ... }`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectArm {
    pub channel: Expr,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// Top-level struct declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum StructDecl {
    StructDecl {
        name: String,
        #[serde(default)]
        type_params: Vec<String>,
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

/// Enum variant with optional payload field types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariantDecl {
    pub name: String,
    #[serde(default)]
    pub field_types: Vec<String>,
    pub span: Span,
}

/// Top-level enum declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum EnumDecl {
    EnumDecl {
        name: String,
        variants: Vec<EnumVariantDecl>,
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
    #[serde(default)]
    pub bindings: Vec<String>,
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
        priority: TaskPriority,
        interval_ms: f64,
        requires: Option<Expr>,
        ensures: Option<Expr>,
        invariant: Option<Expr>,
        budget: Option<ResourceBudgetDecl>,
        body: Vec<Stmt>,
        span: Span,
    },
}

/// Cooperative scheduler priority for deterministic task ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Critical,
    High,
    #[default]
    Normal,
    Low,
}

impl TaskPriority {
    pub fn from_ident(ident: &str) -> Option<Self> {
        match ident {
            "critical" => Some(Self::Critical),
            "high" => Some(Self::High),
            "normal" => Some(Self::Normal),
            "low" => Some(Self::Low),
            _ => None,
        }
    }
}

/// Event declaration and handler.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum EventDecl {
    EventDecl {
        name: String,
        #[serde(default)]
        fields: Vec<FieldDecl>,
        span: Span,
    },
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

/// Unified trigger category for reactive execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "category", rename_all = "snake_case")]
pub enum TriggerKind {
    Event {
        name: String,
    },
    Message {
        topic: String,
    },
    Timer {
        interval_ms: f64,
    },
    Condition {
        expr: Expr,
        #[serde(default)]
        level: bool,
    },
    StateEntered {
        state: String,
    },
    StateExited {
        state: String,
    },
    Safety {
        event: String,
    },
    Hardware {
        event: String,
    },
    Ai {
        event: String,
    },
    Verification {
        event: String,
    },
    Twin {
        event: String,
    },
}

/// Unified trigger handler (`on`, `every`, `when` at robot or agent scope).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TriggerHandlerDecl {
    TriggerHandlerDecl {
        trigger_kind: TriggerKind,
        priority: TaskPriority,
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
    VerifyDecl {
        rules: Vec<Expr>,
        #[serde(default)]
        warnings: Vec<Expr>,
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

/// Device identity for signed telemetry and mission logs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IdentityDecl {
    IdentityDecl {
        type_name: String,
        fields: Vec<(String, String)>,
        span: Span,
    },
}

/// Audit block listing fields to record append-only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum AuditDecl {
    AuditDecl {
        name: String,
        records: Vec<Expr>,
        span: Span,
    },
}

/// Provenance configuration for hashing and signing mission records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ProvenanceDecl {
    ProvenanceDecl {
        name: String,
        hash_algo: String,
        signed_by: String,
        span: Span,
    },
}

/// Signed record declaration for telemetry / mission event streams.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SignedRecordDecl {
    SignedRecordDecl {
        event_name: String,
        signed_by: String,
        span: Span,
    },
}

/// Secret declaration resolved at runtime from env or literal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SecretDecl {
    SecretDecl {
        name: String,
        source: SecretSourceDecl,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum SecretSourceDecl {
    Env { var: String },
    Literal { value: String },
}

/// Robot-level trust tier for secure communication and package gating.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TrustDecl {
    TrustDecl { level: String, span: Span },
}

/// Robot-level package capability grants (`permissions [ audit.write, ... ]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PermissionsDecl {
    PermissionsDecl {
        capabilities: Vec<String>,
        span: Span,
    },
}

/// Security policy for topics, services, and actions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecureBlockDecl {
    pub signed: bool,
    pub min_trust: Option<String>,
    pub requires: Vec<String>,
    pub span: Span,
}

impl Default for SecureBlockDecl {
    fn default() -> Self {
        Self {
            signed: false,
            min_trust: None,
            requires: Vec::new(),
            span: Span {
                start: crate::ast::SourceLocation {
                    line: 0,
                    column: 0,
                    offset: 0,
                },
                end: crate::ast::SourceLocation {
                    line: 0,
                    column: 0,
                    offset: 0,
                },
            },
        }
    }
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
            | "navigation.path_planning"
            | "navigation.localize"
            | "navigation.slam"
            | "safety.validate"
            | "ai.reasoning"
            | "ai.openai"
            | "robotics.ros2"
            | "communication.mqtt"
            | "vision.opencv"
            | "vision.yolo"
            | "vision.core"
            | "manipulation.grasp"
            | "hri.dialogue"
            | "twin.sync"
            | "sim.gazebo"
            | "sim.webots"
            | "ledger.mock"
            | "provenance.core"
            | "identity.core"
            | "supply_chain.trace"
            | "std.core"
            | "std.time"
            | "std.units"
            | "std.spatial"
            | "std.math"
            | "std.collections"
            | "std.result"
            | "std.io"
            | "std.log"
            | "std.ai"
            | "std.robotics"
            | "std.sensors"
            | "std.actuators"
            | "std.safety"
            | "std.communication"
            | "std.hardware"
            | "std.sim"
            | "std.twin"
            | "std.hri"
            | "std.security"
            | "std.audit"
            | "std.crypto"
            | "std.network"
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
