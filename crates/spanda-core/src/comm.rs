//! First-class distributed communication framework for Spanda.
//!
//! Architecture: Message → Topic/Service/Action/Event → Capability → Safety → Transport

use crate::ast::{Span, SpandaType};
use crate::foundations::FieldDecl;
use crate::runtime::RuntimeValue;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

// ── Transport & QoS ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportKind {
    Local,
    Ros2,
    Mqtt,
    Dds,
    Websocket,
    Sim,
}

impl TransportKind {
    pub fn from_ident(s: &str) -> Option<Self> {
        match s {
            "local" => Some(Self::Local),
            "ros2" => Some(Self::Ros2),
            "mqtt" => Some(Self::Mqtt),
            "dds" => Some(Self::Dds),
            "websocket" => Some(Self::Websocket),
            "sim" => Some(Self::Sim),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Ros2 => "ros2",
            Self::Mqtt => "mqtt",
            Self::Dds => "dds",
            Self::Websocket => "websocket",
            Self::Sim => "sim",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QosReliability {
    Reliable,
    BestEffort,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QosDecl {
    pub reliability: Option<QosReliability>,
    pub rate_hz: Option<f64>,
    pub deadline_ms: Option<f64>,
    pub history: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TopicRole {
    #[default]
    Publish,
    Subscribe,
    Both,
}

// ── Message schema registry ──────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum MessageDecl {
    MessageDecl {
        name: String,
        fields: Vec<FieldDecl>,
        version: Option<u32>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageSchema {
    pub name: String,
    pub fields: Vec<(String, String)>,
    pub version: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct MessageRegistry {
    schemas: HashMap<String, MessageSchema>,
    builtin: HashSet<String>,
}

impl MessageRegistry {
    pub fn new() -> Self {
        let mut reg = Self::default();
        for name in ["Velocity", "Pose", "Scan", "String"] {
            reg.builtin.insert(name.into());
        }
        reg
    }

    pub fn register(&mut self, decl: &MessageDecl) {
        let MessageDecl::MessageDecl {
            name,
            fields,
            version,
            ..
        } = decl;
        self.schemas.insert(
            name.clone(),
            MessageSchema {
                name: name.clone(),
                fields: fields
                    .iter()
                    .map(|f| (f.name.clone(), f.type_name.clone()))
                    .collect(),
                version: *version,
            },
        );
    }

    pub fn from_program(
        messages: &[MessageDecl],
        structs: &[crate::foundations::StructDecl],
    ) -> Self {
        let mut reg = Self::new();
        for msg in messages {
            reg.register(msg);
        }
        for s in structs {
            let crate::foundations::StructDecl::StructDecl { name, fields, .. } = s;
            reg.schemas.insert(
                name.clone(),
                MessageSchema {
                    name: name.clone(),
                    fields: fields
                        .iter()
                        .map(|f| (f.name.clone(), f.type_name.clone()))
                        .collect(),
                    version: None,
                },
            );
        }
        reg
    }

    pub fn is_known(&self, name: &str) -> bool {
        self.builtin.contains(name) || self.schemas.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&MessageSchema> {
        self.schemas.get(name)
    }

    pub fn resolve_type(&self, name: &str) -> Option<SpandaType> {
        match name {
            "Velocity" => Some(SpandaType::Velocity),
            "Pose" => Some(SpandaType::Pose),
            "Scan" => Some(SpandaType::Scan),
            "String" => Some(SpandaType::String),
            "Command" | "Conversation" | "Feedback" | "Approval" | "Intent" => {
                Some(SpandaType::Named { name: name.into() })
            }
            "SafeMessage" | "VerifiedMessage" | "TrustedSource" | "ActionProposal"
            | "SafeAction" | "CommandMessage" => Some(SpandaType::Named { name: name.into() }),
            "BatteryRequest" | "BatteryStatus" | "NavigationFeedback" | "NavigationResult"
            | "LidarReading" | "LidarScan" | "Timestamp" | "PathPlan" => {
                Some(SpandaType::Named { name: name.into() })
            }
            other if self.schemas.contains_key(other) => {
                Some(SpandaType::Named { name: other.into() })
            }
            _ => None,
        }
    }
}

// ── Communication declarations ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum BusDecl {
    BusDecl {
        name: String,
        transport: TransportKind,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PeerRobotDecl {
    PeerRobotDecl { name: String, span: Span },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum DeviceDecl {
    DeviceDecl {
        name: String,
        device_type: String,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum AgentChannelDecl {
    AgentChannelDecl {
        from_agent: String,
        to_agent: String,
        message_type: String,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TwinSyncDecl {
    TwinSyncDecl {
        telemetry: bool,
        replay: bool,
        faults: bool,
        events: bool,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum DiscoverTarget {
    Robots,
    Agents,
    Devices,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscoverFilter {
    pub capability: Option<String>,
}

// ── Enhanced service/action shapes (used by parser) ──────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypedServiceFields {
    pub request_type: String,
    pub response_type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypedActionFields {
    pub request_type: String,
    pub feedback_type: String,
    pub result_type: String,
}

// ── CommBus trait (transport abstraction) ────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PublishedCommMessage {
    pub topic_path: String,
    pub message_type: String,
    pub value: RuntimeValue,
    pub transport: TransportKind,
}

#[derive(Debug, Clone)]
pub struct SimNetworkConfig {
    pub delay_ms: f64,
    pub packet_loss: f64,
}

impl Default for SimNetworkConfig {
    fn default() -> Self {
        Self {
            delay_ms: 0.0,
            packet_loss: 0.0,
        }
    }
}

pub trait CommBus {
    fn publish(
        &mut self,
        topic_path: &str,
        message_type: &str,
        value: RuntimeValue,
        transport: TransportKind,
    );
    fn subscribe(&mut self, topic_path: &str, handler: &str);
    fn receive(&mut self, topic_path: &str) -> Option<RuntimeValue>;
    fn call_service(
        &mut self,
        service_name: &str,
        service_type: &str,
        request: Option<RuntimeValue>,
    ) -> RuntimeValue;
    fn send_action(
        &mut self,
        action_name: &str,
        action_type: &str,
        goal: RuntimeValue,
    ) -> RuntimeValue;
    fn discover(&self, target: DiscoverTarget, filter: &DiscoverFilter) -> Vec<String>;
    fn published_messages(&self) -> Vec<PublishedCommMessage>;
    fn inject_fault(&mut self, fault: &str);
    fn set_network_config(&mut self, config: SimNetworkConfig);
}

/// In-memory pub/sub bus for simulation and tests.
#[derive(Debug, Clone, Default)]
pub struct InMemoryCommBus {
    subscriptions: HashMap<String, Vec<String>>,
    buffers: HashMap<String, VecDeque<RuntimeValue>>,
    published: Vec<PublishedCommMessage>,
    discovered_robots: Vec<String>,
    discovered_agents: Vec<String>,
    discovered_devices: Vec<String>,
    network: SimNetworkConfig,
    faults: Vec<String>,
}

impl InMemoryCommBus {
    pub fn new() -> Self {
        Self {
            discovered_robots: vec!["RoverA".into(), "RoverB".into()],
            discovered_agents: vec!["Vision".into(), "Planner".into(), "Navigator".into()],
            discovered_devices: vec!["Camera".into(), "IMU".into(), "Lidar".into()],
            ..Default::default()
        }
    }

    pub fn register_robot(&mut self, name: impl Into<String>) {
        self.discovered_robots.push(name.into());
    }

    pub fn register_agent(&mut self, name: impl Into<String>) {
        self.discovered_agents.push(name.into());
    }

    pub fn register_device(&mut self, name: impl Into<String>) {
        self.discovered_devices.push(name.into());
    }
}

impl CommBus for InMemoryCommBus {
    fn publish(
        &mut self,
        topic_path: &str,
        message_type: &str,
        value: RuntimeValue,
        transport: TransportKind,
    ) {
        if self.faults.iter().any(|f| f == "NetworkOutage") {
            return;
        }
        if self.network.packet_loss > 0.0 {
            let hash = topic_path.len() + message_type.len();
            if (hash as f64 * 0.13).fract() < self.network.packet_loss {
                return;
            }
        }
        self.published.push(PublishedCommMessage {
            topic_path: topic_path.to_string(),
            message_type: message_type.to_string(),
            value: value.clone(),
            transport,
        });
        if let Some(buf) = self.buffers.get_mut(topic_path) {
            buf.push_back(value);
        }
    }

    fn subscribe(&mut self, topic_path: &str, handler: &str) {
        self.subscriptions
            .entry(topic_path.to_string())
            .or_default()
            .push(handler.to_string());
        self.buffers.entry(topic_path.to_string()).or_default();
    }

    fn receive(&mut self, topic_path: &str) -> Option<RuntimeValue> {
        self.buffers.get_mut(topic_path).and_then(|q| q.pop_front())
    }

    fn call_service(
        &mut self,
        _service_name: &str,
        service_type: &str,
        _request: Option<RuntimeValue>,
    ) -> RuntimeValue {
        RuntimeValue::Object {
            type_name: service_type.to_string(),
            fields: HashMap::from([("ok".into(), RuntimeValue::Bool { value: true })]),
        }
    }

    fn send_action(
        &mut self,
        _action_name: &str,
        action_type: &str,
        _goal: RuntimeValue,
    ) -> RuntimeValue {
        RuntimeValue::Object {
            type_name: action_type.to_string(),
            fields: HashMap::from([("success".into(), RuntimeValue::Bool { value: true })]),
        }
    }

    fn discover(&self, target: DiscoverTarget, filter: &DiscoverFilter) -> Vec<String> {
        let base = match target {
            DiscoverTarget::Robots => self.discovered_robots.clone(),
            DiscoverTarget::Agents => self.discovered_agents.clone(),
            DiscoverTarget::Devices => self.discovered_devices.clone(),
        };
        if let Some(cap) = &filter.capability {
            base.into_iter()
                .filter(|n| n.to_lowercase().contains(&cap.to_lowercase()))
                .collect()
        } else {
            base
        }
    }

    fn published_messages(&self) -> Vec<PublishedCommMessage> {
        self.published.clone()
    }

    fn inject_fault(&mut self, fault: &str) {
        self.faults.push(fault.to_string());
    }

    fn set_network_config(&mut self, config: SimNetworkConfig) {
        self.network = config;
    }
}

// ── Safety communication wrappers ────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum CommSafetyStage {
    ActionProposal,
    SafeAction,
    CommandMessage,
    Actuator,
}

pub fn validate_comm_safety_chain(
    stage: CommSafetyStage,
    value: &RuntimeValue,
) -> Result<(), String> {
    match stage {
        CommSafetyStage::ActionProposal => {
            if !matches!(value, RuntimeValue::Object { type_name, .. } if type_name == "ActionProposal")
            {
                return Err("Expected ActionProposal before safety validation".into());
            }
        }
        CommSafetyStage::SafeAction => {
            if !matches!(value, RuntimeValue::Object { type_name, .. } if type_name == "SafeAction")
            {
                return Err("Expected SafeAction before command conversion".into());
            }
        }
        CommSafetyStage::CommandMessage => {
            if !matches!(value, RuntimeValue::Object { type_name, .. } if type_name == "CommandMessage")
            {
                return Err("Expected CommandMessage before actuator dispatch".into());
            }
        }
        CommSafetyStage::Actuator => {}
    }
    Ok(())
}

// ── Network bandwidth estimation from QoS ────────────────────────────────────

pub fn estimate_topic_bandwidth_mbps(rate_hz: f64, message_size_bytes: f64) -> f64 {
    (rate_hz * message_size_bytes * 8.0) / 1_000_000.0
}

pub fn default_message_size(message_type: &str) -> f64 {
    match message_type {
        "Scan" | "LidarScan" | "LidarReading" => 64_000.0,
        "Pose" | "Velocity" => 128.0,
        "PathPlan" | "NavigationFeedback" => 4_096.0,
        _ => 512.0,
    }
}

pub fn qos_to_spanda_type(qos: &QosDecl) -> SpandaType {
    let _ = qos;
    SpandaType::Named { name: "QoS".into() }
}

/// Human-interaction message types (HRI).
pub const HRI_TYPES: &[&str] = &["Command", "Conversation", "Feedback", "Approval", "Intent"];

/// Safety wrapper message types.
pub const SAFETY_MESSAGE_TYPES: &[&str] = &[
    "SafeMessage",
    "VerifiedMessage",
    "TrustedSource",
    "ActionProposal",
    "SafeAction",
    "CommandMessage",
];

/// Communication capability actions for agent ACL.
pub const COMM_CAPABILITIES: &[&str] = &["subscribe", "publish", "call", "execute", "discover"];

pub fn is_comm_capability(action: &str) -> bool {
    COMM_CAPABILITIES.contains(&action)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_registry_builtin_and_custom() {
        let mut reg = MessageRegistry::new();
        assert!(reg.is_known("Velocity"));
        let decl = MessageDecl::MessageDecl {
            name: "LidarReading".into(),
            fields: vec![FieldDecl {
                name: "scan".into(),
                type_name: "LidarScan".into(),
                span: Span {
                    start: crate::ast::SourceLocation {
                        line: 1,
                        column: 1,
                        offset: 0,
                    },
                    end: crate::ast::SourceLocation {
                        line: 1,
                        column: 1,
                        offset: 0,
                    },
                },
            }],
            version: Some(1),
            span: Span {
                start: crate::ast::SourceLocation {
                    line: 1,
                    column: 1,
                    offset: 0,
                },
                end: crate::ast::SourceLocation {
                    line: 1,
                    column: 1,
                    offset: 0,
                },
            },
        };
        reg.register(&decl);
        assert!(reg.is_known("LidarReading"));
    }

    #[test]
    fn in_memory_bus_pub_sub() {
        let mut bus = InMemoryCommBus::new();
        bus.subscribe("/scan", "handler");
        bus.publish(
            "/scan",
            "Scan",
            RuntimeValue::Scan {
                nearest_distance: 1.5,
            },
            TransportKind::Sim,
        );
        let msg = bus.receive("/scan");
        assert!(msg.is_some());
        assert_eq!(bus.published_messages().len(), 1);
    }

    #[test]
    fn discover_robots_with_capability() {
        let bus = InMemoryCommBus::new();
        let results = bus.discover(
            DiscoverTarget::Robots,
            &DiscoverFilter {
                capability: Some("Rover".into()),
            },
        );
        assert!(!results.is_empty());
    }

    #[test]
    fn bandwidth_estimate() {
        let mbps = estimate_topic_bandwidth_mbps(20.0, 64000.0);
        assert!(mbps > 10.0);
    }
}
