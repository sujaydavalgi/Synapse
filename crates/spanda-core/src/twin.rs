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

    /// Compare previous shadow against live mirrored values; true when divergence exceeds threshold.
    pub fn detect_divergence(&self, live: &HashMap<String, RuntimeValue>, threshold: f64) -> bool {
        for field in &self.mirrors {
            let Some(shadow_val) = self.shadow.get(field) else {
                continue;
            };
            let Some(live_val) = live.get(field) else {
                continue;
            };
            if value_distance(shadow_val, live_val) > threshold {
                return true;
            }
        }
        false
    }

    pub fn live_mirrored_fields(
        pose: (f64, f64, f64, f64),
        velocity: (f64, f64),
        mirrors: &[String],
    ) -> HashMap<String, RuntimeValue> {
        let mut live = HashMap::new();
        if mirrors.iter().any(|m| m == "pose") {
            live.insert(
                "pose".into(),
                RuntimeValue::Pose {
                    x: pose.0,
                    y: pose.1,
                    theta: pose.2,
                    z: pose.3,
                },
            );
        }
        if mirrors.iter().any(|m| m == "velocity") {
            live.insert(
                "velocity".into(),
                RuntimeValue::Velocity {
                    linear: velocity.0,
                    angular: velocity.1,
                },
            );
        }
        live
    }
}

fn value_distance(a: &RuntimeValue, b: &RuntimeValue) -> f64 {
    match (a, b) {
        (
            RuntimeValue::Pose {
                x: x1,
                y: y1,
                theta: _,
                z: z1,
            },
            RuntimeValue::Pose {
                x: x2,
                y: y2,
                theta: _,
                z: z2,
            },
        ) => {
            let dx = x1 - x2;
            let dy = y1 - y2;
            let dz = z1 - z2;
            (dx * dx + dy * dy + dz * dz).sqrt()
        }
        (
            RuntimeValue::Velocity {
                linear: l1,
                angular: a1,
            },
            RuntimeValue::Velocity {
                linear: l2,
                angular: a2,
            },
        ) => {
            let dl = l1 - l2;
            let da = a1 - a2;
            (dl * dl + da * da).sqrt()
        }
        (RuntimeValue::Number { value: v1, .. }, RuntimeValue::Number { value: v2, .. }) => {
            (v1 - v2).abs()
        }
        _ => 0.0,
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
