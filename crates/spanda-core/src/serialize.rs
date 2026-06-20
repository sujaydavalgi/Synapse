//! Runtime value serialization (JSON, YAML, binary).

use crate::ast::UnitKind;
use crate::error::SpandaError;
use crate::runtime::RuntimeError;
use crate::runtime::RuntimeValue;
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;

pub fn serialize_value(value: &RuntimeValue, format: &str) -> Result<RuntimeValue, SpandaError> {
    match format.to_ascii_lowercase().as_str() {
        "json" => Ok(RuntimeValue::String {
            value: serde_json::to_string(&runtime_to_json(value)).map_err(|e| {
                RuntimeError::new(format!("serialize json failed: {e}"), 0).into_spanda()
            })?,
        }),
        "yaml" => {
            let yaml = serde_yaml::to_string(&runtime_to_json(value)).map_err(|e| {
                RuntimeError::new(format!("serialize yaml failed: {e}"), 0).into_spanda()
            })?;
            Ok(RuntimeValue::String { value: yaml })
        }
        "binary" => {
            let bytes = serde_json::to_vec(&runtime_to_json(value)).map_err(|e| {
                RuntimeError::new(format!("serialize binary failed: {e}"), 0).into_spanda()
            })?;
            Ok(RuntimeValue::Bytes { data: bytes })
        }
        other => Err(RuntimeError::new(
            format!("Unknown serialize format '{other}' (use json, yaml, or binary)"),
            0,
        )
        .into_spanda()),
    }
}

pub fn deserialize_value(data: &RuntimeValue, format: &str) -> Result<RuntimeValue, SpandaError> {
    match format.to_ascii_lowercase().as_str() {
        "json" => {
            let text = runtime_string(data)?;
            let parsed: JsonValue = serde_json::from_str(&text).map_err(|e| {
                RuntimeError::new(format!("deserialize json failed: {e}"), 0).into_spanda()
            })?;
            json_to_runtime(&parsed)
        }
        "yaml" => {
            let text = runtime_string(data)?;
            let parsed: JsonValue = serde_yaml::from_str(&text).map_err(|e| {
                RuntimeError::new(format!("deserialize yaml failed: {e}"), 0).into_spanda()
            })?;
            json_to_runtime(&parsed)
        }
        "binary" => {
            let bytes = match data {
                RuntimeValue::Bytes { data } => data.clone(),
                RuntimeValue::String { value } => value.as_bytes().to_vec(),
                _ => {
                    return Err(RuntimeError::new(
                        "deserialize binary expects Bytes or String data",
                        0,
                    )
                    .into_spanda())
                }
            };
            let parsed: JsonValue = serde_json::from_slice(&bytes).map_err(|e| {
                RuntimeError::new(format!("deserialize binary failed: {e}"), 0).into_spanda()
            })?;
            json_to_runtime(&parsed)
        }
        other => Err(RuntimeError::new(
            format!("Unknown deserialize format '{other}' (use json, yaml, or binary)"),
            0,
        )
        .into_spanda()),
    }
}

fn runtime_string(value: &RuntimeValue) -> Result<String, SpandaError> {
    match value {
        RuntimeValue::String { value } => Ok(value.clone()),
        _ => {
            Err(RuntimeError::new("Expected string data for text deserialization", 0).into_spanda())
        }
    }
}

fn runtime_to_json(value: &RuntimeValue) -> JsonValue {
    match value {
        RuntimeValue::Number { value, unit } => json!({
            "kind": "number",
            "value": value,
            "unit": unit.as_str(),
        }),
        RuntimeValue::Bool { value } => json!({ "kind": "bool", "value": value }),
        RuntimeValue::String { value } => json!({ "kind": "string", "value": value }),
        RuntimeValue::Void | RuntimeValue::Null => JsonValue::Null,
        RuntimeValue::Bytes { data } => json!({ "kind": "bytes", "data": data }),
        RuntimeValue::Enum { enum_name, variant } => {
            json!({ "kind": "enum", "enum": enum_name, "variant": variant })
        }
        RuntimeValue::Result { ok, value } => json!({
            "kind": if *ok { "ok" } else { "err" },
            "value": runtime_to_json(value),
        }),
        RuntimeValue::Option { present, value } => {
            if *present {
                json!({ "kind": "some", "value": runtime_to_json(value.as_ref().unwrap()) })
            } else {
                json!({ "kind": "none" })
            }
        }
        RuntimeValue::Object { type_name, fields } => {
            let mut map = serde_json::Map::new();
            map.insert("kind".into(), json!("object"));
            map.insert("type".into(), json!(type_name));
            let mut field_map = serde_json::Map::new();
            for (k, v) in fields {
                field_map.insert(k.clone(), runtime_to_json(v));
            }
            map.insert("fields".into(), JsonValue::Object(field_map));
            JsonValue::Object(map)
        }
        RuntimeValue::Pose { x, y, theta, z } => {
            json!({ "kind": "pose", "x": x, "y": y, "theta": theta, "z": z })
        }
        RuntimeValue::Velocity { linear, angular } => {
            json!({ "kind": "velocity", "linear": linear, "angular": angular })
        }
        other => json!({ "kind": "opaque", "type": runtime_kind_name(other) }),
    }
}

