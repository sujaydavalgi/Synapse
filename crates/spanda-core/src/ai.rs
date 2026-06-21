//! ai support for Spanda.
//!
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
    // Build ai registry.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // HashMap<String, AiLibModule>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::build_ai_registry();

    // Produce from as the result.
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
    // Ai lib registry.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // &'static HashMap<String, AiLibModule>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::ai_lib_registry();

    // Produce get or init as the result.
    AI_REGISTRY.get_or_init(build_ai_registry)
}

pub fn resolve_ai_import(path: &str) -> Option<&'static AiLibModule> {
    // Resolve ai import.
    //
    // Parameters:
    // - `path` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::resolve_ai_import(path);

    // Produce get as the result.
    ai_lib_registry().get(path)
}

pub fn ai_lib_registry_export() -> &'static HashMap<String, AiLibModule> {
    // Ai lib registry export.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // &'static HashMap<String, AiLibModule>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::ai_lib_registry_export();

    // Produce ai lib registry as the result.
    ai_lib_registry()
}

pub fn list_ai_libraries() -> Vec<&'static AiLibModule> {
    // List ai libraries.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Vec<&'static AiLibModule>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::list_ai_libraries();

    // Collect filtered entries into a new list.
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
    // Scan distance.
    //
    // Parameters:
    // - `input` — input value
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::scan_distance(input);

    // Match on input and handle each case.
    match input {
        Some(RuntimeValue::Scan { nearest_distance }) => *nearest_distance,
        Some(RuntimeValue::Object { type_name, fields }) if type_name == "Detection" => {
            // Take this path when let Some(RuntimeValue::Number { value, .. }) = fields.get("nearest dis.
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
    // Action proposal.
    //
    // Parameters:
    // - `linear` — input value
    // - `angular` — input value
    // - `source` — input value
    // - `trace` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::action_proposal(linear, angular, source, trace);

    // Build a ActionProposal runtime value.
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
        // Complete.
        //
        // Parameters:
        // - `self` — method receiver
        // - `request` — input value
        //
        // Returns:
        // RuntimeValue.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.complete(request);

        // Compute prompt for the following logic.
        let prompt = request.prompt.clone();
        let dist = scan_distance(request.input.as_ref());

        // Take this path when regex stop halt wait(&request.prompt).
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

        // Take this path when regex turn avoid obstacle(&request.prompt) || dist < 0.8.
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
        // Detect.
        //
        // Parameters:
        // - `self` — method receiver
        // - `request` — input value
        //
        // Returns:
        // RuntimeValue.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.detect(request);

        // Compute dist for the following logic.
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
        // Embed.
        //
        // Parameters:
        // - `self` — method receiver
        // - `request` — input value
        //
        // Returns:
        // RuntimeValue.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.embed(request);

        // Compute vector for the following logic.
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
    // Regex stop halt wait.
    //
    // Parameters:
    // - `prompt` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::regex_stop_halt_wait(prompt);

    // Compute lower for the following logic.
    let lower = prompt.to_lowercase();
    lower.contains("stop") || lower.contains("halt") || lower.contains("wait")
}

fn regex_turn_avoid_obstacle(prompt: &str) -> bool {
    // Regex turn avoid obstacle.
    //
    // Parameters:
    // - `prompt` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::regex_turn_avoid_obstacle(prompt);

    // Compute lower for the following logic.
    let lower = prompt.to_lowercase();
    lower.contains("turn") || lower.contains("avoid") || lower.contains("obstacle")
}

pub fn mock_summarize(input: Option<&RuntimeValue>, model: &str) -> RuntimeValue {
    // Mock summarize.
    //
    // Parameters:
    // - `input` — input value
    // - `model` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::mock_summarize(input, model);

    // Compute summary for the following logic.
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
    // Mock analyze frame.
    //
    // Parameters:
    // - `frame` — input value
    // - `_model` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::mock_analyze_frame(frame, _model);

    // Compute dist for the following logic.
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
    // Mock camera frame.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::mock_camera_frame();

    // Create mutable fields for accumulating results.
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
    // Build prompt.
    //
    // Parameters:
    // - `base` — input value
    // - `input` — input value
    // - `goal` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::build_prompt(base, input, goal);

    // Create mutable header for accumulating results.
    let mut header = String::new();

    // Emit output when is empty provides a g.
    if let Some(g) = goal.filter(|s| !s.is_empty()) {
        header.push_str(&format!("Goal: {g}"));
    }
    let base = base.trim();

    // Skip further work when !base is empty.
    if !base.is_empty() {
        // Skip further work when !header is empty.
        if !header.is_empty() {
            header.push_str("\n\n");
        }
        header.push_str(base);
    }
    let input_summary = summarize_input(input);

    // Skip further work when header is empty.
    if header.is_empty() {
        format!("Context:\n{input_summary}")
    } else {
        format!("{header}\n\nContext:\n{input_summary}")
    }
}

fn summarize_input(input: Option<&RuntimeValue>) -> String {
    // Summarize input.
    //
    // Parameters:
    // - `input` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::summarize_input(input);

    // Match on input and handle each case.
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
    // Runtime value kind.
    //
    // Parameters:
    // - `value` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::runtime_value_kind(value);

    // Match on value and handle each case.
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
        RuntimeValue::MissionControl { .. } => "mission_control",
        RuntimeValue::NavigationControl { .. } => "navigation_control",
        RuntimeValue::FleetControl { .. } => "fleet_control",
        RuntimeValue::AuditCtx => "audit_ctx",
        RuntimeValue::LedgerCtx => "ledger_ctx",
        RuntimeValue::Identity { .. } => "identity",
        RuntimeValue::Secret { .. } => "secret",
        RuntimeValue::Result { .. } => "result",
        RuntimeValue::Option { .. } => "option",
        RuntimeValue::Bytes { .. } => "bytes",
        RuntimeValue::Null => "null",
        RuntimeValue::Future { .. } => "future",
        RuntimeValue::TaskHandle { .. } => "task_handle",
        RuntimeValue::Channel { .. } => "channel",
        RuntimeValue::TraitObject { .. } => "trait_object",
        RuntimeValue::Regex { .. } => "regex",
        RuntimeValue::Capture { .. } => "capture",
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
        // Create a new instance.
        //
        // Parameters:
        // - `decl` — input value
        // - `provider` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_core::ai::new(decl, provider);

        // Assemble the struct fields and return it.
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
        // Reason.
        //
        // Parameters:
        // - `self` — method receiver
        // - `prompt` — input value
        // - `input` — input value
        // - `goal` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.reason(prompt, input, goal);

        // take the branch when model type differs from "LLM".
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
        // Summarize.
        //
        // Parameters:
        // - `self` — method receiver
        // - `input` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.summarize(input);

        // take the branch when model type differs from "LLM".
        if self.model_type != "LLM" {
            return Err(format!(
                "Model '{}' is {}, not LLM",
                self.name, self.model_type
            ));
        }
        Ok(mock_summarize(input.as_ref(), &self.config.model))
    }

    pub fn detect(&self, frame: RuntimeValue) -> Result<RuntimeValue, String> {
        // Detect.
        //
        // Parameters:
        // - `self` — method receiver
        // - `frame` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.detect(frame);

        // take the branch when model type differs from "VisionModel".
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
        // Convert to runtime value.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // RuntimeValue.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.to_runtime_value();

        // Build a AiModel runtime value.
        RuntimeValue::AiModel {
            name: self.name.clone(),
            model_type: self.model_type.clone(),
            provider: self.config.provider.clone(),
        }
    }
}

