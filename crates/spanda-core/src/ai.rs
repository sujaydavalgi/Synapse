use crate::ast::{AgentDecl, AiModelDecl, ConfigValue, MemoryKind, Stmt, UnitKind};
use crate::runtime::RuntimeValue;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiRuntimeKind {
    Onnx,
    Tflite,
    Tensorrt,
    OpenVino,
}

impl AiRuntimeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            AiRuntimeKind::Onnx => "onnx",
            AiRuntimeKind::Tflite => "tflite",
            AiRuntimeKind::Tensorrt => "tensorrt",
            AiRuntimeKind::OpenVino => "openvino",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AiLibModule {
    pub id: String,
    pub vendor: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub runtime: AiRuntimeKind,
}

fn build_ai_registry() -> HashMap<String, AiLibModule> {
    HashMap::from([
        (
            "onnx.runtime".to_string(),
            AiLibModule {
                id: "onnx.runtime".to_string(),
                vendor: "ONNX".to_string(),
                name: "runtime".to_string(),
                version: "1.0.0".to_string(),
                description: "ONNX Runtime inference backend".to_string(),
                runtime: AiRuntimeKind::Onnx,
            },
        ),
        (
            "tflite.runtime".to_string(),
            AiLibModule {
                id: "tflite.runtime".to_string(),
                vendor: "TensorFlow".to_string(),
                name: "runtime".to_string(),
                version: "1.0.0".to_string(),
                description: "TensorFlow Lite inference backend".to_string(),
                runtime: AiRuntimeKind::Tflite,
            },
        ),
        (
            "tensorrt.runtime".to_string(),
            AiLibModule {
                id: "tensorrt.runtime".to_string(),
                vendor: "NVIDIA".to_string(),
                name: "runtime".to_string(),
                version: "1.0.0".to_string(),
                description: "TensorRT inference backend for Jetson".to_string(),
                runtime: AiRuntimeKind::Tensorrt,
            },
        ),
        (
            "openvino.runtime".to_string(),
            AiLibModule {
                id: "openvino.runtime".to_string(),
                vendor: "Intel".to_string(),
                name: "runtime".to_string(),
                version: "1.0.0".to_string(),
                description: "OpenVINO inference backend for Intel CPUs and VPUs".to_string(),
                runtime: AiRuntimeKind::OpenVino,
            },
        ),
    ])
}

static AI_REGISTRY: std::sync::OnceLock<HashMap<String, AiLibModule>> = std::sync::OnceLock::new();

pub fn ai_lib_registry() -> &'static HashMap<String, AiLibModule> {
    AI_REGISTRY.get_or_init(build_ai_registry)
}

pub fn resolve_ai_import(path: &str) -> Option<&'static AiLibModule> {
    ai_lib_registry().get(path)
}

pub fn ai_lib_registry_export() -> &'static HashMap<String, AiLibModule> {
    ai_lib_registry()
}

pub fn list_ai_libraries() -> Vec<&'static AiLibModule> {
    ai_lib_registry().values().collect()
}

#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub prompt: String,
    pub input: Option<RuntimeValue>,
    pub model: String,
    pub provider: String,
    pub temperature: f64,
    pub max_tokens: usize,
}

#[derive(Debug, Clone)]
pub struct DetectionRequest {
    pub model: String,
    pub provider: String,
    pub frame: RuntimeValue,
}

#[derive(Debug, Clone)]
pub struct EmbedRequest {
    pub model: String,
    pub provider: String,
    pub text: String,
}

pub trait AiProvider: Send + Sync {
    fn complete(&self, request: &CompletionRequest) -> RuntimeValue;
    fn detect(&self, request: &DetectionRequest) -> RuntimeValue;
    fn embed(&self, request: &EmbedRequest) -> RuntimeValue;
}

fn scan_distance(input: Option<&RuntimeValue>) -> f64 {
    match input {
        Some(RuntimeValue::Scan { nearest_distance }) => *nearest_distance,
        Some(RuntimeValue::Object { type_name, fields }) if type_name == "Detection" => {
            if let Some(RuntimeValue::Number { value, .. }) = fields.get("nearest_distance") {
                *value
            } else {
                5.0
            }
        }
        _ => 5.0,
    }
}

