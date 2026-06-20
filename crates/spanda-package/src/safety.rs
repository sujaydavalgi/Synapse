//! safety support for Spanda.
//!
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Package trust / safety level for deployment gating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SafetyLevel {
    #[default]
    Experimental,
    SimulationOnly,
    HardwareSafe,
    Certified,
}

impl SafetyLevel {
    pub fn all() -> &'static [SafetyLevel] {
        // All.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // &'static [SafetyLevel].
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_package::safety::all();

        // Return the static list of known values.
        &[
            Self::Experimental,
            Self::SimulationOnly,
            Self::HardwareSafe,
            Self::Certified,
        ]
    }

    pub fn as_str(&self) -> &'static str {
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
            Self::Experimental => "experimental",
            Self::SimulationOnly => "simulation_only",
            Self::HardwareSafe => "hardware_safe",
            Self::Certified => "certified",
        }
    }

    /// Whether this level may control physical actuators on real hardware.
    pub fn can_control_actuators_default(&self) -> bool {
        // Can control actuators default.
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
        // let result = instance.can_control_actuators_default();

        // Produce Certified) as the result.
        matches!(self, Self::HardwareSafe | Self::Certified)
    }

    /// Whether packages at this level require manual review before deployment.
    pub fn requires_review_default(&self) -> bool {
        // Requires review default.
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
        // let result = instance.requires_review_default();

        // Produce SimulationOnly) as the result.
        matches!(self, Self::Experimental | Self::SimulationOnly)
    }
}

impl FromStr for SafetyLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Construct from str.
        //
        // Parameters:
        // - `s` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_package::safety::from_str(s);

        // Match on s and handle each case.
        match s {
            "experimental" => Ok(Self::Experimental),
            "simulation_only" => Ok(Self::SimulationOnly),
            "hardware_safe" => Ok(Self::HardwareSafe),
            "certified" => Ok(Self::Certified),
            other => Err(format!("unknown safety level '{other}'")),
        }
    }
}

/// Safety metadata from `[safety]` in spanda.toml.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafetyMetadata {
    #[serde(default)]
    pub level: SafetyLevel,
    #[serde(default = "default_true")]
    pub requires_review: bool,
    #[serde(default)]
    pub can_control_actuators: bool,
}

fn default_true() -> bool {
    // Default true.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_package::safety::default_true();

    // Produce true as the result.
    true
}

impl Default for SafetyMetadata {
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
        // let value = spanda_package::safety::default();

        // Compute level for the following logic.
        let level = SafetyLevel::Experimental;
        Self {
            level,
            requires_review: level.requires_review_default(),
            can_control_actuators: level.can_control_actuators_default(),
        }
    }
}

impl SafetyMetadata {
    pub fn normalize(&mut self) {
        // Normalize the value.
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
        // let result = instance.normalize();

        // take the branch when level equals can control actuators.
        if self.level == SafetyLevel::default() && !self.can_control_actuators {
            self.requires_review = self.level.requires_review_default();
        }
    }
}
