//! Spanda type system: primitives, generics, physical units, and domain types.

use crate::ast::{SpandaType, UnitKind};
use crate::units::{self, PhysicalCategory};
use std::collections::HashMap;

/// Generic type constructor arity.
#[derive(Debug, Clone, Copy)]
pub struct GenericDef {
    pub name: &'static str,
    pub arity: usize,
    pub namespace: Option<&'static str>,
}

/// Resolve a type name (optionally qualified) to a `SpandaType`.
pub fn resolve_type_name(name: &str) -> Result<SpandaType, String> {
    let name = name.strip_prefix("std.").unwrap_or(name);
    let name = name.rsplit('.').next().unwrap_or(name);

    match name {
        // Foundation
        "Int" | "int" => Ok(SpandaType::Int),
        "Float" | "float" => Ok(SpandaType::Float),
        "Bool" | "bool" => Ok(SpandaType::Bool),
        "String" | "string" => Ok(SpandaType::String),
        "Char" | "char" => Ok(SpandaType::Char),
        "Bytes" | "bytes" => Ok(SpandaType::Bytes),
        "Null" | "null" => Ok(SpandaType::Null),
        "Void" | "void" => Ok(SpandaType::Void),
        // Time
        "Time" => Ok(SpandaType::Named {
            name: "Time".into(),
        }),
        "Duration" => Ok(SpandaType::Number { unit: UnitKind::Ms }),
        "Timestamp" => Ok(SpandaType::Named {
            name: "Timestamp".into(),
        }),
        "Interval" => Ok(SpandaType::Named {
            name: "Interval".into(),
        }),
        // Physical unit types
        "Distance" => Ok(SpandaType::Number { unit: UnitKind::M }),
        "Velocity" => Ok(SpandaType::Velocity),
        "Acceleration" => Ok(SpandaType::Number {
            unit: UnitKind::MPerS2,
        }),
        "Angle" => Ok(SpandaType::Number {
            unit: UnitKind::Rad,
        }),
        "AngularVelocity" => Ok(SpandaType::Number {
            unit: UnitKind::RadPerS,
        }),
        "Mass" | "Force" | "Power" | "Voltage" | "Current" | "Temperature" | "Pressure"
        | "Humidity" | "Illuminance" | "Luminance" | "Concentration" | "SoundLevel"
        | "MagneticField" | "RotationalSpeed" | "Torque" | "Energy" | "UvIndex" | "Ph"
        | "Conductivity" | "ParticulateMatter" | "Turbidity" | "Salinity" | "Radiation"
        | "SoilMoisture" => Ok(SpandaType::Named {
            name: name.to_string(),
        }),
        // Spatial / robotics
        "Point2D" | "Point3D" | "Vector2D" | "Vector3D" | "Quaternion" | "Pose" => {
            Ok(SpandaType::Pose)
        }
        "Transform" => Ok(SpandaType::Transform),
        "Trajectory" | "Path" => Ok(SpandaType::Trajectory),
        "Waypoint" | "MotionCommand" | "ControlSignal" | "PIDConfig" => Ok(SpandaType::Named {
            name: name.to_string(),
        }),
        // Sensors
        "CameraFrame" | "Image" | "DepthImage" | "PointCloud" | "LidarScan" => Ok(SpandaType::Scan),
        "GpsFix" | "ImuData" | "AudioFrame" => Ok(SpandaType::Named {
            name: name.to_string(),
        }),
        // AI
        "LLM" | "VisionModel" | "EmbeddingModel" | "Prompt" | "Completion" | "Embedding"
        | "Token" | "Context" | "Memory" | "Plan" | "ReasoningTrace" => Ok(SpandaType::Named {
            name: name.to_string(),
        }),
        // Agent / autonomy
        "Agent" | "Goal" | "Task" | "Skill" | "Capability" | "Intent" => Ok(SpandaType::Named {
            name: name.to_string(),
        }),
        "ActionProposal" => Ok(SpandaType::Named {
            name: "ActionProposal".into(),
        }),
        "SafeAction" => Ok(SpandaType::Named {
            name: "SafeAction".into(),
        }),
        // HRI
        "Command" | "Conversation" | "Speech" | "Gesture" | "Emotion" | "Feedback" | "Approval" => {
            Ok(SpandaType::Named {
                name: name.to_string(),
            })
        }
        // Uncertainty
        "Confidence" | "Prediction" | "Probability" => Ok(SpandaType::Named {
            name: name.to_string(),
        }),
        // Safety
        "Risk" | "Hazard" | "SafetyConstraint" | "EmergencyStop" => Ok(SpandaType::Named {
            name: name.to_string(),
        }),
        // Digital twin
        "Twin" | "SimulationState" | "Telemetry" | "Replay" | "Fault" | "Scenario" => {
            Ok(SpandaType::Named {
                name: name.to_string(),
            })
        }
        // Network / communication (std.network)
        "Transport"
        | "QosProfile"
        | "QoS"
        | "Bandwidth"
        | "Latency"
        | "TopicPath"
        | "ServiceEndpoint"
        | "MessageEnvelope"
        | "DiscoveryFilter"
        | "NetworkRequirements"
        | "Reliability"
        | "HistoryPolicy"
        | "CommBus"
        | "Endpoint" => Ok(SpandaType::Named {
            name: name.to_string(),
        }),
        // Advanced
        "KnowledgeGraph" | "Belief" | "Observation" | "WorldModel" | "Policy" | "Reward"
        | "StateEstimate" | "SensorFusion" | "FusedObservation" => Ok(SpandaType::Named {
            name: name.to_string(),
        }),
        // Legacy aliases
        "Scan" => Ok(SpandaType::Scan),
        other if is_known_domain_type(other) => Ok(SpandaType::Named {
            name: other.to_string(),
        }),
        other => Err(format!("Unknown type '{other}'")),
    }
}

