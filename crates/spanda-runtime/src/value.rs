//! Interpreter runtime value representations.
//!
use crate::robotics::{FleetRegistry, MissionRuntime};
use spanda_ast::nodes::UnitKind;
use spanda_ast::{CaptureResult, RegexPattern};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct PoseValue {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
    pub z: f64,
}

impl Default for PoseValue {
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
        // let value = spanda_core::runtime::default();

        // Assemble the struct fields and return it.
        Self {
            x: 0.0,
            y: 0.0,
            theta: 0.0,
            z: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Number {
        value: f64,
        unit: UnitKind,
    },
    Bool {
        value: bool,
    },
    String {
        value: String,
    },
    Regex {
        pattern: RegexPattern,
    },
    Capture {
        result: CaptureResult,
    },
    Void,
    Scan {
        nearest_distance: f64,
    },
    Pose {
        x: f64,
        y: f64,
        theta: f64,
        z: f64,
    },
    Velocity {
        linear: f64,
        angular: f64,
    },
    Trajectory {
        waypoints: Vec<PoseValue>,
    },
    Transform {
        from_frame: String,
        to_frame: String,
        pose: PoseValue,
    },
    Object {
        type_name: String,
        fields: HashMap<String, RuntimeValue>,
    },
    Enum {
        enum_name: String,
        variant: String,
        payloads: Vec<RuntimeValue>,
    },
    Sensor {
        name: String,
        sensor_type: String,
        library: Option<String>,
        hal_binding: Option<String>,
        topic: Option<String>,
    },
    Actuator {
        name: String,
        actuator_type: String,
    },
    Topic {
        name: String,
        message_type: String,
        topic_path: String,
    },
    Service {
        name: String,
        service_type: String,
    },
    Action {
        name: String,
        action_type: String,
    },
    Robot,
    Agent {
        name: String,
    },
    TraitObject {
        trait_name: String,
        agent: String,
    },
    Twin {
        name: String,
    },
    SafetyCtx,
    AuditCtx,
    LedgerCtx,
    WorldModelCtx,
    Identity {
        id: String,
        public_key: String,
    },
    Secret {
        name: String,
    },
    AiModel {
        name: String,
        model_type: String,
        provider: String,
    },
    ActionProposal {
        linear: f64,
        angular: f64,
        source: String,
        trace: Vec<String>,
    },
    SafeAction {
        linear: f64,
        angular: f64,
    },
    Goal {
        text: String,
    },
    SensorFusion {
        sensors: Vec<String>,
    },
    MissionControl {
        runtime: MissionRuntime,
    },
    NavigationControl {
        goal: Option<String>,
    },
    SlamControl,
    FleetControl {
        registry: FleetRegistry,
    },
    Completion {
        text: String,
        model: Option<String>,
    },
    Embedding {
        dimensions: usize,
        vector: Vec<f64>,
    },
    Result {
        ok: bool,
        value: Box<RuntimeValue>,
    },
    Option {
        present: bool,
        value: Option<Box<RuntimeValue>>,
    },
    Bytes {
        data: Vec<u8>,
    },
    Null,
    Future {
        func_name: String,
        args: Vec<RuntimeValue>,
        resolved: Option<Box<RuntimeValue>>,
    },
    TaskHandle {
        id: u64,
    },
    Channel {
        id: u64,
    },
}

impl RuntimeValue {
    pub fn number(value: f64, unit: UnitKind) -> Self {
        // Number.
        //
        // Parameters:
        // - `value` — input value
        // - `unit` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::runtime::number(value, unit);

        // Build a Number runtime value.
        RuntimeValue::Number { value, unit }
    }

    pub fn string(value: impl Into<String>) -> Self {
        // String.
        //
        // Parameters:
        // - `value` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::runtime::string(value);

        // Build a String runtime value.
        RuntimeValue::String {
            value: value.into(),
        }
    }

    pub fn object(type_name: impl Into<String>, fields: HashMap<String, RuntimeValue>) -> Self {
        // Object.
        //
        // Parameters:
        // - `type_name` — input value
        // - `fields` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::runtime::object(type_name, fields);

        // Build a Object runtime value.
        RuntimeValue::Object {
            type_name: type_name.into(),
            fields,
        }
    }

    pub fn scan(nearest_distance: f64) -> Self {
        // Scan.
        //
        // Parameters:
        // - `nearest_distance` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::runtime::scan(nearest_distance);

        // Build a Scan runtime value.
        RuntimeValue::Scan { nearest_distance }
    }

