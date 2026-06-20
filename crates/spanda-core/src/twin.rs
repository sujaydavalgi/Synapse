use crate::runtime::RuntimeValue;
use std::collections::HashMap;

/// Shadow state for a digital twin with optional replay buffer.
pub struct TwinRuntime {
    pub name: String,
    pub mirrors: Vec<String>,
    pub replay: bool,
    pub telemetry_sync: bool,
    pub faults_sync: bool,
    pub events_sync: bool,
    pub shadow: HashMap<String, RuntimeValue>,
    replay_buffer: Vec<HashMap<String, RuntimeValue>>,
}

impl TwinRuntime {
    pub fn new(name: String, mirrors: Vec<String>, replay: bool) -> Self {
        Self {
            name,
            mirrors,
            replay,
            telemetry_sync: false,
            faults_sync: false,
            events_sync: false,
            shadow: HashMap::new(),
            replay_buffer: Vec::new(),
        }
    }

    pub fn with_sync(mut self, telemetry: bool, replay: bool, faults: bool, events: bool) -> Self {
        self.telemetry_sync = telemetry;
        if replay {
            self.replay = true;
        }
        self.faults_sync = faults;
        self.events_sync = events;
        if telemetry {
            for field in ["pose", "velocity"] {
                if !self.mirrors.iter().any(|m| m == field) {
                    self.mirrors.push(field.to_string());
                }
            }
        }
        self
    }

    pub fn snapshot(&mut self, field: &str, value: RuntimeValue) {
        if self.mirrors.iter().any(|m| m == field) {
            self.shadow.insert(field.to_string(), value);
        }
    }

    pub fn commit_frame(&mut self) {
        if self.replay && !self.shadow.is_empty() {
            self.replay_buffer.push(self.shadow.clone());
        }
    }

    pub fn replay_frame_count(&self) -> usize {
        self.replay_buffer.len()
    }

    pub fn shadow_field(&self, field: &str) -> Option<&RuntimeValue> {
        if self.mirrors.iter().any(|m| m == field) {
            self.shadow.get(field)
        } else {
            None
        }
    }

    pub fn replay_field(&self, index: usize, field: &str) -> Option<&RuntimeValue> {
        if !self.replay || !self.mirrors.iter().any(|m| m == field) {
            return None;
        }
        self.replay_buffer.get(index)?.get(field)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::RuntimeValue;

    #[test]
    fn replay_field_returns_historical_snapshot() {
        let mut twin = TwinRuntime::new("T".into(), vec!["pose".into()], true);
        twin.snapshot(
            "pose",
            RuntimeValue::Pose {
                x: 1.0,
                y: 2.0,
                theta: 0.0,
                z: 0.0,
            },
        );
        twin.commit_frame();
        twin.snapshot(
            "pose",
            RuntimeValue::Pose {
                x: 3.0,
                y: 4.0,
                theta: 0.0,
                z: 0.0,
            },
        );
        twin.commit_frame();
        assert_eq!(twin.replay_frame_count(), 2);
        let first = twin.replay_field(0, "pose").unwrap();
        assert_eq!(
            first,
            &RuntimeValue::Pose {
                x: 1.0,
                y: 2.0,
                theta: 0.0,
                z: 0.0,
            }
        );
    }
}
