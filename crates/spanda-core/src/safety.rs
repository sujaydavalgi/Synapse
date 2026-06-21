//! safety support for Spanda.
//!
use crate::error::RobotState;
use crate::runtime::Environment;
use std::collections::HashMap;

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
    pub zone_speed_caps: HashMap<String, f64>,
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
        // Create a new instance.
        //
        // Parameters:
        // - `config` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_core::safety::new(config);

        // Assemble the struct fields and return it.
        Self {
            config,
            emergency_stop: false,
        }
    }

    pub fn evaluate_before_motion(&mut self, env: &Environment, pose: &Pose2d) -> SafetyEvaluation {
        // Evaluate before motion.
        //
        // Parameters:
        // - `self` — method receiver
        // - `env` — input value
        // - `pose` — input value
        //
        // Returns:
        // SafetyEvaluation.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.evaluate_before_motion(env, pose);

        // Compute peek for the following logic.
        let peek = self.peek_before_motion(env, pose);

        // Take the branch when emergency stop is false.
        if !peek.allowed && peek.emergency_stop {
            self.emergency_stop = true;
        }
        peek
    }

    pub fn peek_before_motion(&self, env: &Environment, pose: &Pose2d) -> SafetyEvaluation {
        // Peek before motion.
        //
        // Parameters:
        // - `self` — method receiver
        // - `env` — input value
        // - `pose` — input value
        //
        // Returns:
        // SafetyEvaluation.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.peek_before_motion(env, pose);

        // take this path when self.emergency stop.
        if self.emergency_stop {
            return SafetyEvaluation {
                allowed: false,
                reason: Some("Emergency stop active".to_string()),
                emergency_stop: true,
            };
        }

        // Process each stop if rule.
        for rule in &self.config.stop_if_rules {
            // Take this path when rule(env).
            if rule(env) {
                return SafetyEvaluation {
                    allowed: false,
                    reason: Some("stop_if safety rule triggered".to_string()),
                    emergency_stop: true,
                };
            }
        }

        // Process each zone.
        for zone in &self.config.zones {
            // Take this path when Self::is point in zone(pose.x, pose.y, zone).
            if Self::is_point_in_zone(pose.x, pose.y, zone) {
                // Allow motion inside zones that only declare a program speed cap.
                if self.config.zone_speed_caps.contains_key(&zone.name) {
                    continue;
                }
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
        // Validate action proposal.
        //
        // Parameters:
        // - `self` — method receiver
        // - `linear` — input value
        // - `angular` — input value
        // - `env` — input value
        // - `pose` — input value
        //
        // Returns:
        // ValidateActionResult.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.validate_action_proposal(linear, angular, env, pose);

        // Compute peek for the following logic.
        let peek = self.peek_before_motion(env, pose);

        // Take the branch when allowed is false.
        if !peek.allowed {
            return ValidateActionResult::Err {
                reason: peek
                    .reason
                    .unwrap_or_else(|| "Safety validation failed".to_string()),
            };
        }
        ValidateActionResult::Ok(ValidatedMotion {
            linear: self.clamp_speed_at_pose(linear, pose),
            angular,
        })
    }

    pub fn effective_max_speed(&self, pose: &Pose2d) -> f64 {
        // Compute the active speed cap from global max and program zone policies.
        let mut cap = self.config.max_speed;
        for zone in &self.config.zones {
            if Self::is_point_in_zone(pose.x, pose.y, zone) {
                if let Some(zone_cap) = self.config.zone_speed_caps.get(&zone.name) {
                    cap = cap.min(*zone_cap);
                }
            }
        }
        cap
    }

    pub fn clamp_speed_at_pose(&self, requested: f64, pose: &Pose2d) -> f64 {
        // Clamp requested linear speed to the effective cap at the current pose.
        let sign = if requested == 0.0 {
            1.0
        } else {
            requested.signum()
        };
        requested.abs().min(self.effective_max_speed(pose)) * sign
    }

    pub fn is_in_zone(&self, zone_name: &str, pose: &Pose2d) -> bool {
        //
        // Parameters:
        // - `self` — method receiver
        // - `zone_name` — input value
        // - `pose` — input value
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.is_in_zone(zone_name, pose);

        // Compute Some for the following logic.
        let Some(zone) = self.config.zones.iter().find(|z| z.name == zone_name) else {
            return false;
        };
        Self::is_point_in_zone(pose.x, pose.y, zone)
    }

    pub fn clamp_speed(&self, requested: f64) -> f64 {
        // Clamp speed using the global max when pose is unavailable.
        self.clamp_speed_at_pose(requested, &Pose2d { x: 0.0, y: 0.0 })
    }

    pub fn is_emergency_stop(&self) -> bool {
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.is_emergency_stop();

        // Call emergency stop on the current instance.
        self.emergency_stop
    }

    pub fn set_emergency_stop(&mut self, active: bool) {
        // Set emergency stop.
        //
        // Parameters:
        // - `self` — method receiver
        // - `active` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.set_emergency_stop(active);

        // Call emergency stop = active; on the current instance.
        self.emergency_stop = active;
    }

    pub fn reset(&mut self) {
        // Reset the value.
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
        // let result = instance.reset();

        // Call emergency stop = false; on the current instance.
        self.emergency_stop = false;
    }

    fn is_point_in_zone(x: f64, y: f64, zone: &SafetyZoneRuntime) -> bool {
        //
        // Parameters:
        // - `x` — input value
        // - `y` — input value
        // - `zone` — input value
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::safety::is_point_in_zone(x, y, zone);

        // Match on shape and handle each case.
        match zone.shape {
            SafetyZoneShape::Circle => {
                // Emit output when radius provides a radius.
                if let Some(radius) = zone.radius {
                    let dx = x - zone.x;
                    let dy = y - zone.y;
                    (dx * dx + dy * dy).sqrt() <= radius
                } else {
                    false
                }
            }
            SafetyZoneShape::Rect => {
                // Take this path when let (Some(width), Some(height)) = (zone.width, zone.height).
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
    zone_speed_caps: HashMap<String, f64>,
) -> SafetyConfig {
    // Create safety config from robot.
    //
    // Parameters:
    // - `max_speed` — input value
    // - `stop_if_rules` — input value
    // - `zones` — input value
    // - `zone_speed_caps` — program-level caps keyed by zone name
    //
    // Returns:
    // SafetyConfig.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::safety::create_safety_config_from_robot(max_speed, stop_if_rules, zones, zone_speed_caps);

    // Produce SafetyConfig as the result.
    SafetyConfig {
        max_speed,
        stop_if_rules,
        zones,
        zone_speed_caps,
    }
}

pub fn apply_emergency_stop(state: RobotState) -> RobotState {
    // Apply emergency stop.
    //
    // Parameters:
    // - `state` — input value
    //
    // Returns:
    // RobotState.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::safety::apply_emergency_stop(state);

    // Produce RobotState as the result.
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
    // Interpolate poses.
    //
    // Parameters:
    // - `from` — input value
    // - `to` — input value
    // - `steps` — input value
    //
    // Returns:
    // Vec<Pose3d>.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::safety::interpolate_poses(from, to, steps);

    // Compute count for the following logic.
    let count = steps.max(2.0).floor() as usize;
    let from_z = from.z.unwrap_or(0.0);
    let to_z = to.z.unwrap_or(0.0);
    let mut waypoints = Vec::with_capacity(count);

    // Iterate over count.
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
        // Blocks motion when stop if triggers.
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
        // let result = spanda_core::safety::blocks_motion_when_stop_if_triggers();

        let mut env = Environment::new();
        env.define("obstacle", RuntimeValue::number(0.3, UnitKind::M));

        let mut monitor = SafetyMonitor::new(create_safety_config_from_robot(
            1.5,
            vec![Box::new(|e| {
                matches!(
                    e.get("obstacle"),
                    Some(RuntimeValue::Number { value, .. }) if *value < 0.5
                )
            })],
            vec![],
            HashMap::new(),
        ));

        let result = monitor.evaluate_before_motion(&env, &Pose2d { x: 0.0, y: 0.0 });
        assert!(!result.allowed);
        assert!(result.emergency_stop);
    }

    #[test]
    fn allows_motion_when_rules_pass() {
        // Allows motion when rules pass.
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
        // let result = spanda_core::safety::allows_motion_when_rules_pass();

        let mut env = Environment::new();
        env.define("obstacle", RuntimeValue::number(2.0, UnitKind::M));

        let mut monitor = SafetyMonitor::new(create_safety_config_from_robot(
            1.5,
            vec![Box::new(|e| {
                matches!(
                    e.get("obstacle"),
                    Some(RuntimeValue::Number { value, .. }) if *value < 0.5
                )
            })],
            vec![],
            HashMap::new(),
        ));

        let result = monitor.evaluate_before_motion(&env, &Pose2d { x: 0.0, y: 0.0 });
        assert!(result.allowed);
    }

    #[test]
    fn detects_safety_zone_entry() {
        // Detects safety zone entry.
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
        // let result = spanda_core::safety::detects_safety_zone_entry();

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
            HashMap::new(),
        ));
        assert!(monitor.is_in_zone("keepout", &Pose2d { x: 0.5, y: 0.0 }));
        assert!(!monitor.is_in_zone("keepout", &Pose2d { x: 5.0, y: 5.0 }));
    }

    #[test]
    fn clamps_speed_to_max() {
        // Clamps speed to max.
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
        // let result = spanda_core::safety::clamps_speed_to_max();

        let monitor = SafetyMonitor::new(create_safety_config_from_robot(
            1.0,
            vec![],
            vec![],
            HashMap::new(),
        ));
        assert_eq!(monitor.clamp_speed(2.0), 1.0);
        assert_eq!(monitor.clamp_speed(-3.0), -1.0);
    }

    #[test]
    fn clamps_speed_to_program_zone_cap() {
        // Clamps speed to program zone cap when robot is inside the zone.
        let mut caps = HashMap::new();
        caps.insert("HumanArea".into(), 0.5);
        let monitor = SafetyMonitor::new(create_safety_config_from_robot(
            1.0,
            vec![],
            vec![SafetyZoneRuntime {
                name: "HumanArea".into(),
                shape: SafetyZoneShape::Circle,
                x: 0.0,
                y: 0.0,
                radius: Some(2.0),
                width: None,
                height: None,
            }],
            caps,
        ));
        let inside = Pose2d { x: 0.0, y: 0.0 };
        let outside = Pose2d { x: 10.0, y: 10.0 };
        assert_eq!(monitor.clamp_speed_at_pose(0.8, &inside), 0.5);
        assert_eq!(monitor.clamp_speed_at_pose(0.8, &outside), 0.8);
    }

    #[test]
    fn allows_motion_inside_speed_cap_zone() {
        // Speed-cap zones permit motion; clamping applies instead of hard stop.
        let mut caps = HashMap::new();
        caps.insert("HumanArea".into(), 0.5);
        let mut monitor = SafetyMonitor::new(create_safety_config_from_robot(
            1.0,
            vec![],
            vec![SafetyZoneRuntime {
                name: "HumanArea".into(),
                shape: SafetyZoneShape::Circle,
                x: 0.0,
                y: 0.0,
                radius: Some(2.0),
                width: None,
                height: None,
            }],
            caps,
        ));
        let inside = Pose2d { x: 0.0, y: 0.0 };
        let result = monitor.evaluate_before_motion(&Environment::new(), &inside);
        assert!(result.allowed);
    }
}