fn parse_config(decl: &AiModelDecl) -> AiModelConfig {
    // Parse config.
    //
    // Parameters:
    // - `decl` — input value
    //
    // Returns:
    // AiModelConfig.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::parse_config(decl);

    // Compute AiModelDecl for the following logic.
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
    // Config string.
    //
    // Parameters:
    // - `map` — input value
    // - `key` — input value
    // - `default` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::config_string(map, key, default);

    // Match on get and handle each case.
    match map.get(key) {
        Some(ConfigValue::String(s)) => s.clone(),
        Some(ConfigValue::Number(n)) => n.to_string(),
        Some(ConfigValue::Bool(b)) => b.to_string(),
        None => default.to_string(),
    }
}

fn config_number(map: &HashMap<String, ConfigValue>, key: &str, default: f64) -> f64 {
    // Config number.
    //
    // Parameters:
    // - `map` — input value
    // - `key` — input value
    // - `default` — input value
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::config_number(map, key, default);

    // Match on get and handle each case.
    match map.get(key) {
        Some(ConfigValue::Number(n)) => *n,
        Some(ConfigValue::String(s)) => s.parse().unwrap_or(default),
        _ => default,
    }
}

pub fn create_ai_model(decl: &AiModelDecl) -> AiModel {
    // Create ai model.
    //
    // Parameters:
    // - `decl` — input value
    //
    // Returns:
    // AiModel.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::create_ai_model(decl);

    // Produce new as the result.
    AiModel::new(decl, None)
}

