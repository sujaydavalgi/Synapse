//! lexer support for Spanda.
//!
use crate::ast::UnitKind;
use crate::error::SpandaError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TokenType {
    Import,
    Module,
    Export,
    Public,
    Private,
    Return,
    Async,
    Await,
    Extern,
    Spawn,
    Select,
    Parallel,
    Struct,
    Enum,
    Trait,
    Impl,
    For,
    Match,
    Fn,
    StateMachine,
    Task,
    Skill,
    Event,
    Twin,
    State,
    Resource,
    Requires,
    Ensures,
    Invariant,
    Can,
    Transition,
    Mirror,
    Replay,
    Emit,
    Enter,
    Arrow,
    FatArrow,
    Hal,
    Soc,
    From,
    I2c,
    Spi,
    Uart,
    Gpio,
    Pwm,
    Adc,
    Out,
    In,
    Baud,
    Frequency,
    Pin,
    Robot,
    Node,
    Topic,
    Service,
    Action,
    Sensor,
    Actuator,
    Safety,
    AiModel,
    Agent,
    Uses,
    Tools,
    Goal,
    Plan,
    Memory,
    Provider,
    Behavior,
    Loop,
    While,
    Warning,
    Every,
    Let,
    If,
    Else,
    StopIf,
    Publish,
    Call,
    SendGoal,
    With,
    Zone,
    Circle,
    Rect,
    At,
    Radius,
    Size,
    EmergencyStop,
    ResetEmergencyStop,
    Remember,
    Verify,
    Observe,
    SignedBy,
    Secret,
    Trust,
    Permissions,
    Secure,
    Env,
    Hardware,
    Deploy,
    Dyn,
    Cpu,
    Storage,
    Gpu,
    Battery,
    Capacity,
    Sensors,
    Actuators,
    To,
    RequiresHardware,
    RequiresNetwork,
    SimulateCompatibility,
    Budget,
    Fault,
    Mission,
    Network,
    Bandwidth,
    Latency,
    Timing,
    MinPeriod,
    Duration,
    On,
    Message,
    When,
    Entered,
    Exited,
    Priority,
    Ai,
    Subscribe,
    Execute,
    Discover,
    Bus,
    Device,
    Request,
    Response,
    Feedback,
    Result,
    Qos,
    Reliable,
    BestEffort,
    Rate,
    History,
    Deadline,
    Where,
    Includes,
    Receive,
    Telemetry,
    Faults,
    True,
    False,
    And,
    Or,
    Not,
    Ident,
    Number,
    String,
    UnitLiteral,
    Lbrace,
    Rbrace,
    Lbracket,
    Rbracket,
    Lparen,
    Rparen,
    Semicolon,
    Colon,
    Comma,
    Dot,
    Assign,
    Plus,
    Minus,
    Star,
    Slash,
    Lt,
    Lte,
    Gt,
    Gte,
    Percent,
    Eq,
    Neq,
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitLexeme {
    #[serde(rename = "%VWC")]
    PercentVwc,
    #[serde(rename = "%RH")]
    PercentRh,
    #[serde(rename = "µg/m³")]
    UgPerM3Unicode,
    #[serde(rename = "ug/m3")]
    UgPerM3,
    #[serde(rename = "uS/cm")]
    MicroSPerCm,
    #[serde(rename = "mS/cm")]
    MilliSPerCm,
    #[serde(rename = "uSv/h")]
    MicroSvPerH,
    #[serde(rename = "mSv/h")]
    MilliSvPerH,
    #[serde(rename = "S/m")]
    SPerM,
    #[serde(rename = "cd/m²")]
    CdPerM2,
    #[serde(rename = "cd/m2")]
    CdPerM2Ascii,
    #[serde(rename = "N·m")]
    NewtonMeter,
    #[serde(rename = "kWh")]
    KWh,
    #[serde(rename = "dBA")]
    DBA,
    #[serde(rename = "MHz")]
    MHz,
    #[serde(rename = "km/h")]
    KmPerH,
    #[serde(rename = "m/s²")]
    MPerS2,
    #[serde(rename = "m/s2")]
    MPerS2Ascii,
    #[serde(rename = "m/s")]
    MPerS,
    #[serde(rename = "rad/s")]
    RadPerS,
    #[serde(rename = "deg/s")]
    DegPerS,
    #[serde(rename = "fahrenheit")]
    Fahrenheit,
    #[serde(rename = "celsius")]
    Celsius,
    #[serde(rename = "kelvin")]
    Kelvin,
    #[serde(rename = "kHz")]
    KHz,
    #[serde(rename = "kPa")]
    KPa,
    #[serde(rename = "kN")]
    KN,
    #[serde(rename = "kW")]
    KW,
    #[serde(rename = "kV")]
    KVolt,
    #[serde(rename = "mbar")]
    Mbar,
    #[serde(rename = "mph")]
    Mph,
    #[serde(rename = "gram")]
    Gram,
    #[serde(rename = "mm")]
    Mm,
    #[serde(rename = "cm")]
    Cm,
    #[serde(rename = "km")]
    Km,
    #[serde(rename = "ms")]
    Ms,
    #[serde(rename = "us")]
    Us,
    #[serde(rename = "mV")]
    MVolt,
    #[serde(rename = "mA")]
    MA,
    #[serde(rename = "min")]
    Min,
    #[serde(rename = "deg")]
    Deg,
    #[serde(rename = "rad")]
    Rad,
    #[serde(rename = "psi")]
    Psi,
    #[serde(rename = "bar")]
    Bar,
    #[serde(rename = "Pa")]
    Pa,
    #[serde(rename = "Hz")]
    Hz,
    #[serde(rename = "ft")]
    Ft,
    #[serde(rename = "in")]
    In,
    #[serde(rename = "kg")]
    Kg,
    #[serde(rename = "lb")]
    Lb,
    #[serde(rename = "MW")]
    MW,
    #[serde(rename = "m")]
    M,
    #[serde(rename = "s")]
    S,
    #[serde(rename = "h")]
    H,
    #[serde(rename = "g")]
    G,
    #[serde(rename = "N")]
    N,
    #[serde(rename = "W")]
    W,
    #[serde(rename = "V")]
    V,
    #[serde(rename = "A")]
    A,
    #[serde(rename = "rh")]
    Rh,
    #[serde(rename = "lux")]
    Lux,
    #[serde(rename = "lx")]
    Lx,
    #[serde(rename = "nit")]
    Nit,
    #[serde(rename = "ppm")]
    Ppm,
    #[serde(rename = "ppb")]
    Ppb,
    #[serde(rename = "dB")]
    DB,
    #[serde(rename = "uT")]
    MicroTesla,
    #[serde(rename = "gauss")]
    Gauss,
    #[serde(rename = "rpm")]
    Rpm,
    #[serde(rename = "Nm")]
    Nm,
    #[serde(rename = "J")]
    Joule,
    #[serde(rename = "Wh")]
    Wh,
    #[serde(rename = "uvi")]
    Uvi,
    #[serde(rename = "pH")]
    Ph,
    #[serde(rename = "NTU")]
    Ntu,
    #[serde(rename = "FNU")]
    Fnu,
    #[serde(rename = "ppt")]
    Ppt,
    #[serde(rename = "psu")]
    Psu,
    #[serde(rename = "vwc")]
    Vwc,
}

impl UnitLexeme {
    pub fn as_str(self) -> &'static str {
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
            UnitLexeme::PercentVwc => "%VWC",
            UnitLexeme::PercentRh => "%RH",
            UnitLexeme::UgPerM3Unicode => "µg/m³",
            UnitLexeme::UgPerM3 => "ug/m3",
            UnitLexeme::MicroSPerCm => "uS/cm",
            UnitLexeme::MilliSPerCm => "mS/cm",
            UnitLexeme::MicroSvPerH => "uSv/h",
            UnitLexeme::MilliSvPerH => "mSv/h",
            UnitLexeme::SPerM => "S/m",
            UnitLexeme::CdPerM2 => "cd/m²",
            UnitLexeme::CdPerM2Ascii => "cd/m2",
            UnitLexeme::NewtonMeter => "N·m",
            UnitLexeme::KWh => "kWh",
            UnitLexeme::DBA => "dBA",
            UnitLexeme::MHz => "MHz",
            UnitLexeme::KmPerH => "km/h",
            UnitLexeme::MPerS2 => "m/s²",
            UnitLexeme::MPerS2Ascii => "m/s2",
            UnitLexeme::MPerS => "m/s",
            UnitLexeme::RadPerS => "rad/s",
            UnitLexeme::DegPerS => "deg/s",
            UnitLexeme::Fahrenheit => "fahrenheit",
            UnitLexeme::Celsius => "celsius",
            UnitLexeme::Kelvin => "kelvin",
            UnitLexeme::KHz => "kHz",
            UnitLexeme::KPa => "kPa",
            UnitLexeme::KN => "kN",
            UnitLexeme::KW => "kW",
            UnitLexeme::KVolt => "kV",
            UnitLexeme::Mbar => "mbar",
            UnitLexeme::Mph => "mph",
            UnitLexeme::Gram => "gram",
            UnitLexeme::Mm => "mm",
            UnitLexeme::Cm => "cm",
            UnitLexeme::Km => "km",
            UnitLexeme::Ms => "ms",
            UnitLexeme::Us => "us",
            UnitLexeme::MVolt => "mV",
            UnitLexeme::MA => "mA",
            UnitLexeme::Min => "min",
            UnitLexeme::Deg => "deg",
            UnitLexeme::Rad => "rad",
            UnitLexeme::Psi => "psi",
            UnitLexeme::Bar => "bar",
            UnitLexeme::Pa => "Pa",
            UnitLexeme::Hz => "Hz",
            UnitLexeme::Ft => "ft",
            UnitLexeme::In => "in",
            UnitLexeme::Kg => "kg",
            UnitLexeme::Lb => "lb",
            UnitLexeme::MW => "MW",
            UnitLexeme::M => "m",
            UnitLexeme::S => "s",
            UnitLexeme::H => "h",
            UnitLexeme::G => "g",
            UnitLexeme::N => "N",
            UnitLexeme::W => "W",
            UnitLexeme::V => "V",
            UnitLexeme::A => "A",
            UnitLexeme::Rh => "rh",
            UnitLexeme::Lux => "lux",
            UnitLexeme::Lx => "lx",
            UnitLexeme::Nit => "nit",
            UnitLexeme::Ppm => "ppm",
            UnitLexeme::Ppb => "ppb",
            UnitLexeme::DB => "dB",
            UnitLexeme::MicroTesla => "uT",
            UnitLexeme::Gauss => "gauss",
            UnitLexeme::Rpm => "rpm",
            UnitLexeme::Nm => "Nm",
            UnitLexeme::Joule => "J",
            UnitLexeme::Wh => "Wh",
            UnitLexeme::Uvi => "uvi",
            UnitLexeme::Ph => "pH",
            UnitLexeme::Ntu => "NTU",
            UnitLexeme::Fnu => "FNU",
            UnitLexeme::Ppt => "ppt",
            UnitLexeme::Psu => "psu",
            UnitLexeme::Vwc => "vwc",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    #[serde(rename = "type")]
    pub token_type: TokenType,
    pub lexeme: String,
    pub value: TokenValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<UnitLexeme>,
    pub line: u32,
    pub column: u32,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenValue {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

pub fn unit_from_lexeme(lexeme: UnitLexeme) -> UnitKind {
    // Unit from lexeme.
    //
    // Parameters:
    // - `lexeme` — input value
    //
    // Returns:
    // UnitKind.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::lexer::unit_from_lexeme(lexeme);

    // Produce as str as the result.
    UnitKind::from_lexeme(lexeme.as_str())
}

const UNIT_SUFFIXES: &[UnitLexeme] = &[
    UnitLexeme::PercentVwc,
    UnitLexeme::PercentRh,
    UnitLexeme::UgPerM3Unicode,
    UnitLexeme::UgPerM3,
    UnitLexeme::MicroSPerCm,
    UnitLexeme::MilliSPerCm,
    UnitLexeme::MicroSvPerH,
    UnitLexeme::MilliSvPerH,
    UnitLexeme::SPerM,
    UnitLexeme::CdPerM2Ascii,
    UnitLexeme::CdPerM2,
    UnitLexeme::NewtonMeter,
    UnitLexeme::KWh,
    UnitLexeme::DBA,
    UnitLexeme::MHz,
    UnitLexeme::KmPerH,
    UnitLexeme::MPerS2Ascii,
    UnitLexeme::MPerS2,
    UnitLexeme::MPerS,
    UnitLexeme::RadPerS,
    UnitLexeme::DegPerS,
    UnitLexeme::Fahrenheit,
    UnitLexeme::Celsius,
    UnitLexeme::Kelvin,
    UnitLexeme::KHz,
    UnitLexeme::KPa,
    UnitLexeme::KN,
    UnitLexeme::KW,
    UnitLexeme::KVolt,
    UnitLexeme::Mbar,
    UnitLexeme::Mph,
    UnitLexeme::Gram,
    UnitLexeme::Mm,
    UnitLexeme::Cm,
    UnitLexeme::Km,
    UnitLexeme::Ms,
    UnitLexeme::Us,
    UnitLexeme::MVolt,
    UnitLexeme::MA,
    UnitLexeme::Min,
    UnitLexeme::Deg,
    UnitLexeme::Rad,
    UnitLexeme::Psi,
    UnitLexeme::Bar,
    UnitLexeme::Pa,
    UnitLexeme::Hz,
    UnitLexeme::Ft,
    UnitLexeme::In,
    UnitLexeme::Kg,
    UnitLexeme::Lb,
    UnitLexeme::MW,
    UnitLexeme::M,
    UnitLexeme::S,
    UnitLexeme::H,
    UnitLexeme::G,
    UnitLexeme::N,
    UnitLexeme::W,
    UnitLexeme::V,
    UnitLexeme::A,
    UnitLexeme::Rh,
    UnitLexeme::Lux,
    UnitLexeme::Lx,
    UnitLexeme::Nit,
    UnitLexeme::Ppm,
    UnitLexeme::Ppb,
    UnitLexeme::DB,
    UnitLexeme::MicroTesla,
    UnitLexeme::Gauss,
    UnitLexeme::Rpm,
    UnitLexeme::Nm,
    UnitLexeme::Joule,
    UnitLexeme::Wh,
    UnitLexeme::Uvi,
    UnitLexeme::Ph,
    UnitLexeme::Ntu,
    UnitLexeme::Fnu,
    UnitLexeme::Ppt,
    UnitLexeme::Psu,
    UnitLexeme::Vwc,
];

fn keywords() -> HashMap<&'static str, TokenType> {
    // Keywords.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<&'static str, TokenType>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::lexer::keywords();

    // Produce from as the result.
    HashMap::from([
        ("import", TokenType::Import),
        ("module", TokenType::Module),
        ("export", TokenType::Export),
        ("public", TokenType::Public),
        ("private", TokenType::Private),
        ("return", TokenType::Return),
        ("async", TokenType::Async),
        ("await", TokenType::Await),
        ("extern", TokenType::Extern),
        ("spawn", TokenType::Spawn),
        ("select", TokenType::Select),
        ("parallel", TokenType::Parallel),
        ("struct", TokenType::Struct),
        ("enum", TokenType::Enum),
        ("trait", TokenType::Trait),
        ("impl", TokenType::Impl),
        ("for", TokenType::For),
        ("match", TokenType::Match),
        ("fn", TokenType::Fn),
        ("state_machine", TokenType::StateMachine),
        ("task", TokenType::Task),
        ("skill", TokenType::Skill),
        ("event", TokenType::Event),
        ("twin", TokenType::Twin),
        ("state", TokenType::State),
        ("resource", TokenType::Resource),
        ("requires", TokenType::Requires),
        ("ensures", TokenType::Ensures),
        ("invariant", TokenType::Invariant),
        ("can", TokenType::Can),
        ("transition", TokenType::Transition),
        ("mirror", TokenType::Mirror),
        ("replay", TokenType::Replay),
        ("emit", TokenType::Emit),
        ("enter", TokenType::Enter),
        ("hal", TokenType::Hal),
        ("soc", TokenType::Soc),
        ("from", TokenType::From),
        ("i2c", TokenType::I2c),
        ("spi", TokenType::Spi),
        ("uart", TokenType::Uart),
        ("gpio", TokenType::Gpio),
        ("pwm", TokenType::Pwm),
        ("adc", TokenType::Adc),
        ("out", TokenType::Out),
        ("in", TokenType::In),
        ("baud", TokenType::Baud),
        ("frequency", TokenType::Frequency),
        ("pin", TokenType::Pin),
        ("robot", TokenType::Robot),
        ("node", TokenType::Node),
        ("topic", TokenType::Topic),
        ("service", TokenType::Service),
        ("action", TokenType::Action),
        ("sensor", TokenType::Sensor),
        ("actuator", TokenType::Actuator),
        ("safety", TokenType::Safety),
        ("ai_model", TokenType::AiModel),
        ("agent", TokenType::Agent),
        ("uses", TokenType::Uses),
        ("tools", TokenType::Tools),
        ("goal", TokenType::Goal),
        ("plan", TokenType::Plan),
        ("memory", TokenType::Memory),
        ("provider", TokenType::Provider),
        ("behavior", TokenType::Behavior),
        ("loop", TokenType::Loop),
        ("while", TokenType::While),
        ("warning", TokenType::Warning),
        ("every", TokenType::Every),
        ("let", TokenType::Let),
        ("if", TokenType::If),
        ("else", TokenType::Else),
        ("stop_if", TokenType::StopIf),
        ("publish", TokenType::Publish),
        ("call", TokenType::Call),
        ("send_goal", TokenType::SendGoal),
        ("with", TokenType::With),
        ("zone", TokenType::Zone),
        ("circle", TokenType::Circle),
        ("rect", TokenType::Rect),
        ("at", TokenType::At),
        ("radius", TokenType::Radius),
        ("size", TokenType::Size),
        ("emergency_stop", TokenType::EmergencyStop),
        ("reset_emergency_stop", TokenType::ResetEmergencyStop),
        ("remember", TokenType::Remember),
        ("verify", TokenType::Verify),
        ("observe", TokenType::Observe),
        ("signed_by", TokenType::SignedBy),
        ("secret", TokenType::Secret),
        ("trust", TokenType::Trust),
        ("permissions", TokenType::Permissions),
        ("secure", TokenType::Secure),
        ("env", TokenType::Env),
        ("hardware", TokenType::Hardware),
        ("deploy", TokenType::Deploy),
        ("dyn", TokenType::Dyn),
        ("cpu", TokenType::Cpu),
        ("storage", TokenType::Storage),
        ("gpu", TokenType::Gpu),
        ("battery", TokenType::Battery),
        ("capacity", TokenType::Capacity),
        ("sensors", TokenType::Sensors),
        ("actuators", TokenType::Actuators),
        ("to", TokenType::To),
        ("requires_hardware", TokenType::RequiresHardware),
        ("requires_network", TokenType::RequiresNetwork),
        ("simulate_compatibility", TokenType::SimulateCompatibility),
        ("budget", TokenType::Budget),
        ("fault", TokenType::Fault),
        ("mission", TokenType::Mission),
        ("network", TokenType::Network),
        ("bandwidth", TokenType::Bandwidth),
        ("latency", TokenType::Latency),
        ("timing", TokenType::Timing),
        ("min_period", TokenType::MinPeriod),
        ("duration", TokenType::Duration),
        ("message", TokenType::Message),
        ("subscribe", TokenType::Subscribe),
        ("execute", TokenType::Execute),
        ("discover", TokenType::Discover),
        ("bus", TokenType::Bus),
        ("device", TokenType::Device),
        ("request", TokenType::Request),
        ("response", TokenType::Response),
        ("feedback", TokenType::Feedback),
        ("result", TokenType::Result),
        ("qos", TokenType::Qos),
        ("reliable", TokenType::Reliable),
        ("best_effort", TokenType::BestEffort),
        ("rate", TokenType::Rate),
        ("history", TokenType::History),
        ("deadline", TokenType::Deadline),
        ("where", TokenType::Where),
        ("includes", TokenType::Includes),
        ("receive", TokenType::Receive),
        ("telemetry", TokenType::Telemetry),
        ("faults", TokenType::Faults),
        ("on", TokenType::On),
        ("when", TokenType::When),
        ("entered", TokenType::Entered),
        ("exited", TokenType::Exited),
        ("priority", TokenType::Priority),
        ("ai", TokenType::Ai),
        ("true", TokenType::True),
        ("false", TokenType::False),
        ("and", TokenType::And),
        ("or", TokenType::Or),
        ("not", TokenType::Not),
    ])
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, SpandaError> {
    // Tokenize.
    //
    // Parameters:
    // - `source` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::lexer::tokenize(source);

    // Compute keywords for the following logic.
    let keywords = keywords();
    let mut tokens = Vec::new();
    let mut line: u32 = 1;
    let mut column: u32 = 1;
    let mut i = 0;
    let chars: Vec<char> = source.chars().collect();
    let loc = |line: u32, column: u32, offset: usize| (line, column, offset);

    // Repeat while i < chars.len().
    while i < chars.len() {
        let ch = chars[i];

        // Take the branch when ch equals ' ' || ch == '\t' || ch == '\r'.
        if ch == ' ' || ch == '\t' || ch == '\r' {
            i += 1;
            column += 1;
            continue;
        }

        // Take the branch when ch equals '\n'.
        if ch == '\n' {
            i += 1;
            line += 1;
            column = 1;
            continue;
        }

        // Take the branch when ch equals len.
        if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            // Repeat while i < chars.len() && chars[i] != '\n'.
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        let (start_line, start_column, start_offset) = loc(line, column, i);

        // Match on ch and handle each case.
        match ch {
            '[' => {
                push_single(
                    &mut tokens,
                    TokenType::Lbracket,
                    "[",
                    start_line,
                    start_column,
                    start_offset,
                );
                i += 1;
                column += 1;
            }
            ']' => {
                push_single(
                    &mut tokens,
                    TokenType::Rbracket,
                    "]",
                    start_line,
                    start_column,
                    start_offset,
                );
                i += 1;
                column += 1;
            }
            '{' => {
                push_single(
                    &mut tokens,
                    TokenType::Lbrace,
                    "{",
                    start_line,
                    start_column,
                    start_offset,
                );
                i += 1;
                column += 1;
            }
            '}' => {
                push_single(
                    &mut tokens,
                    TokenType::Rbrace,
                    "}",
                    start_line,
                    start_column,
                    start_offset,
                );
                i += 1;
                column += 1;
            }
            '(' => {
                push_single(
                    &mut tokens,
                    TokenType::Lparen,
                    "(",
                    start_line,
                    start_column,
                    start_offset,
                );
                i += 1;
                column += 1;
            }
            ')' => {
                push_single(
                    &mut tokens,
                    TokenType::Rparen,
                    ")",
                    start_line,
                    start_column,
                    start_offset,
                );
                i += 1;
                column += 1;
            }
            ';' => {
                push_single(
                    &mut tokens,
                    TokenType::Semicolon,
                    ";",
                    start_line,
                    start_column,
                    start_offset,
                );
                i += 1;
                column += 1;
            }
            ':' => {
                push_single(
                    &mut tokens,
                    TokenType::Colon,
                    ":",
                    start_line,
                    start_column,
                    start_offset,
                );
                i += 1;
                column += 1;
            }
            ',' => {
                push_single(
                    &mut tokens,
                    TokenType::Comma,
                    ",",
                    start_line,
                    start_column,
                    start_offset,
                );
                i += 1;
                column += 1;
            }
            '.' => {
                push_single(
                    &mut tokens,
                    TokenType::Dot,
                    ".",
                    start_line,
                    start_column,
                    start_offset,
                );
                i += 1;
                column += 1;
            }
            '+' => {
                push_single(
                    &mut tokens,
                    TokenType::Plus,
                    "+",
                    start_line,
                    start_column,
                    start_offset,
                );
                i += 1;
                column += 1;
            }
            '-' if i + 1 < chars.len() && chars[i + 1] == '>' => {
                tokens.push(Token {
                    token_type: TokenType::Arrow,
                    lexeme: "->".to_string(),
                    value: TokenValue::Null,
                    unit: None,
                    line: start_line,
                    column: start_column,
                    offset: start_offset,
                });
                i += 2;
                column += 2;
            }
            '-' => {
                push_single(
                    &mut tokens,
                    TokenType::Minus,
                    "-",
                    start_line,
                    start_column,
                    start_offset,
                );
                i += 1;
                column += 1;
            }
            '*' => {
                push_single(
                    &mut tokens,
                    TokenType::Star,
                    "*",
                    start_line,
                    start_column,
                    start_offset,
                );
                i += 1;
                column += 1;
            }
            '/' => {
                push_single(
                    &mut tokens,
                    TokenType::Slash,
                    "/",
                    start_line,
                    start_column,
                    start_offset,
                );
                i += 1;
                column += 1;
            }
            '%' => {
                push_single(
                    &mut tokens,
                    TokenType::Percent,
                    "%",
                    start_line,
                    start_column,
                    start_offset,
                );
                i += 1;
                column += 1;
            }
            '<' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                tokens.push(Token {
                    token_type: TokenType::Lte,
                    lexeme: "<=".to_string(),
                    value: TokenValue::Null,
                    unit: None,
                    line: start_line,
                    column: start_column,
                    offset: start_offset,
                });
                i += 2;
                column += 2;
            }
            '<' => {
                push_single(
                    &mut tokens,
                    TokenType::Lt,
                    "<",
                    start_line,
                    start_column,
                    start_offset,
                );
                i += 1;
                column += 1;
            }
            '>' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                tokens.push(Token {
                    token_type: TokenType::Gte,
                    lexeme: ">=".to_string(),
                    value: TokenValue::Null,
                    unit: None,
                    line: start_line,
                    column: start_column,
                    offset: start_offset,
                });
                i += 2;
                column += 2;
            }
            '>' => {
                push_single(
                    &mut tokens,
                    TokenType::Gt,
                    ">",
                    start_line,
                    start_column,
                    start_offset,
                );
                i += 1;
                column += 1;
            }
            '=' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                tokens.push(Token {
                    token_type: TokenType::Eq,
                    lexeme: "==".to_string(),
                    value: TokenValue::Null,
                    unit: None,
                    line: start_line,
                    column: start_column,
                    offset: start_offset,
                });
                i += 2;
                column += 2;
            }
            '!' if i + 1 < chars.len() && chars[i + 1] == '=' => {
                tokens.push(Token {
                    token_type: TokenType::Neq,
                    lexeme: "!=".to_string(),
                    value: TokenValue::Null,
                    unit: None,
                    line: start_line,
                    column: start_column,
                    offset: start_offset,
                });
                i += 2;
                column += 2;
            }
            '=' if i + 1 < chars.len() && chars[i + 1] == '>' => {
                tokens.push(Token {
                    token_type: TokenType::FatArrow,
                    lexeme: "=>".to_string(),
                    value: TokenValue::Null,
                    unit: None,
                    line: start_line,
                    column: start_column,
                    offset: start_offset,
                });
                i += 2;
                column += 2;
            }
            '=' => {
                push_single(
                    &mut tokens,
                    TokenType::Assign,
                    "=",
                    start_line,
                    start_column,
                    start_offset,
                );
                i += 1;
                column += 1;
            }
            '"' => {
                i += 1;
                column += 1;
                let mut value = String::new();

                // Repeat while i < chars.len() && chars[i] != '"'.
                while i < chars.len() && chars[i] != '"' {
                    // Take the branch when chars[i] equals len.
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        value.push(chars[i + 1]);
                        i += 2;
                        column += 2;
                    } else {
                        value.push(chars[i]);
                        i += 1;
                        column += 1;
                    }
                }

                // Take this path when i >= chars.len().
                if i >= chars.len() {
                    return Err(SpandaError::Lexer {
                        message: "Unterminated string".to_string(),
                        line: start_line,
                        column: start_column,
                    });
                }
                i += 1;
                column += 1;
                tokens.push(Token {
                    token_type: TokenType::String,
                    lexeme: value.clone(),
                    value: TokenValue::String(value),
                    unit: None,
                    line: start_line,
                    column: start_column,
                    offset: start_offset,
                });
            }
            '0' if i + 1 < chars.len() && (chars[i + 1] == 'x' || chars[i + 1] == 'X') => {
                i += 2;
                column += 2;
                let mut hex_str = String::new();

                // Repeat while i < chars.len() && is hex digit(chars[i]).
                while i < chars.len() && is_hex_digit(chars[i]) {
                    hex_str.push(chars[i]);
                    i += 1;
                    column += 1;
                }
                let num = i64::from_str_radix(&hex_str, 16).unwrap_or(0) as f64;
                tokens.push(Token {
                    token_type: TokenType::Number,
                    lexeme: format!("0x{hex_str}"),
                    value: TokenValue::Number(num),
                    unit: None,
                    line: start_line,
                    column: start_column,
                    offset: start_offset,
                });
            }
            _ if is_digit(ch) || (ch == '.' && i + 1 < chars.len() && is_digit(chars[i + 1])) => {
                let mut num_str = String::new();

                // Repeat while i < chars.len() && (is digit(chars[i]) || chars[i] == '.').
                while i < chars.len() && (is_digit(chars[i]) || chars[i] == '.') {
                    num_str.push(chars[i]);
                    i += 1;
                    column += 1;
                }
                let num: f64 = num_str.parse().unwrap_or(0.0);

                // Repeat while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t').
                while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t') {
                    i += 1;
                    column += 1;
                }
                let mut matched_unit: Option<UnitLexeme> = None;

                // Iterate over UNIT SUFFIXES.
                for suffix in UNIT_SUFFIXES {
                    let suffix_str = suffix.as_str();
                    let suffix_chars: Vec<char> = suffix_str.chars().collect();

                    // Take this path when i + suffix chars.len() <= chars.len().
                    if i + suffix_chars.len() <= chars.len() {
                        let slice: String = chars[i..i + suffix_chars.len()].iter().collect();

                        // Take the branch when slice equals suffix str.
                        if slice == suffix_str {
                            let next = chars.get(i + suffix_chars.len()).copied();

                            // Emit output when next provides a n.
                            if let Some(n) = next {
                                // Take the branch when is ident char equals '/'.
                                if is_ident_char(n) || n == '/' {
                                    continue;
                                }
                            }
                            matched_unit = Some(*suffix);
                            i += suffix_chars.len();
                            column += suffix_chars.len() as u32;
                            break;
                        }
                    }
                }

                // Emit output when matched unit provides a unit.
                if let Some(unit) = matched_unit {
                    tokens.push(Token {
                        token_type: TokenType::UnitLiteral,
                        lexeme: format!("{num_str}{}", unit.as_str()),
                        value: TokenValue::Number(num),
                        unit: Some(unit),
                        line: start_line,
                        column: start_column,
                        offset: start_offset,
                    });
                } else {
                    tokens.push(Token {
                        token_type: TokenType::Number,
                        lexeme: num_str,
                        value: TokenValue::Number(num),
                        unit: None,
                        line: start_line,
                        column: start_column,
                        offset: start_offset,
                    });
                }
            }
            _ if is_ident_start(ch) => {
                let mut ident = String::new();

                // Repeat while i < chars.len() && is ident char(chars[i]).
                while i < chars.len() && is_ident_char(chars[i]) {
                    ident.push(chars[i]);
                    i += 1;
                    column += 1;
                }
                let token_type = keywords
                    .get(ident.as_str())
                    .copied()
                    .unwrap_or(TokenType::Ident);
                tokens.push(Token {
                    token_type,
                    lexeme: ident.clone(),
                    value: TokenValue::String(ident),
                    unit: None,
                    line: start_line,
                    column: start_column,
                    offset: start_offset,
                });
            }
            _ => {
                return Err(SpandaError::Lexer {
                    message: format!("Unexpected character '{ch}'"),
                    line,
                    column,
                });
            }
        }
    }
    tokens.push(Token {
        token_type: TokenType::Eof,
        lexeme: String::new(),
        value: TokenValue::Null,
        unit: None,
        line,
        column,
        offset: i,
    });
    Ok(tokens)
}