    pub fn as_number(&self) -> Option<f64> {
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
        // let result = instance.as_number();

        // Dispatch based on the enum variant or current state.
        match self {
            RuntimeValue::Number { value, .. } => Some(*value),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
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
        // let result = instance.as_string();

        // Dispatch based on the enum variant or current state.
        match self {
            RuntimeValue::String { value } => Some(value),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MotionCommand {
    Drive {
        linear: f64,
        angular: f64,
        actuator: String,
    },
    Stop {
        actuator: String,
    },
    MoveTo {
        x: f64,
        y: f64,
        z: f64,
        actuator: String,
    },
    Follow {
        waypoints: Vec<PoseValue>,
        actuator: String,
    },
    Grip {
        actuator: String,
    },
    Release {
        actuator: String,
    },
    Open {
        actuator: String,
    },
    SetThrust {
        thrust: f64,
        actuator: String,
    },
    Hover {
        actuator: String,
    },
}
pub fn format_runtime_value(value: &RuntimeValue) -> String {
    // Format runtime value.
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
    // let result = spanda_core::runtime::format_runtime_value(value);

    // Match on value and handle each case.
    match value {
        RuntimeValue::Number { value, unit } => {
            // Take the branch when *unit equals None.
            if *unit == UnitKind::None {
                value.to_string()
            } else {
                format!("{value} {}", unit.as_str())
            }
        }
        RuntimeValue::Bool { value } => value.to_string(),
        RuntimeValue::String { value } => value.clone(),
        RuntimeValue::Void => "void".into(),
        RuntimeValue::Enum {
            variant, payloads, ..
        } => {
            // Skip further work when payloads is empty.
            if payloads.is_empty() {
                variant.clone()
            } else {
                format!(
                    "{variant}({})",
                    payloads
                        .iter()
                        .map(format_runtime_value)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        RuntimeValue::TraitObject { trait_name, agent } => {
            format!("dyn {trait_name}@{agent}")
        }
        RuntimeValue::Agent { name } => format!("agent {name}"),
        RuntimeValue::Object { type_name, fields } => format!(
            "{type_name} {{ {} }}",
            fields
                .iter()
                .map(|(k, v)| format!("{k}: {}", format_runtime_value(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        other => format!("{other:?}"),
    }
}
pub fn runtime_pose(x: f64, y: f64, theta: f64, z: f64) -> RuntimeValue {
    // Runtime pose.
    //
    // Parameters:
    // - `x` — input value
    // - `y` — input value
    // - `theta` — input value
    // - `z` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::runtime::runtime_pose(x, y, theta, z);

    // Build a Pose runtime value.
    RuntimeValue::Pose { x, y, theta, z }
}

pub fn runtime_velocity(linear: f64, angular: f64) -> RuntimeValue {
    // Runtime velocity.
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
    // let result = spanda_core::runtime::runtime_velocity(linear, angular);

    // Build a Velocity runtime value.
    RuntimeValue::Velocity { linear, angular }
}

pub fn runtime_trajectory(waypoints: Vec<PoseValue>) -> RuntimeValue {
    // Runtime trajectory.
    //
    // Parameters:
    // - `waypoints` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::runtime::runtime_trajectory(waypoints);

    // Build a Trajectory runtime value.
    RuntimeValue::Trajectory { waypoints }
}
pub fn get_pose_fields(val: &RuntimeValue) -> Option<PoseValue> {
    //
    // Parameters:
    // - `val` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::runtime::get_pose_fields(val);

    // Match on val and handle each case.
    match val {
        RuntimeValue::Pose { x, y, theta, z } => Some(PoseValue {
            x: *x,
            y: *y,
            theta: *theta,
            z: *z,
        }),
        _ => None,
    }
}

pub fn get_velocity_fields(val: &RuntimeValue) -> Option<(f64, f64)> {
    //
    // Parameters:
    // - `val` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::runtime::get_velocity_fields(val);

    // Match on val and handle each case.
    match val {
        RuntimeValue::Velocity { linear, angular } => Some((*linear, *angular)),
        _ => None,
    }
}

pub fn get_trajectory_waypoints(val: &RuntimeValue) -> Option<Vec<PoseValue>> {
    //
    // Parameters:
    // - `val` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::runtime::get_trajectory_waypoints(val);

    // Match on val and handle each case.
    match val {
        RuntimeValue::Trajectory { waypoints } => Some(waypoints.clone()),
        _ => None,
    }
}

pub fn get_number(val: &RuntimeValue, default: f64) -> f64 {
    //
    // Parameters:
    // - `val` — input value
    // - `default` — input value
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::runtime::get_number(val, default);

    // Produce unwrap or as the result.
    val.as_number().unwrap_or(default)
}

pub fn get_string(val: &RuntimeValue, default: &str) -> String {
    //
    // Parameters:
    // - `val` — input value
    // - `default` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::runtime::get_string(val, default);

    // Produce to string as the result.
    val.as_string().unwrap_or(default).to_string()
}
