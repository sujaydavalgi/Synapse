//! foundations support for Spanda.
//!
use crate::nodes::{Expr, Span, SpandaType, Stmt};
use crate::regex::RegexPattern;
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
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Text result.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.as_str();

        // Dispatch based on the enum variant or current state.
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
        #[serde(default)]
        deadline_ms: Option<f64>,
        #[serde(default)]
        jitter_ms_max: Option<f64>,
        #[serde(default)]
        isolated: bool,
        requires: Option<Expr>,
        ensures: Option<Expr>,
        invariant: Option<Expr>,
        budget: Option<ResourceBudgetDecl>,
        #[serde(default = "default_void_type")]
        return_type: SpandaType,
        body: Vec<Stmt>,
        span: Span,
    },
}

fn default_void_type() -> SpandaType {
    SpandaType::Void
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
        // Construct from ident.
        //
        // Parameters:
        // - `ident` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::foundations::from_ident(ident);

        // Match on ident and handle each case.
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
    LogMatch {
        pattern: RegexPattern,
    },
    MessageMatch {
        field: String,
        pattern: RegexPattern,
    },
    /// Connectivity trigger: `on gps.lost`, `on network.disconnected`, etc.
    Connectivity {
        domain: String,
        event: String,
    },
    /// Geofence trigger: `on geofence SafeZone exited`.
    Geofence {
        name: String,
        phase: String,
    },
    /// Sensor event trigger: `on gps.fix`.
    SensorEvent {
        sensor: String,
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
        #[serde(default = "default_void_type")]
        return_type: SpandaType,
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
        #[serde(default)]
        connectivity: Vec<String>,
        #[serde(default)]
        components: Vec<HardwareComponentDecl>,
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

/// Positioning and wireless connectivity requirements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RequiresConnectivityDecl {
    RequiresConnectivityDecl {
        channels: Vec<(String, spanda_connectivity::ConnectivityRequirement)>,
        latency_ms_max: Option<f64>,
        bandwidth_mbps_min: Option<f64>,
        packet_loss_pct_max: Option<f64>,
        span: Span,
    },
}

/// WGS84 geofence zone for safety verification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum GeofenceDecl {
    GeofenceDecl {
        name: String,
        center_lat: f64,
        center_lon: f64,
        radius_m: f64,
        span: Span,
    },
}

/// Multi-link network failover policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ConnectivityPolicyDecl {
    ConnectivityPolicyDecl {
        name: String,
        preferred: String,
        fallback: String,
        emergency: Option<String>,
        switch_if_latency_ms: Option<f64>,
        switch_if_packet_loss_pct: Option<f64>,
        span: Span,
    },
}

/// Bluetooth discovery and pairing configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum BluetoothConfigDecl {
    BluetoothConfigDecl {
        scan_pattern: Option<RegexPattern>,
        pair_mode: Option<String>,
        span: Span,
    },
}

/// BLE GATT service declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum BleServiceDecl {
    BleServiceDecl {
        name: String,
        uuid: String,
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
        #[serde(default)]
        gpu_pct_max: Option<f64>,
        network_mbps_max: Option<f64>,
        storage_mb_max: Option<f64>,
        span: Span,
    },
}

/// Message subscription filter using regex pattern matching.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubscribeFilterDecl {
    pub field: String,
    pub pattern: RegexPattern,
    pub span: Span,
}

/// Latency-budgeted processing pipeline inside a robot block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PipelineDecl {
    PipelineDecl {
        name: String,
        budget_ms: f64,
        body: Vec<Stmt>,
        span: Span,
    },
}

/// Watchdog handler for hung or missed-heartbeat tasks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum WatchdogDecl {
    WatchdogDecl {
        name: String,
        #[serde(default)]
        target: Option<String>,
        timeout_ms: f64,
        body: Vec<Stmt>,
        span: Span,
    },
}

/// Operating mode with configuration statements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ModeDecl {
    ModeDecl {
        name: String,
        body: Vec<Stmt>,
        span: Span,
    },
}

/// Retry policy with optional fallback block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RetryDecl {
    RetryDecl {
        attempts: u32,
        backoff_ms: f64,
        body: Vec<Stmt>,
        fallback: Vec<Stmt>,
        span: Span,
    },
}

/// Recovery handler for runtime or sensor failures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RecoverDecl {
    RecoverDecl {
        error_name: String,
        body: Vec<Stmt>,
        span: Span,
    },
}

/// Hardware fault handler with fallback actions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum FaultHandlerDecl {
    FaultHandlerDecl {
        fault_type: String,
        body: Vec<Stmt>,
        span: Span,
    },
}

/// Top-level validation rule using regex pattern matching.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ValidateRuleDecl {
    ValidateRuleDecl {
        name: String,
        pattern: RegexPattern,
        span: Span,
    },
}

/// Mission declaration for power budgeting and step-based execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum MissionDecl {
    MissionDecl {
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        duration_hours: Option<f64>,
        #[serde(default)]
        steps: Vec<String>,
        #[serde(default)]
        required_capabilities: Vec<String>,
        span: Span,
    },
}