fn action_proposal(
    linear: f64,
    angular: f64,
    source: impl Into<String>,
    trace: Vec<String>,
) -> RuntimeValue {
    RuntimeValue::ActionProposal {
        linear,
        angular,
        source: source.into(),
        trace,
    }
}

pub struct MockAiProvider;

impl AiProvider for MockAiProvider {
    fn complete(&self, request: &CompletionRequest) -> RuntimeValue {
        let prompt = request.prompt.clone();
        let dist = scan_distance(request.input.as_ref());

        if regex_stop_halt_wait(&request.prompt) {
            return action_proposal(
                0.0,
                0.0,
                &request.model,
                vec![
                    format!("model={}", request.model),
                    format!("prompt={prompt}"),
                    "decision=stop".into(),
                ],
            );
        }

        if regex_turn_avoid_obstacle(&request.prompt) || dist < 0.8 {
            let angular = if dist < 0.4 { 0.6 } else { 0.25 };
            let linear = if dist < 0.4 {
                0.0
            } else {
                (0.4_f64).min(dist * 0.3)
            };
            return action_proposal(
                linear,
                angular,
                &request.model,
                vec![
                    format!("model={}", request.model),
                    format!("prompt={prompt}"),
                    format!("nearest_distance={dist:.2}"),
                    "decision=avoid_obstacle".into(),
                ],
            );
        }

        let linear = (0.8_f64).min(dist * 0.45);
        action_proposal(
            linear,
            0.0,
            &request.model,
            vec![
                format!("model={}", request.model),
                format!("prompt={prompt}"),
                format!("nearest_distance={dist:.2}"),
                "decision=forward".into(),
            ],
        )
    }

    fn detect(&self, request: &DetectionRequest) -> RuntimeValue {
        let dist = scan_distance(Some(&request.frame));
        let (label, confidence) = if dist < 0.6 {
            ("obstacle", 0.94)
        } else if dist < 1.2 {
            ("object", 0.82)
        } else {
            ("clear", 0.71)
        };

        let mut fields = HashMap::new();
        fields.insert("label".to_string(), RuntimeValue::string(label));
        fields.insert(
            "confidence".to_string(),
            RuntimeValue::number(confidence, UnitKind::None),
        );
        fields.insert(
            "nearest_distance".to_string(),
            RuntimeValue::number(dist, UnitKind::M),
        );
        RuntimeValue::object("Detection", fields)
    }

    fn embed(&self, request: &EmbedRequest) -> RuntimeValue {
        let vector: Vec<f64> = (0..8)
            .map(|i| (request.text.len() as f64 * 0.13 + i as f64).sin() * 0.5 + 0.5)
            .collect();
        RuntimeValue::Embedding {
            dimensions: vector.len(),
            vector,
        }
    }
}

fn regex_stop_halt_wait(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    lower.contains("stop") || lower.contains("halt") || lower.contains("wait")
}

fn regex_turn_avoid_obstacle(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    lower.contains("turn") || lower.contains("avoid") || lower.contains("obstacle")
}

pub fn mock_summarize(input: Option<&RuntimeValue>, model: &str) -> RuntimeValue {
    let summary = match input {
        Some(RuntimeValue::Scan { nearest_distance }) => {
            format!("Nearest obstacle at {nearest_distance:.2} m")
        }
        Some(RuntimeValue::Object { type_name, fields }) if type_name == "Detection" => {
            let label = match fields.get("label") {
                Some(RuntimeValue::String { value }) => value.as_str(),
                _ => "object",
            };
            format!("Detected {label}")
        }
        _ => "Environment stable".to_string(),
    };
    RuntimeValue::Completion {
        text: format!("[{model}] {summary}"),
        model: Some(model.to_string()),
    }
}

