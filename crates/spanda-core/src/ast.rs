use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub line: u32,
    pub column: u32,
    pub offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UnitKind {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "m")]
    M,
    #[serde(rename = "s")]
    S,
    #[serde(rename = "ms")]
    Ms,
    #[serde(rename = "rad")]
    Rad,
    #[serde(rename = "m/s")]
    MPerS,
    #[serde(rename = "m/s²")]
    MPerS2,
    #[serde(rename = "rad/s")]
    RadPerS,
    #[serde(rename = "deg")]
    Deg,
    #[serde(rename = "Hz")]
    Hz,
}

impl UnitKind {
    pub fn as_str(self) -> &'static str {
        match self {
            UnitKind::None => "none",
            UnitKind::M => "m",
            UnitKind::S => "s",
            UnitKind::Ms => "ms",
            UnitKind::Rad => "rad",
            UnitKind::MPerS => "m/s",
            UnitKind::MPerS2 => "m/s²",
            UnitKind::RadPerS => "rad/s",
            UnitKind::Deg => "deg",
            UnitKind::Hz => "Hz",
        }
    }

    pub fn from_lexeme(lexeme: &str) -> Self {
        match lexeme {
            "m" => UnitKind::M,
            "s" => UnitKind::S,
            "ms" => UnitKind::Ms,
            "rad" => UnitKind::Rad,
            "m/s" => UnitKind::MPerS,
            "m/s²" | "m/s2" => UnitKind::MPerS2,
            "rad/s" => UnitKind::RadPerS,
            "deg" => UnitKind::Deg,
            "Hz" => UnitKind::Hz,
            _ => UnitKind::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SpandaType {
    #[serde(rename = "void")]
    Void,
    #[serde(rename = "int")]
    Int,
    #[serde(rename = "float")]
    Float,
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "number")]
    Number { unit: UnitKind },
    #[serde(rename = "string")]
    String,
    #[serde(rename = "char")]
    Char,
    #[serde(rename = "bytes")]
    Bytes,
    #[serde(rename = "null")]
    Null,
    #[serde(rename = "named")]
    Named { name: String },
    #[serde(rename = "generic")]
    Generic {
        name: String,
        type_args: Vec<SpandaType>,
    },
    #[serde(rename = "scan")]
    Scan,
    #[serde(rename = "pose")]
    Pose,
    #[serde(rename = "velocity")]
    Velocity,
    #[serde(rename = "trajectory")]
    Trajectory,
    #[serde(rename = "transform")]
    Transform,
    #[serde(rename = "enum_variant")]
    EnumVariant { enum_name: String, variant: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Program {
    Program {
        module_name: Option<String>,
        imports: Vec<ImportDecl>,
        structs: Vec<crate::foundations::StructDecl>,
        enums: Vec<crate::foundations::EnumDecl>,
        traits: Vec<crate::foundations::TraitDecl>,
        hardware_profiles: Vec<crate::foundations::HardwareDecl>,
        deployments: Vec<crate::foundations::DeployDecl>,
        requires_hardware: Option<crate::foundations::RequiresHardwareDecl>,
        requires_network: Option<crate::foundations::RequiresNetworkDecl>,
        simulate_compatibility: Option<crate::foundations::SimulateCompatibilityDecl>,
        robots: Vec<RobotDecl>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ImportDecl {
    ImportDecl { path: String, span: Span },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RobotDecl {
    RobotDecl {
        name: String,
        soc: Option<SocDecl>,
        hal: Option<HalBlock>,
        nodes: Vec<NodeDecl>,
        topics: Vec<TopicDecl>,
        services: Vec<ServiceDecl>,
        actions: Vec<ActionDecl>,
        sensors: Vec<SensorDecl>,
        actuators: Vec<ActuatorDecl>,
        safety: Option<SafetyBlock>,
        ai_models: Vec<AiModelDecl>,
        agents: Vec<AgentDecl>,
        behaviors: Vec<BehaviorDecl>,
        tasks: Vec<crate::foundations::TaskDecl>,
        state_machines: Vec<crate::foundations::StateMachineDecl>,
        events: Vec<crate::foundations::EventDecl>,
        event_handlers: Vec<crate::foundations::EventHandlerDecl>,
        twin: Option<crate::foundations::TwinDecl>,
        observe: Option<crate::foundations::ObserveDecl>,
        verify: Option<crate::foundations::VerifyDecl>,
        requires_hardware: Option<crate::foundations::RequiresHardwareDecl>,
        requires_network: Option<crate::foundations::RequiresNetworkDecl>,
        mission: Option<crate::foundations::MissionDecl>,
        trait_impls: Vec<crate::foundations::TraitImplDecl>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SocDecl {
    SocDecl { profile: String, span: Span },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum HalBlock {
    HalBlock {
        members: Vec<HalMemberDecl>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum HalMemberDecl {
    HalI2cDecl {
        name: String,
        address: f64,
        span: Span,
    },
    HalSpiDecl {
        name: String,
        bus: f64,
        cs_pin: Option<f64>,
        span: Span,
    },
    HalGpioDecl {
        name: String,
        direction: GpioDirection,
        pin: f64,
        span: Span,
    },
    HalPwmDecl {
        name: String,
        pin: f64,
        frequency_hz: f64,
        span: Span,
    },
    HalUartDecl {
        name: String,
        device: String,
        baud: f64,
        span: Span,
    },
    HalAdcDecl {
        name: String,
        channel: f64,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GpioDirection {
    In,
    Out,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum NodeDecl {
    NodeDecl {
        name: String,
        namespace: Option<String>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TopicDecl {
    TopicDecl {
        name: String,
        message_type: String,
        topic: String,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ServiceDecl {
    ServiceDecl {
        name: String,
        service_type: String,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ActionDecl {
    ActionDecl {
        name: String,
        action_type: String,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SensorDecl {
    SensorDecl {
        name: String,
        sensor_type: String,
        library: Option<String>,
        binding: Option<SensorBinding>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SensorBinding {
    #[serde(rename = "topic")]
    Topic { path: String },
    #[serde(rename = "hal")]
    Hal { bus_name: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ActuatorDecl {
    ActuatorDecl {
        name: String,
        actuator_type: String,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SafetyBlock {
    SafetyBlock {
        rules: Vec<SafetyRule>,
        zones: Vec<SafetyZoneDecl>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiConfigEntry {
    pub key: String,
    pub value: ConfigValue,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    String(String),
    Number(f64),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum AiModelDecl {
    AiModelDecl {
        name: String,
        model_type: String,
        config: Vec<AiConfigEntry>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum AgentDecl {
    AgentDecl {
        name: String,
        uses_ai: Vec<String>,
        memory_kind: Option<MemoryKind>,
        tools: Vec<String>,
        skills: Vec<String>,
        capabilities: Vec<crate::foundations::CapabilityDecl>,
        goal: String,
        plan_body: Vec<Stmt>,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind {
    ShortTerm,
    LongTerm,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SafetyRule {
    MaxSpeedRule {
        name: String,
        value: Expr,
        unit: UnitKind,
        span: Span,
    },
    StopIfRule {
        condition: Expr,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SafetyZoneDecl {
    SafetyZoneDecl {
        name: String,
        shape: ZoneShape,
        x: Expr,
        y: Expr,
        radius: Option<Expr>,
        width: Option<Expr>,
        height: Option<Expr>,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ZoneShape {
    Circle,
    Rect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum BehaviorDecl {
    BehaviorDecl {
        name: String,
        requires: Option<Expr>,
        ensures: Option<Expr>,
        invariant: Option<Expr>,
        body: Vec<Stmt>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Stmt {
    VarDecl {
        name: String,
        type_annotation: Option<SpandaType>,
        init: Option<Expr>,
        span: Span,
    },
    IfStmt {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
        span: Span,
    },
    LoopStmt {
        interval_ms: f64,
        body: Vec<Stmt>,
        span: Span,
    },
    ExprStmt {
        expr: Expr,
        span: Span,
    },
    ReturnStmt {
        value: Option<Expr>,
        span: Span,
    },
    PublishStmt {
        topic_name: String,
        value: Expr,
        span: Span,
    },
    ServiceCallStmt {
        service_name: String,
        span: Span,
    },
    ActionSendStmt {
        action_name: String,
        goal: Expr,
        span: Span,
    },
    EmergencyStopStmt {
        span: Span,
    },
    ResetEmergencyStopStmt {
        span: Span,
    },
    EmitStmt {
        event_name: String,
        span: Span,
    },
    EnterStmt {
        state_name: String,
        span: Span,
    },
    RememberStmt {
        key: String,
        value: Expr,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Expr {
    LiteralExpr {
        value: LiteralValue,
        span: Span,
    },
    UnitLiteralExpr {
        value: f64,
        unit: UnitKind,
        span: Span,
    },
    IdentExpr {
        name: String,
        span: Span,
    },
    BinaryExpr {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    UnaryExpr {
        op: UnaryOp,
        operand: Box<Expr>,
        span: Span,
    },
    CallExpr {
        callee: Box<Expr>,
        args: Vec<Expr>,
        named_args: Vec<NamedArg>,
        span: Span,
    },
    MemberExpr {
        object: Box<Expr>,
        property: String,
        span: Span,
    },
    MatchExpr {
        scrutinee: Box<Expr>,
        arms: Vec<crate::foundations::MatchArm>,
        span: Span,
    },
    StructLiteralExpr {
        type_name: String,
        fields: Vec<StructFieldInit>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructFieldInit {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LiteralValue {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamedArg {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    #[serde(rename = "+")]
    Add,
    #[serde(rename = "-")]
    Sub,
    #[serde(rename = "*")]
    Mul,
    #[serde(rename = "/")]
    Div,
    #[serde(rename = "<")]
    Lt,
    #[serde(rename = "<=")]
    Lte,
    #[serde(rename = ">")]
    Gt,
    #[serde(rename = ">=")]
    Gte,
    #[serde(rename = "==")]
    Eq,
    #[serde(rename = "!=")]
    Neq,
    #[serde(rename = "and")]
    And,
    #[serde(rename = "or")]
    Or,
}

impl BinaryOp {
    pub fn from_lexeme(lexeme: &str) -> Option<Self> {
        match lexeme {
            "+" => Some(BinaryOp::Add),
            "-" => Some(BinaryOp::Sub),
            "*" => Some(BinaryOp::Mul),
            "/" => Some(BinaryOp::Div),
            "<" => Some(BinaryOp::Lt),
            "<=" => Some(BinaryOp::Lte),
            ">" => Some(BinaryOp::Gt),
            ">=" => Some(BinaryOp::Gte),
            "==" => Some(BinaryOp::Eq),
            "!=" => Some(BinaryOp::Neq),
            "and" => Some(BinaryOp::And),
            "or" => Some(BinaryOp::Or),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Lt => "<",
            BinaryOp::Lte => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Gte => ">=",
            BinaryOp::Eq => "==",
            BinaryOp::Neq => "!=",
            BinaryOp::And => "and",
            BinaryOp::Or => "or",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    #[serde(rename = "-")]
    Neg,
    #[serde(rename = "not")]
    Not,
}

pub const MESSAGE_TYPES: &[&str] = &["Velocity", "Pose", "Scan", "String"];
pub const SERVICE_TYPES: &[&str] = &["ResetCostmap", "ClearCostmap", "SetPose"];
pub const ACTION_TYPES: &[&str] = &["NavigateTo", "FollowPath", "PickObject"];

// Helpers to access inner fields ergonomically
impl Program {
    pub fn imports(&self) -> &[ImportDecl] {
        match self {
            Program::Program { imports, .. } => imports,
        }
    }

    pub fn robots(&self) -> &[RobotDecl] {
        match self {
            Program::Program { robots, .. } => robots,
        }
    }
}

impl RobotDecl {
    pub fn name(&self) -> &str {
        match self {
            RobotDecl::RobotDecl { name, .. } => name,
        }
    }
}