fn is_hex_digit(ch: char) -> bool {
    //
    // Parameters:
    // - `ch` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::lexer::is_hex_digit(ch);

    // Produce contains as the result.
    is_digit(ch) || ('a'..='f').contains(&ch) || ('A'..='F').contains(&ch)
}

fn is_digit(ch: char) -> bool {
    //
    // Parameters:
    // - `ch` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::lexer::is_digit(ch);

    // Produce is ascii digit as the result.
    ch.is_ascii_digit()
}

fn is_ident_start(ch: char) -> bool {
    //
    // Parameters:
    // - `ch` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::lexer::is_ident_start(ch);

    // Produce is ascii alphabetic as the result.
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident_char(ch: char) -> bool {
    //
    // Parameters:
    // - `ch` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::lexer::is_ident_char(ch);

    // Produce is ident start as the result.
    is_ident_start(ch) || is_digit(ch)
}

fn push_single(
    tokens: &mut Vec<Token>,
    token_type: TokenType,
    lexeme: &str,
    line: u32,
    column: u32,
    offset: usize,
) {
    // Push single.
    //
    // Parameters:
    // - `tokens` — input value
    // - `token_type` — input value
    // - `lexeme` — input value
    // - `line` — input value
    // - `column` — input value
    // - `offset` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::lexer::push_single(tokens, token_type, lexeme, line, column, offset);

    // Append into tokens.
    tokens.push(Token {
        token_type,
        lexeme: lexeme.to_string(),
        value: TokenValue::Null,
        unit: None,
        line,
        column,
        offset,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_robot_keywords() {
        // Tokenizes robot keywords.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::lexer::tokenizes_robot_keywords();

        let tokens = tokenize("robot Rover { sensor lidar: Lidar; }").unwrap();
        let types: Vec<_> = tokens.iter().map(|t| t.token_type).collect();
        assert!(types.contains(&TokenType::Robot));
        assert!(types.contains(&TokenType::Sensor));
        assert!(types.contains(&TokenType::Ident));
        assert_eq!(types.last(), Some(&TokenType::Eof));
    }

    #[test]
    fn tokenizes_unit_literals() {
        // Tokenizes unit literals.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::lexer::tokenizes_unit_literals();

        let tokens = tokenize("1.5m/s").unwrap();
        let unit_tok = tokens
            .iter()
            .find(|t| t.token_type == TokenType::UnitLiteral);
        assert!(unit_tok.is_some());
        let t = unit_tok.unwrap();
        assert_eq!(t.value, TokenValue::Number(1.5));
        assert_eq!(t.unit, Some(UnitLexeme::MPerS));
    }

    #[test]
    fn tokenizes_spaced_unit_literals() {
        // Tokenizes spaced unit literals.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::lexer::tokenizes_spaced_unit_literals();

        let tokens = tokenize("1.5 m/s").unwrap();
        let unit_tok = tokens
            .iter()
            .find(|t| t.token_type == TokenType::UnitLiteral);
        assert!(unit_tok.is_some());
    }

    #[test]
    fn tokenizes_duration_units() {
        // Tokenizes duration units.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::lexer::tokenizes_duration_units();

        let tokens = tokenize("loop every 50ms").unwrap();
        let ms_tok = tokens
            .iter()
            .find(|t| t.token_type == TokenType::UnitLiteral);
        assert!(ms_tok.is_some());
        assert_eq!(ms_tok.unwrap().unit, Some(UnitLexeme::Ms));
    }

    #[test]
    fn tokenizes_stop_if() {
        // Tokenizes stop if.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::lexer::tokenizes_stop_if();

        let tokens = tokenize("stop_if x < 0.5 m;").unwrap();
        assert!(tokens.iter().any(|t| t.token_type == TokenType::StopIf));
    }

    #[test]
    fn skips_comments() {
        // Skips comments.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::lexer::skips_comments();

        let tokens = tokenize("// comment\nrobot R {}").unwrap();
        assert_eq!(tokens[0].token_type, TokenType::Robot);
    }

    #[test]
    fn tokenizes_comparisons() {
        // Tokenizes comparisons.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::lexer::tokenizes_comparisons();

        let tokens = tokenize("< <= > >= == !=").unwrap();
        let types: Vec<_> = tokens
            .iter()
            .filter(|t| t.token_type != TokenType::Eof)
            .map(|t| t.token_type)
            .collect();
        assert_eq!(
            types,
            vec![
                TokenType::Lt,
                TokenType::Lte,
                TokenType::Gt,
                TokenType::Gte,
                TokenType::Eq,
                TokenType::Neq,
            ]
        );
    }
}