pub fn resolve_generic_type(name: &str, args: &[SpandaType]) -> Result<SpandaType, String> {
    let base = name.rsplit('.').next().unwrap_or(name);
    let expected = generic_arity(base).ok_or_else(|| format!("Unknown generic type '{base}'"))?;
    if args.len() != expected {
        return Err(format!(
            "Type '{base}' expects {expected} type argument(s), got {}",
            args.len()
        ));
    }
    Ok(SpandaType::Generic {
        name: base.to_string(),
        type_args: args.to_vec(),
    })
}

fn generic_arity(name: &str) -> Option<usize> {
    match name {
        "Array" | "Set" | "Queue" | "Stack" | "Topic" | "Message" => Some(1),
        "Map" | "Service" | "Tuple" => Some(2),
        "Action" => Some(3),
        "Endpoint" => Some(1),
        _ => None,
    }
}

fn is_known_domain_type(name: &str) -> bool {
    KNOWN_DOMAIN_TYPES.contains(&name)
}

const KNOWN_DOMAIN_TYPES: &[&str] = &[
    "Mass",
    "Force",
    "Power",
    "Voltage",
    "Current",
    "Temperature",
    "Pressure",
    "Humidity",
    "Illuminance",
    "Luminance",
    "Concentration",
    "SoundLevel",
    "MagneticField",
    "RotationalSpeed",
    "Torque",
    "Energy",
    "UvIndex",
    "Ph",
    "Conductivity",
    "ParticulateMatter",
    "Turbidity",
    "Salinity",
    "Radiation",
    "SoilMoisture",
    "Time",
    "Timestamp",
    "Interval",
    "Waypoint",
    "MotionCommand",
    "ControlSignal",
    "PIDConfig",
    "GpsFix",
    "ImuData",
    "AudioFrame",
    "Prompt",
    "Completion",
    "Embedding",
    "Token",
    "Context",
    "Memory",
    "Plan",
    "ReasoningTrace",
    "Agent",
    "Goal",
    "Task",
    "Skill",
    "Capability",
    "Intent",
    "Approval",
    "Command",
    "Conversation",
    "Speech",
    "Gesture",
    "Emotion",
    "Feedback",
    "Confidence",
    "Prediction",
    "Probability",
    "Risk",
    "Hazard",
    "SafetyConstraint",
    "Twin",
    "SimulationState",
    "Telemetry",
    "Replay",
    "Fault",
    "Scenario",
    "KnowledgeGraph",
    "Belief",
    "Observation",
    "WorldModel",
    "Policy",
    "Reward",
    "StateEstimate",
    "SensorFusion",
    "FusedObservation",
    "LLM",
    "VisionModel",
    "EmbeddingModel",
    "CameraFrame",
    "Image",
    "DepthImage",
    "PointCloud",
    "LidarScan",
    "Transport",
    "QosProfile",
    "QoS",
    "Bandwidth",
    "Latency",
    "TopicPath",
    "ServiceEndpoint",
    "MessageEnvelope",
    "DiscoveryFilter",
    "NetworkRequirements",
    "Reliability",
    "HistoryPolicy",
    "CommBus",
    "Endpoint",
];

