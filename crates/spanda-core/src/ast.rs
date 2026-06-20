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
    // Distance (canonical: m)
    #[serde(rename = "m")]
    M,
    #[serde(rename = "mm")]
    Mm,
    #[serde(rename = "cm")]
    Cm,
    #[serde(rename = "km")]
    Km,
    #[serde(rename = "ft")]
    Ft,
    #[serde(rename = "in")]
    In,
    // Duration (canonical: s)
    #[serde(rename = "s")]
    S,
    #[serde(rename = "ms")]
    Ms,
    #[serde(rename = "us")]
    Us,
    #[serde(rename = "min")]
    Min,
    #[serde(rename = "h")]
    H,
    // Velocity (canonical: m/s)
    #[serde(rename = "m/s")]
    MPerS,
    #[serde(rename = "km/h")]
    KmPerH,
    #[serde(rename = "mph")]
    Mph,
    // Acceleration (canonical: m/s²)
    #[serde(rename = "m/s²")]
    MPerS2,
    #[serde(rename = "g")]
    G,
    // Angle (canonical: rad)
    #[serde(rename = "rad")]
    Rad,
    #[serde(rename = "deg")]
    Deg,
    // Angular velocity (canonical: rad/s)
    #[serde(rename = "rad/s")]
    RadPerS,
    #[serde(rename = "deg/s")]
    DegPerS,
    // Mass (canonical: kg)
    #[serde(rename = "kg")]
    Kg,
    #[serde(rename = "gram")]
    Gram,
    #[serde(rename = "lb")]
    Lb,
    // Force (canonical: N)
    #[serde(rename = "N")]
    N,
    #[serde(rename = "kN")]
    KN,
    // Power (canonical: W)
    #[serde(rename = "W")]
    W,
    #[serde(rename = "kW")]
    KW,
    #[serde(rename = "MW")]
    MW,
    // Voltage (canonical: V)
    #[serde(rename = "V")]
    V,
    #[serde(rename = "mV")]
    MVolt,
    #[serde(rename = "kV")]
    KVolt,
    // Current (canonical: A)
    #[serde(rename = "A")]
    A,
    #[serde(rename = "mA")]
    MA,
    // Temperature (canonical: celsius)
    #[serde(rename = "celsius")]
    Celsius,
    #[serde(rename = "fahrenheit")]
    Fahrenheit,
    #[serde(rename = "kelvin")]
    Kelvin,
    // Pressure (canonical: Pa)
    #[serde(rename = "Pa")]
    Pa,
    #[serde(rename = "kPa")]
    KPa,
    #[serde(rename = "bar")]
    Bar,
    #[serde(rename = "psi")]
    Psi,
    #[serde(rename = "mbar")]
    Mbar,
    // Frequency (canonical: Hz)
    #[serde(rename = "Hz")]
    Hz,
    #[serde(rename = "kHz")]
    KHz,
    #[serde(rename = "MHz")]
    MHz,
    // Humidity (canonical: rh, 0–100 %RH)
    #[serde(rename = "rh")]
    Rh,
    #[serde(rename = "%RH")]
    PercentRh,
    // Illuminance (canonical: lux)
    #[serde(rename = "lux")]
    Lux,
    #[serde(rename = "lx")]
    Lx,
    // Luminance (canonical: cd/m²)
    #[serde(rename = "cd/m²")]
    CdPerM2,
    #[serde(rename = "nit")]
    Nit,
    // Gas concentration (canonical: ppm)
    #[serde(rename = "ppm")]
    Ppm,
    #[serde(rename = "ppb")]
    Ppb,
    // Sound level (canonical: dB)
    #[serde(rename = "dB")]
    DB,
    #[serde(rename = "dBA")]
    DBA,
    // Magnetic field (canonical: uT)
    #[serde(rename = "uT")]
    MicroTesla,
    #[serde(rename = "gauss")]
    Gauss,
    // Rotational speed (canonical: rpm)
    #[serde(rename = "rpm")]
    Rpm,
    // Torque (canonical: N·m)
    #[serde(rename = "N·m")]
    NewtonMeter,
    #[serde(rename = "Nm")]
    Nm,
    // Energy (canonical: J)
    #[serde(rename = "J")]
    Joule,
    #[serde(rename = "Wh")]
    Wh,
    #[serde(rename = "kWh")]
    KWh,
    // UV index (canonical: uvi)
    #[serde(rename = "uvi")]
    Uvi,
    // Acidity (canonical: pH)
    #[serde(rename = "pH")]
    Ph,
    // Conductivity (canonical: uS/cm)
    #[serde(rename = "uS/cm")]
    MicroSPerCm,
    #[serde(rename = "mS/cm")]
    MilliSPerCm,
    #[serde(rename = "S/m")]
    SPerM,
    // Particulate matter (canonical: ug/m3)
    #[serde(rename = "ug/m3")]
    UgPerM3,
    // Turbidity (canonical: NTU)
    #[serde(rename = "NTU")]
    Ntu,
    #[serde(rename = "FNU")]
    Fnu,
    // Salinity (canonical: ppt)
    #[serde(rename = "ppt")]
    Ppt,
    #[serde(rename = "psu")]
    Psu,
    // Radiation dose rate (canonical: uSv/h)
    #[serde(rename = "uSv/h")]
    MicroSvPerH,
    #[serde(rename = "mSv/h")]
    MilliSvPerH,
    // Soil moisture (canonical: %VWC)
    #[serde(rename = "%VWC")]
    PercentVwc,
    #[serde(rename = "vwc")]
    Vwc,
}