fn runtime_kind_name(value: &RuntimeValue) -> &'static str {
    match value {
        RuntimeValue::Scan { .. } => "scan",
        RuntimeValue::Trajectory { .. } => "trajectory",
        RuntimeValue::Transform { .. } => "transform",
        RuntimeValue::Sensor { .. } => "sensor",
        RuntimeValue::Actuator { .. } => "actuator",
        RuntimeValue::Topic { .. } => "topic",
        RuntimeValue::Service { .. } => "service",
        RuntimeValue::Action { .. } => "action",
        RuntimeValue::Future { .. } => "future",
        RuntimeValue::Channel { .. } => "channel",
        RuntimeValue::Bytes { .. } => "bytes",
        RuntimeValue::Null => "null",
        _ => "value",
    }
}

fn json_to_runtime(value: &JsonValue) -> Result<RuntimeValue, SpandaError> {
    if value.is_null() {
        return Ok(RuntimeValue::Null);
    }
    let obj = value
        .as_object()
        .ok_or_else(|| RuntimeError::new("deserialize expected JSON object", 0).into_spanda())?;
    let kind = obj.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    match kind {
        "number" => {
            let n = obj.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let unit = obj
                .get("unit")
                .and_then(|v| v.as_str())
                .map(UnitKind::from_lexeme)
                .unwrap_or(UnitKind::None);
            Ok(RuntimeValue::Number { value: n, unit })
        }
        "bool" => Ok(RuntimeValue::Bool {
            value: obj.get("value").and_then(|v| v.as_bool()).unwrap_or(false),
        }),
        "string" => Ok(RuntimeValue::String {
            value: obj
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }),
        "bytes" => {
            let data = obj
                .get("data")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_u64().map(|n| n as u8))
                        .collect()
                })
                .unwrap_or_default();
            Ok(RuntimeValue::Bytes { data })
        }
        "enum" => Ok(RuntimeValue::Enum {
            enum_name: obj
                .get("enum")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            variant: obj
                .get("variant")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }),
        "ok" => Ok(RuntimeValue::Result {
            ok: true,
            value: Box::new(json_to_runtime(
                obj.get("value").unwrap_or(&JsonValue::Null),
            )?),
        }),
        "err" => Ok(RuntimeValue::Result {
            ok: false,
            value: Box::new(json_to_runtime(
                obj.get("value").unwrap_or(&JsonValue::Null),
            )?),
        }),
        "some" => Ok(RuntimeValue::Option {
            present: true,
            value: Some(Box::new(json_to_runtime(
                obj.get("value").unwrap_or(&JsonValue::Null),
            )?)),
        }),
        "none" => Ok(RuntimeValue::Option {
            present: false,
            value: None,
        }),
        "object" => {
            let type_name = obj
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("Object")
                .to_string();
            let mut fields = HashMap::new();
            if let Some(field_obj) = obj.get("fields").and_then(|v| v.as_object()) {
                for (k, v) in field_obj {
                    fields.insert(k.clone(), json_to_runtime(v)?);
                }
            }
            Ok(RuntimeValue::Object { type_name, fields })
        }
        "pose" => Ok(RuntimeValue::Pose {
            x: obj.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
            y: obj.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
            theta: obj.get("theta").and_then(|v| v.as_f64()).unwrap_or(0.0),
            z: obj.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0),
        }),
        "velocity" => Ok(RuntimeValue::Velocity {
            linear: obj.get("linear").and_then(|v| v.as_f64()).unwrap_or(0.0),
            angular: obj.get("angular").and_then(|v| v.as_f64()).unwrap_or(0.0),
        }),
        _ => Err(
            RuntimeError::new(format!("Unsupported deserialized kind '{kind}'"), 0).into_spanda(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_json_pose() {
        let value = RuntimeValue::Pose {
            x: 1.0,
            y: 2.0,
            theta: 0.5,
            z: 0.0,
        };
        let serialized = serialize_value(&value, "json").unwrap();
        let restored = deserialize_value(&serialized, "json").unwrap();
        assert_eq!(value, restored);
    }
}