pub fn mock_analyze_frame(frame: Option<&RuntimeValue>, _model: &str) -> RuntimeValue {
    let dist = scan_distance(frame);
    let mut fields = HashMap::new();
    fields.insert(
        "label".to_string(),
        RuntimeValue::string(if dist < 0.7 {
            "cluttered_scene"
        } else {
            "open_scene"
        }),
    );
    fields.insert(
        "confidence".to_string(),
        RuntimeValue::number(0.86, UnitKind::None),
    );
    fields.insert(
        "nearest_distance".to_string(),
        RuntimeValue::number(dist, UnitKind::M),
    );
    RuntimeValue::object("Detection", fields)
}

pub fn mock_camera_frame() -> RuntimeValue {
    let mut fields = HashMap::new();
    fields.insert(
        "width".to_string(),
        RuntimeValue::number(640.0, UnitKind::None),
    );
    fields.insert(
        "height".to_string(),
        RuntimeValue::number(480.0, UnitKind::None),
    );
    fields.insert(
        "nearest_distance".to_string(),
        RuntimeValue::number(1.5, UnitKind::M),
    );
    RuntimeValue::object("CameraFrame", fields)
}

pub fn build_prompt(base: &str, input: Option<&RuntimeValue>, goal: Option<&str>) -> String {
    let mut header = String::new();
    if let Some(g) = goal.filter(|s| !s.is_empty()) {
        header.push_str(&format!("Goal: {g}"));
    }
    let base = base.trim();
    if !base.is_empty() {
        if !header.is_empty() {
            header.push_str("\n\n");
        }
        header.push_str(base);
    }
    let input_summary = summarize_input(input);
    if header.is_empty() {
        format!("Context:\n{input_summary}")
    } else {
        format!("{header}\n\nContext:\n{input_summary}")
    }
}

fn summarize_input(input: Option<&RuntimeValue>) -> String {
    match input {
        None | Some(RuntimeValue::Void) => "(no input)".to_string(),
        Some(RuntimeValue::Scan { nearest_distance }) => {
            format!("LiDAR scan — nearest obstacle {nearest_distance:.2} m")
        }
        Some(RuntimeValue::String { value }) => value.clone(),
        Some(RuntimeValue::Object { type_name, fields }) if type_name == "Detection" => {
            let label = match fields.get("label") {
                Some(RuntimeValue::String { value }) => value.as_str(),
                _ => "object",
            };
            let conf = match fields.get("confidence") {
                Some(RuntimeValue::Number { value, .. }) => *value,
                _ => 0.0,
            };
            format!("Vision scene — {label} ({conf:.2} confidence)")
        }
        Some(RuntimeValue::Object { type_name, fields }) if type_name == "Detections" => {
            let count = match fields.get("count") {
                Some(RuntimeValue::Number { value, .. }) => *value,
                _ => 0.0,
            };
            format!("Detections — {count} object(s) in view")
        }
        Some(RuntimeValue::Object { type_name, .. }) => format!("{type_name} object"),
        Some(RuntimeValue::Completion { text, .. }) => text.clone(),
        Some(RuntimeValue::Goal { text }) => format!("Goal — {text}"),
        Some(other) => format!("({} value)", runtime_value_kind(other)),
    }
}

fn runtime_value_kind(value: &RuntimeValue) -> &'static str {
    match value {
        RuntimeValue::Number { .. } => "number",
        RuntimeValue::Bool { .. } => "bool",
        RuntimeValue::String { .. } => "string",
        RuntimeValue::Void => "void",
        RuntimeValue::Scan { .. } => "scan",
        RuntimeValue::Pose { .. } => "pose",
        RuntimeValue::Velocity { .. } => "velocity",
        RuntimeValue::Trajectory { .. } => "trajectory",
        RuntimeValue::Transform { .. } => "transform",
        RuntimeValue::Object { .. } => "object",
        RuntimeValue::Enum { .. } => "enum",
        RuntimeValue::Sensor { .. } => "sensor",
        RuntimeValue::Actuator { .. } => "actuator",
        RuntimeValue::Topic { .. } => "topic",
        RuntimeValue::Service { .. } => "service",
        RuntimeValue::Action { .. } => "action",
        RuntimeValue::Robot => "robot",
        RuntimeValue::Agent { .. } => "agent",
        RuntimeValue::Twin { .. } => "twin",
        RuntimeValue::SafetyCtx => "safety_ctx",
        RuntimeValue::AiModel { .. } => "ai_model",
        RuntimeValue::ActionProposal { .. } => "action_proposal",
        RuntimeValue::SafeAction { .. } => "safe_action",
        RuntimeValue::Completion { .. } => "completion",
        RuntimeValue::Embedding { .. } => "embedding",
        RuntimeValue::Goal { .. } => "goal",
        RuntimeValue::SensorFusion { .. } => "sensor_fusion",
    }
}