impl UnitKind {
    pub fn as_str(self) -> &'static str {
        match self {
            UnitKind::None => "none",
            UnitKind::M => "m",
            UnitKind::Mm => "mm",
            UnitKind::Cm => "cm",
            UnitKind::Km => "km",
            UnitKind::Ft => "ft",
            UnitKind::In => "in",
            UnitKind::S => "s",
            UnitKind::Ms => "ms",
            UnitKind::Us => "us",
            UnitKind::Min => "min",
            UnitKind::H => "h",
            UnitKind::MPerS => "m/s",
            UnitKind::KmPerH => "km/h",
            UnitKind::Mph => "mph",
            UnitKind::MPerS2 => "m/s²",
            UnitKind::G => "g",
            UnitKind::Rad => "rad",
            UnitKind::Deg => "deg",
            UnitKind::RadPerS => "rad/s",
            UnitKind::DegPerS => "deg/s",
            UnitKind::Kg => "kg",
            UnitKind::Gram => "gram",
            UnitKind::Lb => "lb",
            UnitKind::N => "N",
            UnitKind::KN => "kN",
            UnitKind::W => "W",
            UnitKind::KW => "kW",
            UnitKind::MW => "MW",
            UnitKind::V => "V",
            UnitKind::MVolt => "mV",
            UnitKind::KVolt => "kV",
            UnitKind::A => "A",
            UnitKind::MA => "mA",
            UnitKind::Celsius => "celsius",
            UnitKind::Fahrenheit => "fahrenheit",
            UnitKind::Kelvin => "kelvin",
            UnitKind::Pa => "Pa",
            UnitKind::KPa => "kPa",
            UnitKind::Bar => "bar",
            UnitKind::Psi => "psi",
            UnitKind::Mbar => "mbar",
            UnitKind::Hz => "Hz",
            UnitKind::KHz => "kHz",
            UnitKind::MHz => "MHz",
            UnitKind::Rh => "rh",
            UnitKind::PercentRh => "%RH",
            UnitKind::Lux => "lux",
            UnitKind::Lx => "lx",
            UnitKind::CdPerM2 => "cd/m²",
            UnitKind::Nit => "nit",
            UnitKind::Ppm => "ppm",
            UnitKind::Ppb => "ppb",
            UnitKind::DB => "dB",
            UnitKind::DBA => "dBA",
            UnitKind::MicroTesla => "uT",
            UnitKind::Gauss => "gauss",
            UnitKind::Rpm => "rpm",
            UnitKind::NewtonMeter => "N·m",
            UnitKind::Nm => "Nm",
            UnitKind::Joule => "J",
            UnitKind::Wh => "Wh",
            UnitKind::KWh => "kWh",
            UnitKind::Uvi => "uvi",
            UnitKind::Ph => "pH",
            UnitKind::MicroSPerCm => "uS/cm",
            UnitKind::MilliSPerCm => "mS/cm",
            UnitKind::SPerM => "S/m",
            UnitKind::UgPerM3 => "ug/m3",
            UnitKind::Ntu => "NTU",
            UnitKind::Fnu => "FNU",
            UnitKind::Ppt => "ppt",
            UnitKind::Psu => "psu",
            UnitKind::MicroSvPerH => "uSv/h",
            UnitKind::MilliSvPerH => "mSv/h",
            UnitKind::PercentVwc => "%VWC",
            UnitKind::Vwc => "vwc",
        }
    }

    pub fn from_lexeme(lexeme: &str) -> Self {
        match lexeme {
            "m" => UnitKind::M,
            "mm" => UnitKind::Mm,
            "cm" => UnitKind::Cm,
            "km" => UnitKind::Km,
            "ft" => UnitKind::Ft,
            "in" => UnitKind::In,
            "s" => UnitKind::S,
            "ms" => UnitKind::Ms,
            "us" => UnitKind::Us,
            "min" => UnitKind::Min,
            "h" => UnitKind::H,
            "m/s" => UnitKind::MPerS,
            "km/h" => UnitKind::KmPerH,
            "mph" => UnitKind::Mph,
            "m/s²" | "m/s2" => UnitKind::MPerS2,
            "g" => UnitKind::G,
            "rad" => UnitKind::Rad,
            "deg" => UnitKind::Deg,
            "rad/s" => UnitKind::RadPerS,
            "deg/s" => UnitKind::DegPerS,
            "kg" => UnitKind::Kg,
            "gram" => UnitKind::Gram,
            "lb" => UnitKind::Lb,
            "N" => UnitKind::N,
            "kN" => UnitKind::KN,
            "W" => UnitKind::W,
            "kW" => UnitKind::KW,
            "MW" => UnitKind::MW,
            "V" => UnitKind::V,
            "mV" => UnitKind::MVolt,
            "kV" => UnitKind::KVolt,
            "A" => UnitKind::A,
            "mA" => UnitKind::MA,
            "celsius" => UnitKind::Celsius,
            "fahrenheit" => UnitKind::Fahrenheit,
            "kelvin" => UnitKind::Kelvin,
            "Pa" => UnitKind::Pa,
            "kPa" => UnitKind::KPa,
            "bar" => UnitKind::Bar,
            "psi" => UnitKind::Psi,
            "mbar" => UnitKind::Mbar,
            "Hz" => UnitKind::Hz,
            "kHz" => UnitKind::KHz,
            "MHz" => UnitKind::MHz,
            "rh" => UnitKind::Rh,
            "%RH" => UnitKind::PercentRh,
            "lux" => UnitKind::Lux,
            "lx" => UnitKind::Lx,
            "cd/m²" | "cd/m2" => UnitKind::CdPerM2,
            "nit" => UnitKind::Nit,
            "ppm" => UnitKind::Ppm,
            "ppb" => UnitKind::Ppb,
            "dB" => UnitKind::DB,
            "dBA" => UnitKind::DBA,
            "uT" => UnitKind::MicroTesla,
            "gauss" => UnitKind::Gauss,
            "rpm" => UnitKind::Rpm,
            "N·m" | "Nm" => UnitKind::NewtonMeter,
            "J" => UnitKind::Joule,
            "Wh" => UnitKind::Wh,
            "kWh" => UnitKind::KWh,
            "uvi" => UnitKind::Uvi,
            "pH" => UnitKind::Ph,
            "uS/cm" => UnitKind::MicroSPerCm,
            "mS/cm" => UnitKind::MilliSPerCm,
            "S/m" => UnitKind::SPerM,
            "ug/m3" | "µg/m³" => UnitKind::UgPerM3,
            "NTU" => UnitKind::Ntu,
            "FNU" => UnitKind::Fnu,
            "ppt" => UnitKind::Ppt,
            "psu" => UnitKind::Psu,
            "uSv/h" => UnitKind::MicroSvPerH,
            "mSv/h" => UnitKind::MilliSvPerH,
            "%VWC" => UnitKind::PercentVwc,
            "vwc" => UnitKind::Vwc,
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
    #[serde(rename = "type_param")]
    TypeParam { name: String },
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
        #[serde(default)]
        functions: Vec<crate::foundations::ModuleFnDecl>,
        #[serde(default)]
        tests: Vec<crate::foundations::TestDecl>,
        structs: Vec<crate::foundations::StructDecl>,
        enums: Vec<crate::foundations::EnumDecl>,
        traits: Vec<crate::foundations::TraitDecl>,
        hardware_profiles: Vec<crate::foundations::HardwareDecl>,
        deployments: Vec<crate::foundations::DeployDecl>,
        requires_hardware: Option<crate::foundations::RequiresHardwareDecl>,
        requires_network: Option<crate::foundations::RequiresNetworkDecl>,
        simulate_compatibility: Option<crate::foundations::SimulateCompatibilityDecl>,
        messages: Vec<crate::comm::MessageDecl>,
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
        identity: Option<crate::foundations::IdentityDecl>,
        audit: Option<crate::foundations::AuditDecl>,
        provenance: Option<crate::foundations::ProvenanceDecl>,
        signed_records: Vec<crate::foundations::SignedRecordDecl>,
        secrets: Vec<crate::foundations::SecretDecl>,
        trust: Option<crate::foundations::TrustDecl>,
        permissions: Option<crate::foundations::PermissionsDecl>,
        requires_hardware: Option<crate::foundations::RequiresHardwareDecl>,
        requires_network: Option<crate::foundations::RequiresNetworkDecl>,
        mission: Option<crate::foundations::MissionDecl>,
        trait_impls: Vec<crate::foundations::TraitImplDecl>,
        buses: Vec<crate::comm::BusDecl>,
        peer_robots: Vec<crate::comm::PeerRobotDecl>,
        devices: Vec<crate::comm::DeviceDecl>,
        agent_channels: Vec<crate::comm::AgentChannelDecl>,
        twin_sync: Option<crate::comm::TwinSyncDecl>,
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
        #[serde(default)]
        topic: Option<String>,
        #[serde(default)]
        role: crate::comm::TopicRole,
        #[serde(default)]
        qos: Option<crate::comm::QosDecl>,
        #[serde(default)]
        transport: Option<crate::comm::TransportKind>,
        #[serde(default)]
        secure: Option<crate::foundations::SecureBlockDecl>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ServiceDecl {
    ServiceDecl {
        name: String,
        #[serde(default)]
        service_type: Option<String>,
        #[serde(default)]
        request_type: Option<String>,
        #[serde(default)]
        response_type: Option<String>,
        #[serde(default)]
        secure: Option<crate::foundations::SecureBlockDecl>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ActionDecl {
    ActionDecl {
        name: String,
        #[serde(default)]
        action_type: Option<String>,
        #[serde(default)]
        request_type: Option<String>,
        #[serde(default)]
        feedback_type: Option<String>,
        #[serde(default)]
        result_type: Option<String>,
        #[serde(default)]
        secure: Option<crate::foundations::SecureBlockDecl>,
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
    SubscribeStmt {
        target: String,
        span: Span,
    },
    ExecuteStmt {
        action_name: String,
        goal: Expr,
        span: Span,
    },
    DiscoverStmt {
        target: crate::comm::DiscoverTarget,
        filter: Option<crate::comm::DiscoverFilter>,
        span: Span,
    },
    ReceiveStmt {
        topic_name: String,
        var_name: String,
        span: Span,
    },
    SpawnStmt {
        callee: Expr,
        args: Vec<Expr>,
        span: Span,
    },
    SelectStmt {
        arms: Vec<crate::foundations::SelectArm>,
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
    ServiceCallExpr {
        service_name: String,
        span: Span,
    },
    ExecuteExpr {
        action_name: String,
        goal: Box<Expr>,
        span: Span,
    },
    DiscoverExpr {
        target: crate::comm::DiscoverTarget,
        filter: Option<crate::comm::DiscoverFilter>,
        span: Span,
    },
    AwaitExpr {
        operand: Box<Expr>,
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
