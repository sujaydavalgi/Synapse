//! Minimal world-model belief state from recent observations.
//!

use crate::value::RuntimeValue;
use serde_json::{json, Value};

const MAX_OBSERVATIONS: usize = 32;

/// Rolling observation buffer with a simple confidence belief.
#[derive(Debug, Clone, Default)]
pub struct WorldModelRuntime {
    observations: Vec<Value>,
    belief_confidence: f64,
}

impl WorldModelRuntime {
    /// Create an empty world model.
    pub fn new() -> Self {
        // Allocate an empty rolling observation buffer.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Empty `WorldModelRuntime`.
        //
        // Options:
        // None.
        //
        // Example:
        // let model = WorldModelRuntime::new();

        Self::default()
    }

    /// Record one observation and refresh belief confidence.
    pub fn update(&mut self, observation: &RuntimeValue) -> f64 {
        // Append an observation and recompute belief confidence.
        //
        // Parameters:
        // - `observation` — runtime value from `observe` / fusion
        //
        // Returns:
        // Updated belief confidence in `[0, 1]`.
        //
        // Options:
        // Keeps the most recent `MAX_OBSERVATIONS` entries.
        //
        // Example:
        // let confidence = model.update(&obs);

        self.observations
            .push(runtime_value_to_json(observation));
        if self.observations.len() > MAX_OBSERVATIONS {
            let overflow = self.observations.len() - MAX_OBSERVATIONS;
            self.observations.drain(0..overflow);
        }
        self.belief_confidence = Self::confidence_from_observations(&self.observations);
        self.belief_confidence
    }

    /// Return the current belief confidence.
    pub fn belief_confidence(&self) -> f64 {
        // Return the latest belief confidence score.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Confidence in `[0, 1]`.
        //
        // Options:
        // None.
        //
        // Example:
        // let confidence = model.belief_confidence();

        self.belief_confidence
    }

    /// Export observations and belief for cloud upload or replay.
    pub fn export_json(&self) -> Value {
        // Serialize observations and belief for telemetry export.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // JSON object with `observations` and `belief_confidence`.
        //
        // Options:
        // None.
        //
        // Example:
        // let payload = model.export_json();

        json!({
            "observations": self.observations,
            "belief_confidence": self.belief_confidence,
        })
    }

    fn confidence_from_observations(observations: &[Value]) -> f64 {
        if observations.is_empty() {
            return 0.0;
        }
        let capped = observations.len().min(MAX_OBSERVATIONS) as f64;
        (capped / MAX_OBSERVATIONS as f64).clamp(0.1, 1.0)
    }
}

fn runtime_value_to_json(value: &RuntimeValue) -> Value {
    match value {
        RuntimeValue::Number { value, unit } => {
            json!({ "kind": "number", "value": value, "unit": format!("{unit:?}") })
        }
        RuntimeValue::Bool { value } => json!({ "kind": "bool", "value": value }),
        RuntimeValue::String { value } => json!({ "kind": "string", "value": value }),
        RuntimeValue::Object { type_name, fields } => {
            let mut map = serde_json::Map::new();
            map.insert("kind".into(), json!("object"));
            map.insert("type_name".into(), json!(type_name));
            for (key, field_value) in fields {
                map.insert(key.clone(), runtime_value_to_json(field_value));
            }
            Value::Object(map)
        }
        other => json!({ "kind": "opaque", "value": format!("{other:?}") }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_ast::nodes::UnitKind;

    #[test]
    fn update_increases_belief_with_observations() {
        let mut model = WorldModelRuntime::new();
        assert_eq!(model.belief_confidence(), 0.0);
        let obs = RuntimeValue::Number {
            value: 1.0,
            unit: UnitKind::None,
        };
        let belief = model.update(&obs);
        assert!(belief > 0.0);
        assert_eq!(model.belief_confidence(), belief);
    }
}