/// Physical category used to reject invalid unit operations.
pub fn physical_category(ty: &SpandaType) -> PhysicalCategory {
    match ty {
        SpandaType::Int | SpandaType::Float => PhysicalCategory::Scalar,
        SpandaType::Number { unit, .. } => units::unit_category(*unit),
        SpandaType::Velocity => PhysicalCategory::Velocity,
        SpandaType::Pose => PhysicalCategory::Distance,
        SpandaType::Named { name } => match name.as_str() {
            "Distance" => PhysicalCategory::Distance,
            "Duration" | "Time" | "Timestamp" | "Interval" => PhysicalCategory::Duration,
            "Velocity" => PhysicalCategory::Velocity,
            "Acceleration" => PhysicalCategory::Acceleration,
            "Angle" | "AngularVelocity" => PhysicalCategory::AngularVelocity,
            "Mass" => PhysicalCategory::Mass,
            "Force" => PhysicalCategory::Force,
            "Power" => PhysicalCategory::Power,
            "Voltage" => PhysicalCategory::Voltage,
            "Current" => PhysicalCategory::Current,
            "Temperature" => PhysicalCategory::Temperature,
            "Pressure" => PhysicalCategory::Pressure,
            "Humidity" => PhysicalCategory::Humidity,
            "Illuminance" => PhysicalCategory::Illuminance,
            "Luminance" => PhysicalCategory::Luminance,
            "Concentration" => PhysicalCategory::Concentration,
            "SoundLevel" => PhysicalCategory::SoundLevel,
            "MagneticField" => PhysicalCategory::MagneticField,
            "RotationalSpeed" => PhysicalCategory::RotationalSpeed,
            "Torque" => PhysicalCategory::Torque,
            "Energy" => PhysicalCategory::Energy,
            "UvIndex" => PhysicalCategory::UvIndex,
            "Ph" => PhysicalCategory::Ph,
            "Conductivity" => PhysicalCategory::Conductivity,
            "ParticulateMatter" => PhysicalCategory::ParticulateMatter,
            "Turbidity" => PhysicalCategory::Turbidity,
            "Salinity" => PhysicalCategory::Salinity,
            "Radiation" => PhysicalCategory::Radiation,
            "SoilMoisture" => PhysicalCategory::SoilMoisture,
            _ => PhysicalCategory::Scalar,
        },
        _ => PhysicalCategory::Scalar,
    }
}

/// Returns `None` when the operation is invalid (e.g. distance + duration).
pub fn binary_physical_op_allowed(
    op: crate::ast::BinaryOp,
    left: &SpandaType,
    right: &SpandaType,
) -> bool {
    use crate::ast::BinaryOp;
    let cat_l = physical_category(left);
    let cat_r = physical_category(right);

    match op {
        BinaryOp::Add | BinaryOp::Sub => {
            if cat_l == PhysicalCategory::Scalar && cat_r == PhysicalCategory::Scalar {
                return true;
            }
            cat_l == cat_r && cat_l != PhysicalCategory::Scalar
        }
        BinaryOp::Lt
        | BinaryOp::Lte
        | BinaryOp::Gt
        | BinaryOp::Gte
        | BinaryOp::Eq
        | BinaryOp::Neq => cat_l == cat_r,
        BinaryOp::Mul | BinaryOp::Div => true,
        BinaryOp::And | BinaryOp::Or => {
            matches!(left, SpandaType::Bool) && matches!(right, SpandaType::Bool)
        }
    }
}

