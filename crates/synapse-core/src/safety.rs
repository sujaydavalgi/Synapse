use crate::error::RobotState;
use crate::runtime::Environment;

#[derive(Debug, Clone, PartialEq)]
pub struct SafetyZoneRuntime {
    pub name: String,
    pub shape: SafetyZoneShape,
    pub x: f64,
    pub y: f64,
    pub radius: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyZoneShape {
    Circle,
    Rect,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SafetyEvaluation {
    pub allowed: bool,
    pub reason: Option<String>,
    pub emergency_stop: bool,
}

pub type StopIfRule = Box<dyn Fn(&Environment) -> bool>;

pub struct SafetyConfig {
    pub max_speed: f64,
    pub stop_if_rules: Vec<StopIfRule>,
    pub zones: Vec<SafetyZoneRuntime>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedMotion {
    pub linear: f64,
    pub angular: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidateActionResult {
    Ok(ValidatedMotion),
    Err { reason: String },
}

pub struct SafetyMonitor {
    config: SafetyConfig,
    emergency_stop: bool,
}

impl SafetyMonitor {
    pub fn new(config: SafetyConfig) -> Self {
        Self {
            config,
            emergency_stop: false,
        }
    }

    pub fn evaluate_before_motion(
        &mut self,
        env: &Environment,
        pose: &Pose2d,
    ) -> SafetyEvaluation {
        let peek = self.peek_before_motion(env, pose);
        if !peek.allowed && peek.emergency_stop {
            self.emergency_stop = true;
        }
        peek
    }

    pub fn peek_before_motion(&self, env: &Environment, pose: &Pose2d) -> SafetyEvaluation {
        if self.emergency_stop {
            return SafetyEvaluation {
                allowed: false,
                reason: Some("Emergency stop active".to_string()),
                emergency_stop: true,
            };
        }

        for rule in &self.config.stop_if_rules {
            if rule(env) {
                return SafetyEvaluation {
                    allowed: false,
                    reason: Some("stop_if safety rule triggered".to_string()),
                    emergency_stop: true,
                };
            }
        }

        for zone in &self.config.zones {
            if Self::is_point_in_zone(pose.x, pose.y, zone) {
                return SafetyEvaluation {
                    allowed: false,
                    reason: Some(format!("Robot entered safety zone '{}'", zone.name)),
                    emergency_stop: true,
                };
            }
        }

        SafetyEvaluation {
            allowed: true,
            reason: None,
            emergency_stop: false,
        }
    }

    pub fn validate_action_proposal(
        &self,
        linear: f64,
        angular: f64,
        env: &Environment,
        pose: &Pose2d,
    ) -> ValidateActionResult {
        let peek = self.peek_before_motion(env, pose);
        if !peek.allowed {
            return ValidateActionResult::Err {
                reason: peek
                    .reason
                    .unwrap_or_else(|| "Safety validation failed".to_string()),
            };
        }
        ValidateActionResult::Ok(ValidatedMotion {
            linear: self.clamp_speed(linear),
            angular,
        })
    }

    pub fn is_in_zone(&self, zone_name: &str, pose: &Pose2d) -> bool {
        let Some(zone) = self.config.zones.iter().find(|z| z.name == zone_name) else {
            return false;
        };
        Self::is_point_in_zone(pose.x, pose.y, zone)
    }

    pub fn clamp_speed(&self, requested: f64) -> f64 {
        let sign = if requested == 0.0 { 1.0 } else { requested.signum() };
        requested.abs().min(self.config.max_speed) * sign
    }

    pub fn is_emergency_stop(&self) -> bool {
        self.emergency_stop
    }

    pub fn set_emergency_stop(&mut self, active: bool) {
        self.emergency_stop = active;
    }

    pub fn reset(&mut self) {
        self.emergency_stop = false;
    }

    fn is_point_in_zone(x: f64, y: f64, zone: &SafetyZoneRuntime) -> bool {
        match zone.shape {
            SafetyZoneShape::Circle => {
                if let Some(radius) = zone.radius {
                    let dx = x - zone.x;
                    let dy = y - zone.y;
                    (dx * dx + dy * dy).sqrt() <= radius
                } else {
                    false
                }
            }
            SafetyZoneShape::Rect => {
                if let (Some(width), Some(height)) = (zone.width, zone.height) {
                    x >= zone.x && x <= zone.x + width && y >= zone.y && y <= zone.y + height
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose2d {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pose3d {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
    pub z: f64,
}

pub fn create_safety_config_from_robot(
    max_speed: f64,
    stop_if_rules: Vec<StopIfRule>,
    zones: Vec<SafetyZoneRuntime>,
) -> SafetyConfig {
    SafetyConfig {
        max_speed,
        stop_if_rules,
        zones,
    }
}

pub fn apply_emergency_stop(state: RobotState) -> RobotState {
    RobotState {
        emergency_stop: true,
        velocity: crate::error::VelocityState {
            linear: 0.0,
            angular: 0.0,
        },
        ..state
    }
}

pub fn interpolate_poses(
    from: &crate::error::PoseState,
    to: &crate::error::PoseState,
    steps: f64,
) -> Vec<Pose3d> {
    let count = steps.max(2.0).floor() as usize;
    let from_z = from.z.unwrap_or(0.0);
    let to_z = to.z.unwrap_or(0.0);
    let mut waypoints = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f64 / (count as f64 - 1.0);
        waypoints.push(Pose3d {
            x: from.x + (to.x - from.x) * t,
            y: from.y + (to.y - from.y) * t,
            theta: from.theta + (to.theta - from.theta) * t,
            z: from_z + (to_z - from_z) * t,
        });
    }
    waypoints
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::UnitKind;
    use crate::runtime::RuntimeValue;

    #[test]
    fn blocks_motion_when_stop_if_triggers() {
        let mut env = Environment::new();
        env.define(
            "obstacle",
            RuntimeValue::number(0.3, UnitKind::M),
        );

        let mut monitor = SafetyMonitor::new(create_safety_config_from_robot(
            1.5,
            vec![Box::new(|e| {
                matches!(
                    e.get("obstacle"),
                    Some(RuntimeValue::Number { value, .. }) if *value < 0.5
                )
            })],
            vec![],
        ));

        let result = monitor.evaluate_before_motion(&env, &Pose2d { x: 0.0, y: 0.0 });
        assert!(!result.allowed);
        assert!(result.emergency_stop);
    }

    #[test]
    fn allows_motion_when_rules_pass() {
        let mut env = Environment::new();
        env.define(
            "obstacle",
            RuntimeValue::number(2.0, UnitKind::M),
        );

        let mut monitor = SafetyMonitor::new(create_safety_config_from_robot(
            1.5,
            vec![Box::new(|e| {
                matches!(
                    e.get("obstacle"),
                    Some(RuntimeValue::Number { value, .. }) if *value < 0.5
                )
            })],
            vec![],
        ));

        let result = monitor.evaluate_before_motion(&env, &Pose2d { x: 0.0, y: 0.0 });
        assert!(result.allowed);
    }

    #[test]
    fn detects_safety_zone_entry() {
        let monitor = SafetyMonitor::new(create_safety_config_from_robot(
            1.5,
            vec![],
            vec![SafetyZoneRuntime {
                name: "keepout".to_string(),
                shape: SafetyZoneShape::Circle,
                x: 0.0,
                y: 0.0,
                radius: Some(1.0),
                width: None,
                height: None,
            }],
        ));
        assert!(monitor.is_in_zone("keepout", &Pose2d { x: 0.5, y: 0.0 }));
        assert!(!monitor.is_in_zone("keepout", &Pose2d { x: 5.0, y: 5.0 }));
    }

    #[test]
    fn clamps_speed_to_max() {
        let monitor = SafetyMonitor::new(create_safety_config_from_robot(1.0, vec![], vec![]));
        assert_eq!(monitor.clamp_speed(2.0), 1.0);
        assert_eq!(monitor.clamp_speed(-3.0), -1.0);
    }
}