#[derive(Clone)]
pub struct AgentRuntime {
    pub decl: AgentDecl,
    pub memory: Option<MemoryStore>,
}

pub fn create_agent_runtime(decl: AgentDecl, memory: Option<MemoryStore>) -> AgentRuntime {
    // Create agent runtime.
    //
    // Parameters:
    // - `decl` — input value
    // - `memory` — input value
    //
    // Returns:
    // AgentRuntime.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::create_agent_runtime(decl, memory);

    // Produce AgentRuntime { decl, memory } as the result.
    AgentRuntime { decl, memory }
}

pub fn agent_tool_names(decl: &AgentDecl) -> Vec<String> {
    // Agent tool names.
    //
    // Parameters:
    // - `decl` — input value
    //
    // Returns:
    // Vec<String>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::agent_tool_names(decl);

    // Match on decl and handle each case.
    match decl {
        AgentDecl::AgentDecl { tools, .. } => tools.clone(),
    }
}

pub fn agent_uses_models(decl: &AgentDecl) -> Vec<String> {
    // Agent uses models.
    //
    // Parameters:
    // - `decl` — input value
    //
    // Returns:
    // Vec<String>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::agent_uses_models(decl);

    // Match on decl and handle each case.
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
    // Execute agent plan.
    //
    // Parameters:
    // - `agent` — input value
    // - `executor` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::execute_agent_plan(agent, executor);

    // Compute plan body for the following logic.
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
        // From.
        //
        // Parameters:
        // - `kind` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::ai::from(kind);

        // Match on kind and handle each case.
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
        // Create a new instance.
        //
        // Parameters:
        // - `kind` — input value
        // - `limit` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_core::ai::new(kind, limit);

        // Compute default limit for the following logic.
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
        // Remember.
        //
        // Parameters:
        // - `self` — method receiver
        // - `key` — input value
        // - `value` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.remember(key, value);

        // Append into self.
        self.entries.push(MemoryEntry {
            key: key.into(),
            value,
            at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0),
        });

        // Take this path when self.entries.len() > self.limit.
        if self.entries.len() > self.limit {
            self.entries.remove(0);
        }
    }

    pub fn recall(&self, key: &str) -> Option<&RuntimeValue> {
        // Recall.
        //
        // Parameters:
        // - `self` — method receiver
        // - `key` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.recall(key);

        // Call entries on the current instance.
        self.entries
            .iter()
            .rev()
            .find(|e| e.key == key)
            .map(|e| &e.value)
    }

    pub fn recent(&self, count: usize) -> Vec<&RuntimeValue> {
        // Recent.
        //
        // Parameters:
        // - `self` — method receiver
        // - `count` — input value
        //
        // Returns:
        // Vec<&RuntimeValue>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.recent(count);

        // Call entries on the current instance.
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
        // Clear the value.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.clear();

        // Call clear on the current instance.
        self.entries.clear();
    }

    pub fn summary_for_prompt(&self) -> Option<String> {
        // Summary for prompt.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.summary_for_prompt();

        // skip further work when entries is empty.
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
    // Runtime safe action.
    //
    // Parameters:
    // - `linear` — input value
    // - `angular` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::runtime_safe_action(linear, angular);

    // Build a SafeAction runtime value.
    RuntimeValue::SafeAction { linear, angular }
}

pub fn runtime_action_proposal(
    linear: f64,
    angular: f64,
    source: impl Into<String>,
) -> RuntimeValue {
    // Runtime action proposal.
    //
    // Parameters:
    // - `linear` — input value
    // - `angular` — input value
    // - `source` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::runtime_action_proposal(linear, angular, source);

    // Build a ActionProposal runtime value.
    RuntimeValue::ActionProposal {
        linear,
        angular,
        source: source.into(),
        trace: Vec::new(),
    }
}

pub fn is_action_proposal(value: &RuntimeValue) -> bool {
    //
    // Parameters:
    // - `value` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::is_action_proposal(value);

    // Produce }) as the result.
    matches!(value, RuntimeValue::ActionProposal { .. })
}

