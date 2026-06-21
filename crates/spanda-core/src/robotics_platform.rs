//! Robotics platform primitives: mission lifecycle, fleet grouping, and navigation helpers.
//!
//! Core language constructs (`mission`, `fleet`, `safety_zone`) are parsed into AST nodes in
//! [`crate::foundations`]. This module holds shared runtime state and validation helpers.

use crate::ast::Span;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mission lifecycle states tracked at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum MissionState {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
}

impl MissionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Running => "Running",
            Self::Paused => "Paused",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
        }
    }
}

/// Runtime mission controller for named step sequences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionRuntime {
    pub name: Option<String>,
    pub steps: Vec<String>,
    pub state: MissionState,
    pub step_index: usize,
    pub duration_hours: Option<f64>,
}

impl MissionRuntime {
    pub fn new(name: Option<String>, steps: Vec<String>, duration_hours: Option<f64>) -> Self {
        // Build a mission controller starting in the pending state.
        Self {
            name,
            steps,
            state: MissionState::Pending,
            step_index: 0,
            duration_hours,
        }
    }

    pub fn start(&mut self) {
        // Transition a pending mission into the running state.
        if self.state == MissionState::Pending {
            self.state = MissionState::Running;
        }
    }

    pub fn pause(&mut self) {
        // Pause an active mission without losing step progress.
        if self.state == MissionState::Running {
            self.state = MissionState::Paused;
        }
    }

    pub fn resume(&mut self) {
        // Resume a paused mission from the current step.
        if self.state == MissionState::Paused {
            self.state = MissionState::Running;
        }
    }

    pub fn advance(&mut self) -> Option<String> {
        // Move to the next mission step and return its name when one remains.
        if self.state != MissionState::Running {
            return None;
        }
        if self.step_index >= self.steps.len() {
            self.state = MissionState::Completed;
            return None;
        }
        let step = self.steps[self.step_index].clone();
        self.step_index += 1;
        if self.step_index >= self.steps.len() {
            self.state = MissionState::Completed;
        }
        Some(step)
    }

    pub fn complete(&mut self) {
        // Mark the mission completed regardless of remaining steps.
        self.state = MissionState::Completed;
        self.step_index = self.steps.len();
    }

    pub fn fail(&mut self) {
        // Mark the mission failed and stop step progression.
        self.state = MissionState::Failed;
    }

    pub fn current_step(&self) -> Option<&str> {
        // Return the active step name while the mission is running.
        if self.state != MissionState::Running {
            return None;
        }
        self.steps.get(self.step_index).map(String::as_str)
    }
}

/// Known safety certification standards referenced by program metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CertificationStandard {
    Iso13849,
    Iec61508,
    Iso26262,
}

impl CertificationStandard {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Iso13849 => "ISO13849",
            Self::Iec61508 => "IEC61508",
            Self::Iso26262 => "ISO26262",
        }
    }

    pub fn parse_ident(name: &str) -> Option<Self> {
        match name {
            "ISO13849" => Some(Self::Iso13849),
            "IEC61508" => Some(Self::Iec61508),
            "ISO26262" => Some(Self::Iso26262),
            _ => None,
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Iso13849, Self::Iec61508, Self::Iso26262]
    }
}

/// Program-level certification metadata (`certify ISO13849;` or block with level).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum CertifyDecl {
    CertifyDecl {
        standard: CertificationStandard,
        level: Option<String>,
        span: Span,
    },
}

/// Validate certification standard identifiers at parse/type-check time.
pub fn validate_certification_standard(name: &str) -> Option<String> {
    if CertificationStandard::parse_ident(name).is_some() {
        return None;
    }
    Some(format!(
        "unknown certification standard '{name}' (expected ISO13849, IEC61508, or ISO26262)"
    ))
}

/// Program-level fleet grouping of robot names.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum FleetDecl {
    FleetDecl {
        name: String,
        members: Vec<String>,
        span: Span,
    },
}

/// Program-level safety zone policy with optional speed cap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ProgramSafetyZoneDecl {
    ProgramSafetyZoneDecl {
        name: String,
        max_speed_mps: Option<f64>,
        span: Span,
    },
}

/// Registry of fleet groups declared at program scope.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FleetRegistry {
    fleets: HashMap<String, Vec<String>>,
}

impl FleetRegistry {
    pub fn register(&mut self, name: &str, members: Vec<String>) {
        // Store a fleet name and its member robot identifiers.
        self.fleets.insert(name.to_string(), members);
    }

    pub fn members(&self, name: &str) -> Option<&[String]> {
        // Look up fleet members by fleet name.
        self.fleets.get(name).map(Vec::as_slice)
    }

    pub fn names(&self) -> impl Iterator<Item = &String> {
        // Iterate declared fleet names.
        self.fleets.keys()
    }
}

/// Program-level safety zone speed policies keyed by zone name.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProgramSafetyZoneRegistry {
    zones: HashMap<String, f64>,
}

impl ProgramSafetyZoneRegistry {
    pub fn register(&mut self, name: &str, max_speed_mps: f64) {
        // Register a zone-specific maximum speed in meters per second.
        self.zones.insert(name.to_string(), max_speed_mps);
    }

    pub fn max_speed_for(&self, zone_name: &str) -> Option<f64> {
        // Resolve the configured speed cap for a named zone.
        self.zones.get(zone_name).copied()
    }

    pub fn speed_caps(&self) -> &HashMap<String, f64> {
        // Return all registered zone speed caps.
        &self.zones
    }
}

/// Validate fleet member names against declared robots.
pub fn validate_fleet_members(
    fleet_name: &str,
    members: &[String],
    robot_names: &[String],
) -> Option<String> {
    // Report the first fleet member that does not match a declared robot.
    for member in members {
        if !robot_names.iter().any(|r| r == member) {
            return Some(format!(
                "fleet '{fleet_name}' references unknown robot '{member}'"
            ));
        }
    }
    None
}

/// Validate mission declarations have either duration or steps.
pub fn validate_mission_decl(
    name: &Option<String>,
    duration_hours: Option<f64>,
    steps: &[String],
) -> Option<String> {
    // Require at least one of duration budgeting or executable steps.
    if duration_hours.is_none() && steps.is_empty() {
        let label = name
            .as_deref()
            .map(|n| format!("mission '{n}'"))
            .unwrap_or_else(|| "mission".into());
        return Some(format!(
            "{label} requires at least one of duration or mission steps"
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mission_advances_through_steps() {
        // Mission advances through steps.
        let mut mission = MissionRuntime::new(
            Some("Delivery".into()),
            vec!["navigate".into(), "deliver".into()],
            Some(0.5),
        );
        mission.start();
        assert_eq!(mission.advance(), Some("navigate".into()));
        assert_eq!(mission.advance(), Some("deliver".into()));
        assert_eq!(mission.state, MissionState::Completed);
    }

    #[test]
    fn fleet_registry_resolves_members() {
        // Fleet registry resolves members.
        let mut registry = FleetRegistry::default();
        registry.register("Warehouse", vec!["Picker1".into(), "Picker2".into()]);
        assert_eq!(
            registry.members("Warehouse"),
            Some(["Picker1".into(), "Picker2".into()].as_slice())
        );
    }
}