#[derive(Debug, Clone)]
pub struct AiModelConfig {
    pub provider: String,
    pub model: String,
    pub temperature: f64,
    pub max_tokens: usize,
}

pub struct AiModel {
    pub name: String,
    pub model_type: String,
    pub config: AiModelConfig,
    provider: Box<dyn AiProvider>,
}

impl AiModel {
    pub fn new(decl: &AiModelDecl, provider: Option<Box<dyn AiProvider>>) -> Self {
        Self {
            name: match decl {
                AiModelDecl::AiModelDecl { name, .. } => name.clone(),
            },
            model_type: match decl {
                AiModelDecl::AiModelDecl { model_type, .. } => model_type.clone(),
            },
            config: parse_config(decl),
            provider: provider.unwrap_or_else(|| Box::new(MockAiProvider)),
        }
    }

    pub fn reason(
        &self,
        prompt: &str,
        input: Option<RuntimeValue>,
        goal: Option<&str>,
    ) -> Result<RuntimeValue, String> {
        if self.model_type != "LLM" {
            return Err(format!(
                "Model '{}' is {}, not LLM",
                self.name, self.model_type
            ));
        }
        Ok(self.provider.complete(&CompletionRequest {
            prompt: build_prompt(prompt, input.as_ref(), goal),
            input,
            model: self.config.model.clone(),
            provider: self.config.provider.clone(),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
        }))
    }

    pub fn summarize(&self, input: Option<RuntimeValue>) -> Result<RuntimeValue, String> {
        if self.model_type != "LLM" {
            return Err(format!(
                "Model '{}' is {}, not LLM",
                self.name, self.model_type
            ));
        }
        Ok(mock_summarize(input.as_ref(), &self.config.model))
    }

    pub fn detect(&self, frame: RuntimeValue) -> Result<RuntimeValue, String> {
        if self.model_type != "VisionModel" {
            return Err(format!(
                "Model '{}' is {}, not VisionModel",
                self.name, self.model_type
            ));
        }
        Ok(self.provider.detect(&DetectionRequest {
            model: self.config.model.clone(),
            provider: self.config.provider.clone(),
            frame,
        }))
    }

    pub fn to_runtime_value(&self) -> RuntimeValue {
        RuntimeValue::AiModel {
            name: self.name.clone(),
            model_type: self.model_type.clone(),
            provider: self.config.provider.clone(),
        }
    }
}

fn parse_config(decl: &AiModelDecl) -> AiModelConfig {
    let AiModelDecl::AiModelDecl { config, name, .. } = decl;
    let map: HashMap<String, ConfigValue> = config
        .iter()
        .map(|e| (e.key.clone(), e.value.clone()))
        .collect();

    AiModelConfig {
        provider: config_string(&map, "provider", "mock"),
        model: config_string(&map, "model", name),
        temperature: config_number(&map, "temperature", 0.2),
        max_tokens: config_number(&map, "max_tokens", 512.0) as usize,
    }
}

fn config_string(map: &HashMap<String, ConfigValue>, key: &str, default: &str) -> String {
    match map.get(key) {
        Some(ConfigValue::String(s)) => s.clone(),
        Some(ConfigValue::Number(n)) => n.to_string(),
        Some(ConfigValue::Bool(b)) => b.to_string(),
        None => default.to_string(),
    }
}

fn config_number(map: &HashMap<String, ConfigValue>, key: &str, default: f64) -> f64 {
    match map.get(key) {
        Some(ConfigValue::Number(n)) => *n,
        Some(ConfigValue::String(s)) => s.parse().unwrap_or(default),
        _ => default,
    }
}

pub fn create_ai_model(decl: &AiModelDecl) -> AiModel {
    AiModel::new(decl, None)
}