/// Estimate model confidence from an ActionProposal (0.0–1.0).
pub fn proposal_confidence(value: &RuntimeValue) -> f64 {
    // Proposal confidence.
    //
    // Parameters:
    // - `value` — input value
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::proposal_confidence(value);

    // Match on value and handle each case.
    match value {
        RuntimeValue::ActionProposal { trace, .. } => {
            // Handle each input line.
            for line in trace {
                // Emit output when strip prefix provides a dist str.
                if let Some(dist_str) = line.strip_prefix("nearest_distance=") {
                    // Handle the success value from <f64>.
                    if let Ok(dist) = dist_str.parse::<f64>() {
                        return (dist / 5.0).clamp(0.05, 1.0);
                    }
                }

                // Check membership before continuing.
                if line.contains("decision=stop") {
                    return 0.95;
                }
            }
            0.75
        }
        RuntimeValue::Object { fields, .. } => {
            // Take this path when let Some(RuntimeValue::Number { value, .. }) = fields.get("confidence".
            if let Some(RuntimeValue::Number { value, .. }) = fields.get("confidence") {
                return value.clamp(0.0, 1.0);
            }
            0.75
        }
        _ => 1.0,
    }
}

pub const AI_CONFIDENCE_LOW_THRESHOLD: f64 = 0.5;

pub fn is_safe_action(value: &RuntimeValue) -> bool {
    //
    // Parameters:
    // - `value` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::is_safe_action(value);

    // Produce }) as the result.
    matches!(value, RuntimeValue::SafeAction { .. })
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionProposalFields {
    pub linear: f64,
    pub angular: f64,
    pub source: String,
}

pub fn proposal_from_value(value: &RuntimeValue) -> Option<ActionProposalFields> {
    // Proposal from value.
    //
    // Parameters:
    // - `value` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::proposal_from_value(value);

    // Match on value and handle each case.
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
    // Safe action from proposal.
    //
    // Parameters:
    // - `linear` — input value
    // - `angular` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::safe_action_from_proposal(linear, angular);

    // Produce runtime safe action as the result.
    runtime_safe_action(linear, angular)
}

pub fn wrap_completion(text: impl Into<String>, model: impl Into<String>) -> RuntimeValue {
    // Wrap completion.
    //
    // Parameters:
    // - `text` — input value
    // - `model` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::wrap_completion(text, model);

    // Build a Completion runtime value.
    RuntimeValue::Completion {
        text: text.into(),
        model: Some(model.into()),
    }
}

pub fn wrap_detection(label: &str, confidence: f64, nearest_distance: f64) -> RuntimeValue {
    // Wrap detection.
    //
    // Parameters:
    // - `label` — input value
    // - `confidence` — input value
    // - `nearest_distance` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::ai::wrap_detection(label, confidence, nearest_distance);

    // Create mutable fields for accumulating results.
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
        // Mock provider proposes motion.
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
        // let result = spanda_core::ai::mock_provider_proposes_motion();

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
        // Mock provider stops on halt prompt.
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
        // let result = spanda_core::ai::mock_provider_stops_on_halt_prompt();

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
        // Mock summarize scan.
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
        // let result = spanda_core::ai::mock_summarize_scan();

        let summary = mock_summarize(Some(&RuntimeValue::scan(1.25)), "mock");
        if let RuntimeValue::Completion { text, .. } = summary {
            assert!(text.contains("1.25"));
        } else {
            panic!("expected completion");
        }
    }

    #[test]
    fn memory_store_recalls_latest() {
        // Memory store recalls latest.
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
        // let result = spanda_core::ai::memory_store_recalls_latest();

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
        // Resolves ai imports.
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
        // let result = spanda_core::ai::resolves_ai_imports();

        assert!(resolve_ai_import("onnx.runtime").is_some());
        assert!(resolve_ai_import("openvino.runtime").is_some());
        assert!(list_ai_libraries().len() >= 4);
    }

    #[test]
    fn proposal_from_velocity() {
        // Proposal from velocity.
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
        // let result = spanda_core::ai::proposal_from_velocity();

        let proposal = proposal_from_value(&RuntimeValue::Velocity {
            linear: 0.5,
            angular: 0.1,
        });
        assert_eq!(proposal.unwrap().source, "velocity");
    }
}
