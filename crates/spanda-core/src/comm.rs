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
        // Construct from ident.
        //
        // Parameters:
        // - `s` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::comm::from_ident(s);

        // Match on s and handle each case.
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
        // Create a new instance.
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
        // let value = spanda_core::comm::new();

        // Create mutable reg for accumulating results.
        let mut reg = Self::default();

        // Iterate over each name in ["Velocity", "Pose", "Scan", "String"].
        for name in ["Velocity", "Pose", "Scan", "String"] {
            reg.builtin.insert(name.into());
        }
        reg
    }

    pub fn register(&mut self, decl: &MessageDecl) {
        // Register the value.
        //
        // Parameters:
        // - `self` — method receiver
        // - `decl` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.register(decl);

        // Compute MessageDecl for the following logic.
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
        // Construct from program.
        //
        // Parameters:
        // - `messages` — input value
        // - `structs` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::comm::from_program(messages, structs);

        // Create mutable reg for accumulating results.
        let mut reg = Self::new();

        // Process each message.
        for msg in messages {
            reg.register(msg);
        }

        // Process each struct.
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
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.is_known(name);

        // Call contains on the current instance.
        self.builtin.contains(name) || self.schemas.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&MessageSchema> {
        // Get.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.get(name);

        // Call get on the current instance.
        self.schemas.get(name)
    }

    pub fn resolve_type(&self, name: &str) -> Option<SpandaType> {
        // Resolve type.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.resolve_type(name);

        // Match on name and handle each case.
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
        // let value = spanda_core::comm::default();

        // Assemble the struct fields and return it.
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
    fn active_faults(&self) -> Vec<String>;
    fn subscription_paths(&self) -> Vec<String>;
    fn push_inbound(&mut self, topic_path: &str, value: RuntimeValue);
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
        // Create a new instance.
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
        // let value = spanda_core::comm::new();

        // Assemble the struct fields and return it.
        Self {
            discovered_robots: vec!["RoverA".into(), "RoverB".into()],
            discovered_agents: vec!["Vision".into(), "Planner".into(), "Navigator".into()],
            discovered_devices: vec!["Camera".into(), "IMU".into(), "Lidar".into()],
            ..Default::default()
        }
    }

    pub fn register_robot(&mut self, name: impl Into<String>) {
        // Register robot.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.register_robot(name);

        // Append into self.
        self.discovered_robots.push(name.into());
    }

    pub fn register_agent(&mut self, name: impl Into<String>) {
        // Register agent.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.register_agent(name);

        // Append into self.
        self.discovered_agents.push(name.into());
    }

    pub fn register_device(&mut self, name: impl Into<String>) {
        // Register device.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.register_device(name);

        // Append into self.
        self.discovered_devices.push(name.into());
    }

    pub fn active_faults(&self) -> Vec<String> {
        // Active faults.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<String>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.active_faults();

        // Call clone on the current instance.
        self.faults.clone()
    }

    pub fn subscription_paths(&self) -> Vec<String> {
        // Subscription paths.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<String>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.subscription_paths();

        // Collect filtered entries into a new list.
        self.subscriptions.keys().cloned().collect()
    }

    pub fn push_inbound(&mut self, topic_path: &str, value: RuntimeValue) {
        // Push inbound.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic_path` — input value
        // - `value` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.push_inbound(topic_path, value);

        // Call buffers on the current instance.
        self.buffers
            .entry(topic_path.to_string())
            .or_default()
            .push_back(value);
    }

    /// Deliver a message to a peer robot topic namespace (`/{peer}/{topic}`).
    pub fn publish_peer(
        &mut self,
        peer: &str,
        topic: &str,
        value: RuntimeValue,
        transport: TransportKind,
    ) {
        // Publish peer.
        //
        // Parameters:
        // - `self` — method receiver
        // - `peer` — input value
        // - `topic` — input value
        // - `value` — input value
        // - `transport` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.publish_peer(peer, topic, value, transport);

        // Resolve the filesystem path for the next step.
        let path = format!("/{peer}/{topic}");
        self.publish(&path, "PeerMessage", value, transport);
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
        // Publish.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic_path` — input value
        // - `message_type` — input value
        // - `value` — input value
        // - `transport` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.publish(topic_path, message_type, value, transport);

        // take the branch when any equals "NetworkOutage").
        if self.faults.iter().any(|f| f == "NetworkOutage") {
            return;
        }

        // Take this path when self.network.packet loss > 0.0.
        if self.network.packet_loss > 0.0 {
            let hash = topic_path.len() + message_type.len();

            // Take this path when (hash as f64 * 0.13).fract() < self.network.packet loss.
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

        // Emit output when get mut provides a buf.
        if let Some(buf) = self.buffers.get_mut(topic_path) {
            buf.push_back(value);
        }
    }

    fn subscribe(&mut self, topic_path: &str, handler: &str) {
        // Subscribe.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic_path` — input value
        // - `handler` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.subscribe(topic_path, handler);

        // Call subscriptions on the current instance.
        self.subscriptions
            .entry(topic_path.to_string())
            .or_default()
            .push(handler.to_string());
        self.buffers.entry(topic_path.to_string()).or_default();
    }

    fn receive(&mut self, topic_path: &str) -> Option<RuntimeValue> {
        // Receive.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic_path` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.receive(topic_path);

        // Transform self and continue the chain.
        self.buffers.get_mut(topic_path).and_then(|q| q.pop_front())
    }

    fn call_service(
        &mut self,
        _service_name: &str,
        service_type: &str,
        _request: Option<RuntimeValue>,
    ) -> RuntimeValue {
        // Call service.
        //
        // Parameters:
        // - `self` — method receiver
        // - `_service_name` — input value
        // - `service_type` — input value
        // - `_request` — input value
        //
        // Returns:
        // RuntimeValue.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.call_service(_service_name, service_type, _request);

        // Build a Object runtime value.
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
        // Send action.
        //
        // Parameters:
        // - `self` — method receiver
        // - `_action_name` — input value
        // - `action_type` — input value
        // - `_goal` — input value
        //
        // Returns:
        // RuntimeValue.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.send_action(_action_name, action_type, _goal);

        // Build a Object runtime value.
        RuntimeValue::Object {
            type_name: action_type.to_string(),
            fields: HashMap::from([("success".into(), RuntimeValue::Bool { value: true })]),
        }
    }

    fn discover(&self, target: DiscoverTarget, filter: &DiscoverFilter) -> Vec<String> {
        // Discover.
        //
        // Parameters:
        // - `self` — method receiver
        // - `target` — input value
        // - `filter` — input value
        //
        // Returns:
        // Vec<String>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.discover(target, filter);

        // Compute base for the following logic.
        let base = match target {
            DiscoverTarget::Robots => self.discovered_robots.clone(),
            DiscoverTarget::Agents => self.discovered_agents.clone(),
            DiscoverTarget::Devices => self.discovered_devices.clone(),
        };

        // Emit output when capability provides a cap.
        if let Some(cap) = &filter.capability {
            base.into_iter()
                .filter(|n| n.to_lowercase().contains(&cap.to_lowercase()))
                .collect()
        } else {
            base
        }
    }

    fn published_messages(&self) -> Vec<PublishedCommMessage> {
        // Published messages.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<PublishedCommMessage>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.published_messages();

        // Call clone on the current instance.
        self.published.clone()
    }

    fn inject_fault(&mut self, fault: &str) {
        // Inject fault.
        //
        // Parameters:
        // - `self` — method receiver
        // - `fault` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.inject_fault(fault);

        // Append into self.
        self.faults.push(fault.to_string());
    }

    fn set_network_config(&mut self, config: SimNetworkConfig) {
        // Set network config.
        //
        // Parameters:
        // - `self` — method receiver
        // - `config` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.set_network_config(config);

        // Call network = config; on the current instance.
        self.network = config;
    }

    fn active_faults(&self) -> Vec<String> {
        // Active faults.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<String>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.active_faults();

        // Call clone on the current instance.
        self.faults.clone()
    }

    fn subscription_paths(&self) -> Vec<String> {
        // Subscription paths.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<String>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.subscription_paths();

        // Collect filtered entries into a new list.
        self.subscriptions.keys().cloned().collect()
    }

    fn push_inbound(&mut self, topic_path: &str, value: RuntimeValue) {
        // Push inbound.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic_path` — input value
        // - `value` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.push_inbound(topic_path, value);

        // Call buffers on the current instance.
        self.buffers
            .entry(topic_path.to_string())
            .or_default()
            .push_back(value);
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
    // Validate comm safety chain.
    //
    // Parameters:
    // - `stage` — input value
    // - `value` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::comm::validate_comm_safety_chain(stage, value);

    // Match on stage and handle each case.
    match stage {
        CommSafetyStage::ActionProposal => {
            // Keep entries that match the expected pattern.
            if !matches!(value, RuntimeValue::Object { type_name, .. } if type_name == "ActionProposal")
            {
                return Err("Expected ActionProposal before safety validation".into());
            }
        }
        CommSafetyStage::SafeAction => {
            // Keep entries that match the expected pattern.
            if !matches!(value, RuntimeValue::Object { type_name, .. } if type_name == "SafeAction")
            {
                return Err("Expected SafeAction before command conversion".into());
            }
        }
        CommSafetyStage::CommandMessage => {
            // Keep entries that match the expected pattern.
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
    // Estimate topic bandwidth mbps.
    //
    // Parameters:
    // - `rate_hz` — input value
    // - `message_size_bytes` — input value
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::comm::estimate_topic_bandwidth_mbps(rate_hz, message_size_bytes);

    // Produce 0 as the result.
    (rate_hz * message_size_bytes * 8.0) / 1_000_000.0
}

pub fn default_message_size(message_type: &str) -> f64 {
    // Default message size.
    //
    // Parameters:
    // - `message_type` — input value
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::comm::default_message_size(message_type);

    // Match on message type and handle each case.
    match message_type {
        "Scan" | "LidarScan" | "LidarReading" => 64_000.0,
        "Pose" | "Velocity" => 128.0,
        "PathPlan" | "NavigationFeedback" => 4_096.0,
        _ => 512.0,
    }
}

pub fn qos_to_spanda_type(qos: &QosDecl) -> SpandaType {
    // Qos to spanda type.
    //
    // Parameters:
    // - `qos` — input value
    //
    // Returns:
    // SpandaType.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::comm::qos_to_spanda_type(qos);

    // Compute value for the following logic.
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
    //
    // Parameters:
    // - `action` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::comm::is_comm_capability(action);

    // Produce contains as the result.
    COMM_CAPABILITIES.contains(&action)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_registry_builtin_and_custom() {
        // Message registry builtin and custom.
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
        // let result = spanda_core::comm::message_registry_builtin_and_custom();

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
        // In memory bus pub sub.
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
        // let result = spanda_core::comm::in_memory_bus_pub_sub();

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
        // Discover robots with capability.
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
        // let result = spanda_core::comm::discover_robots_with_capability();

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
        // Bandwidth estimate.
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
        // let result = spanda_core::comm::bandwidth_estimate();

        let mbps = estimate_topic_bandwidth_mbps(20.0, 64000.0);
        assert!(mbps > 10.0);
    }
}