pub fn is_action_proposal_type(ty: &SpandaType) -> bool {
    matches!(
        ty,
        SpandaType::Named { name } if name == "ActionProposal"
    )
}

pub fn is_safe_action_type(ty: &SpandaType) -> bool {
    matches!(
        ty,
        SpandaType::Named { name } if name == "SafeAction"
    )
}

pub fn std_namespaces() -> HashMap<&'static str, &'static [&'static str]> {
    let mut m = HashMap::new();
    m.insert(
        "std.time",
        &["Time", "Duration", "Timestamp", "Interval"][..],
    );
    m.insert(
        "std.units",
        &[
            "Distance",
            "Velocity",
            "Acceleration",
            "Angle",
            "AngularVelocity",
            "Mass",
            "Force",
            "Power",
            "Voltage",
            "Current",
            "Temperature",
            "Pressure",
            "Humidity",
            "Illuminance",
            "Luminance",
            "Concentration",
            "SoundLevel",
            "MagneticField",
            "RotationalSpeed",
            "Torque",
            "Energy",
            "UvIndex",
            "Ph",
            "Conductivity",
            "ParticulateMatter",
            "Turbidity",
            "Salinity",
            "Radiation",
            "SoilMoisture",
        ][..],
    );
    m.insert(
        "std.spatial",
        &[
            "Point2D",
            "Point3D",
            "Vector2D",
            "Vector3D",
            "Quaternion",
            "Pose",
            "Transform",
            "Trajectory",
            "Path",
            "Waypoint",
        ][..],
    );
    m.insert(
        "std.ai",
        &[
            "LLM",
            "VisionModel",
            "EmbeddingModel",
            "Prompt",
            "Completion",
            "Embedding",
            "Token",
            "Context",
            "Memory",
            "Plan",
            "ReasoningTrace",
        ][..],
    );
    m.insert(
        "std.robotics",
        &[
            "MotionCommand",
            "ControlSignal",
            "PIDConfig",
            "ActionProposal",
            "SafeAction",
            "Agent",
            "Goal",
            "Task",
            "Skill",
            "Capability",
            "Intent",
        ][..],
    );
    m.insert(
        "std.sensors",
        &[
            "CameraFrame",
            "Image",
            "DepthImage",
            "PointCloud",
            "LidarScan",
            "GpsFix",
            "ImuData",
            "AudioFrame",
        ][..],
    );
    m.insert(
        "std.safety",
        &[
            "Risk",
            "Hazard",
            "SafetyConstraint",
            "EmergencyStop",
            "SafeAction",
        ][..],
    );
    m.insert(
        "std.twin",
        &[
            "Twin",
            "SimulationState",
            "Telemetry",
            "Replay",
            "Fault",
            "Scenario",
        ][..],
    );
    m.insert(
        "std.hri",
        &[
            "Command",
            "Conversation",
            "Speech",
            "Gesture",
            "Emotion",
            "Feedback",
        ][..],
    );
    m.insert(
        "std.network",
        &[
            "Transport",
            "QosProfile",
            "QoS",
            "Bandwidth",
            "Latency",
            "TopicPath",
            "ServiceEndpoint",
            "MessageEnvelope",
            "DiscoveryFilter",
            "NetworkRequirements",
            "Reliability",
            "HistoryPolicy",
            "CommBus",
            "Endpoint",
            "Topic",
            "Message",
            "Service",
            "Action",
        ][..],
    );
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_type() {
        assert!(resolve_type_name("NotARealType").is_err());
    }

    #[test]
    fn resolves_generics_with_arity() {
        let goal = SpandaType::Named {
            name: "Goal".into(),
        };
        let err = resolve_generic_type("Array", &[]).unwrap_err();
        assert!(err.contains("expects 1"));
        let ok = resolve_generic_type("Array", &[goal]).unwrap();
        assert!(matches!(ok, SpandaType::Generic { .. }));
    }

    #[test]
    fn rejects_mixed_physical_add() {
        let vel = SpandaType::Velocity;
        let volt = SpandaType::Named {
            name: "Voltage".into(),
        };
        assert!(!binary_physical_op_allowed(
            crate::ast::BinaryOp::Add,
            &vel,
            &volt
        ));
    }
}