#[derive(Clone)]
pub struct AgentRuntime {
    pub decl: AgentDecl,
    pub memory: Option<MemoryStore>,
}

pub fn create_agent_runtime(decl: AgentDecl, memory: Option<MemoryStore>) -> AgentRuntime {
    AgentRuntime { decl, memory }
}

pub fn agent_tool_names(decl: &AgentDecl) -> Vec<String> {
    match decl {
        AgentDecl::AgentDecl { tools, .. } => tools.clone(),
    }
}

pub fn agent_uses_models(decl: &AgentDecl) -> Vec<String> {
    match decl {
        AgentDecl::AgentDecl { uses_ai, .. } => uses_ai.clone(),
    }
}

pub trait PlanExecutor {
    fn execute_block(&mut self, stmts: &[Stmt]) -> Result<(), crate::error::SpandaError>;
}

pub fn execute_agent_plan(
    agent: &AgentRuntime,
    executor: &mut dyn PlanExecutor,
) -> Result<(), crate::error::SpandaError> {
    let plan_body = match &agent.decl {
        AgentDecl::AgentDecl { plan_body, .. } => plan_body,
    };
    executor.execute_block(plan_body)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiMemoryKind {
    ShortTerm,
    LongTerm,
}

impl From<MemoryKind> for AiMemoryKind {
    fn from(kind: MemoryKind) -> Self {
        match kind {
            MemoryKind::ShortTerm => AiMemoryKind::ShortTerm,
            MemoryKind::LongTerm => AiMemoryKind::LongTerm,
        }
    }
}

#[derive(Clone)]
pub struct MemoryStore {
    pub kind: AiMemoryKind,
    entries: Vec<MemoryEntry>,
    limit: usize,
}

#[derive(Clone)]
struct MemoryEntry {
    key: String,
    value: RuntimeValue,
    #[allow(dead_code)]
    at: u128,
}

impl MemoryStore {
    pub fn new(kind: AiMemoryKind, limit: Option<usize>) -> Self {
        let default_limit = match kind {
            AiMemoryKind::ShortTerm => 32,
            AiMemoryKind::LongTerm => 256,
        };
        Self {
            kind,
            entries: Vec::new(),
            limit: limit.unwrap_or(default_limit),
        }
    }

    pub fn remember(&mut self, key: impl Into<String>, value: RuntimeValue) {
        self.entries.push(MemoryEntry {
            key: key.into(),
            value,
            at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0),
        });
        if self.entries.len() > self.limit {
            self.entries.remove(0);
        }
    }

    pub fn recall(&self, key: &str) -> Option<&RuntimeValue> {
        self.entries
            .iter()
            .rev()
            .find(|e| e.key == key)
            .map(|e| &e.value)
    }

    pub fn recent(&self, count: usize) -> Vec<&RuntimeValue> {
        self.entries
            .iter()
            .rev()
            .take(count)
            .map(|e| &e.value)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn summary_for_prompt(&self) -> Option<String> {
        if self.entries.is_empty() {
            return None;
        }
        let kind = match self.kind {
            AiMemoryKind::ShortTerm => "short_term",
            AiMemoryKind::LongTerm => "long_term",
        };
        let keys: Vec<&str> = self
            .entries
            .iter()
            .rev()
            .take(5)
            .map(|e| e.key.as_str())
            .collect();
        Some(format!("Agent memory ({kind}): {}", keys.join(", ")))
    }
}

pub fn runtime_safe_action(linear: f64, angular: f64) -> RuntimeValue {
    RuntimeValue::SafeAction { linear, angular }
}

pub fn runtime_action_proposal(
    linear: f64,
    angular: f64,
    source: impl Into<String>,
) -> RuntimeValue {
    RuntimeValue::ActionProposal {
        linear,
        angular,
        source: source.into(),
        trace: Vec::new(),
    }
}

pub fn is_action_proposal(value: &RuntimeValue) -> bool {
    matches!(value, RuntimeValue::ActionProposal { .. })
}

pub fn is_safe_action(value: &RuntimeValue) -> bool {
    matches!(value, RuntimeValue::SafeAction { .. })
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionProposalFields {
    pub linear: f64,
    pub angular: f64,
    pub source: String,
}

pub fn proposal_from_value(value: &RuntimeValue) -> Option<ActionProposalFields> {
    match value {
        RuntimeValue::ActionProposal {
            linear,
            angular,
            source,
            trace: _,
        } => Some(ActionProposalFields {
            linear: *linear,
            angular: *angular,
            source: source.clone(),
        }),
        RuntimeValue::Object { type_name, fields } if type_name == "ActionProposal" => {
            let linear = match fields.get("linear") {
                Some(RuntimeValue::Number { value, .. }) => *value,
                _ => 0.0,
            };
            let angular = match fields.get("angular") {
                Some(RuntimeValue::Number { value, .. }) => *value,
                _ => 0.0,
            };
            Some(ActionProposalFields {
                linear,
                angular,
                source: "object".to_string(),
            })
        }
        RuntimeValue::Velocity { linear, angular } => Some(ActionProposalFields {
            linear: *linear,
            angular: *angular,
            source: "velocity".to_string(),
        }),
        _ => None,
    }
}

pub fn safe_action_from_proposal(linear: f64, angular: f64) -> RuntimeValue {
    runtime_safe_action(linear, angular)
}

pub fn wrap_completion(text: impl Into<String>, model: impl Into<String>) -> RuntimeValue {
    RuntimeValue::Completion {
        text: text.into(),
        model: Some(model.into()),
    }
}

pub fn wrap_detection(label: &str, confidence: f64, nearest_distance: f64) -> RuntimeValue {
    let mut fields = HashMap::new();
    fields.insert("label".to_string(), RuntimeValue::string(label));
    fields.insert(
        "confidence".to_string(),
        RuntimeValue::number(confidence, UnitKind::None),
    );
    fields.insert(
        "nearest_distance".to_string(),
        RuntimeValue::number(nearest_distance, UnitKind::M),
    );
    RuntimeValue::object("Detection", fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_provider_proposes_motion() {
        let provider = MockAiProvider;
        let result = provider.complete(&CompletionRequest {
            prompt: "Go forward".to_string(),
            input: Some(RuntimeValue::scan(2.0)),
            model: "mock".to_string(),
            provider: "mock".to_string(),
            temperature: 0.2,
            max_tokens: 512,
        });
        assert!(is_action_proposal(&result));
    }

    #[test]
    fn mock_provider_stops_on_halt_prompt() {
        let provider = MockAiProvider;
        let result = provider.complete(&CompletionRequest {
            prompt: "stop now".to_string(),
            input: None,
            model: "mock".to_string(),
            provider: "mock".to_string(),
            temperature: 0.2,
            max_tokens: 512,
        });
        if let RuntimeValue::ActionProposal {
            linear, angular, ..
        } = result
        {
            assert_eq!(linear, 0.0);
            assert_eq!(angular, 0.0);
        } else {
            panic!("expected action proposal");
        }
    }

    #[test]
    fn mock_summarize_scan() {
        let summary = mock_summarize(Some(&RuntimeValue::scan(1.25)), "mock");
        if let RuntimeValue::Completion { text, .. } = summary {
            assert!(text.contains("1.25"));
        } else {
            panic!("expected completion");
        }
    }

    #[test]
    fn memory_store_recalls_latest() {
        let mut store = MemoryStore::new(AiMemoryKind::ShortTerm, None);
        store.remember("a", RuntimeValue::number(1.0, UnitKind::None));
        store.remember("a", RuntimeValue::number(2.0, UnitKind::None));
        assert_eq!(
            store.recall("a"),
            Some(&RuntimeValue::number(2.0, UnitKind::None))
        );
    }

    #[test]
    fn resolves_ai_imports() {
        assert!(resolve_ai_import("onnx.runtime").is_some());
        assert!(resolve_ai_import("openvino.runtime").is_some());
        assert!(list_ai_libraries().len() >= 4);
    }

    #[test]
    fn proposal_from_velocity() {
        let proposal = proposal_from_value(&RuntimeValue::Velocity {
            linear: 0.5,
            angular: 0.1,
        });
        assert_eq!(proposal.unwrap().source, "velocity");
    }
}