/// Fault injection scenario for simulation-time compatibility checks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimFaultDecl {
    pub fault_type: String,
    #[serde(default)]
    pub at_offset_ms: Option<f64>,
    #[serde(default)]
    pub duration_ms: Option<f64>,
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

/// World-model belief buffer configuration on a robot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum WorldModelDecl {
    WorldModelDecl { enabled: bool, span: Span },
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
    File { path: String },
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

/// Robot-wide secure communication defaults (`secure_comm { encryption: required; ... }`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SecureCommPolicyDecl {
    SecureCommPolicyDecl {
        encryption: Option<String>,
        authentication: Option<String>,
        integrity: Option<String>,
        span: Span,
    },
}

/// Named trust boundary for cross-domain validation (`trust_boundary robot_to_robot;`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TrustBoundaryDecl {
    TrustBoundaryDecl { name: String, span: Span },
}

/// Security policy for topics, services, and actions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecureBlockDecl {
    pub signed: bool,
    pub signed_required: bool,
    pub min_trust: Option<String>,
    pub requires: Vec<String>,
    pub encryption: Option<String>,
    pub authentication: Option<String>,
    pub integrity: Option<String>,
    pub trusted_sources: Vec<String>,
    pub reject_untrusted: bool,
    pub span: Span,
}

impl Default for SecureBlockDecl {
    fn default() -> Self {
        //
        // Parameters:
        // None.
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_core::foundations::default();

        // Assemble the struct fields and return it.
        Self {
            signed: false,
            signed_required: false,
            min_trust: None,
            requires: Vec::new(),
            encryption: None,
            authentication: None,
            integrity: None,
            trusted_sources: Vec::new(),
            reject_untrusted: false,
            span: Span {
                start: crate::nodes::SourceLocation {
                    line: 0,
                    column: 0,
                    offset: 0,
                },
                end: crate::nodes::SourceLocation {
                    line: 0,
                    column: 0,
                    offset: 0,
                },
            },
        }
    }
}

/// Detailed hardware component with explicit capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HardwareComponentDecl {
    pub component_kind: String,
    pub name: String,
    pub component_type: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub properties: Vec<(String, String)>,
    pub span: Span,
}

/// First-class kill switch declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum KillSwitchDecl {
    KillSwitchDecl {
        name: String,
        source: Option<String>,
        priority: String,
        body: Vec<crate::nodes::Stmt>,
        remote_signed: bool,
        span: Span,
    },
}

/// Single condition in a health check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthCheckCondition {
    pub metric: String,
    pub operator: String,
    pub threshold: String,
    pub span: Span,
}

/// Health check declaration for robots, sensors, actuators, or fleets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum HealthCheckDecl {
    HealthCheckDecl {
        name: String,
        target_kind: String,
        target: String,
        conditions: Vec<HealthCheckCondition>,
        span: Span,
    },
}

/// Single health-policy reaction for a status level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthPolicyReaction {
    pub status: String,
    pub body: Vec<crate::nodes::Stmt>,
}

/// Automatic reaction policy for health status transitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum HealthPolicyDecl {
    HealthPolicyDecl {
        name: String,
        reactions: Vec<HealthPolicyReaction>,
        span: Span,
    },
}

/// Minimum capability requirement with hardware constraints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequiresCapabilityDecl {
    pub capability: String,
    #[serde(default)]
    pub required_by: Option<String>,
    #[serde(default)]
    pub any_of_sensors: Vec<String>,
    #[serde(default)]
    pub any_of_actuators: Vec<String>,
    #[serde(default)]
    pub any_of_connectivity: Vec<String>,
    #[serde(default)]
    pub safety_rules: Vec<String>,
    #[serde(default)]
    pub severity: RequiresCapabilitySeverity,
    pub span: Span,
}

/// Severity for requires_capability checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RequiresCapabilitySeverity {
    #[default]
    Error,
    Warning,
    Info,
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
    // Resolve module import.
    //
    // Parameters:
    // - `path` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::foundations::resolve_module_import(path);

    // Produce matches! as the result.
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
            | "navigation.nav2"
            | "navigation.cartographer"
            | "navigation.rtabmap"
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
            | "provenance.ledger"
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
            | "std.positioning"
            | "std.connectivity"
            | "std.wifi"
            | "std.bluetooth"
            | "std.cellular"
            | "std.geofence"
    )
}

/// Map user-facing type aliases to physical units / builtin types.
pub fn resolve_type_alias(name: &str) -> Option<&'static str> {
    // Resolve type alias.
    //
    // Parameters:
    // - `name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::foundations::resolve_type_alias(name);

    // Match on name and handle each case.
    match name {
        "Distance" | "meter" | "Meter" => Some("distance"),
        "Angle" | "radian" | "Radian" => Some("angle"),
        "Path" => Some("path"),
        "Velocity" => Some("velocity"),
        "Pose" => Some("pose"),
        _ => None,
    }
}
