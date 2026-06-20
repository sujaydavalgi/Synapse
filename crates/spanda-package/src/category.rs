//! category support for Spanda.
//!
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Community package categories for registry metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackageCategory {
    Ai,
    Robotics,
    Vision,
    Navigation,
    Manipulation,
    Simulation,
    Ros2,
    Mqtt,
    Hardware,
    Sensors,
    Actuators,
    DigitalTwin,
    Safety,
    Hri,
    Testing,
    Provenance,
    Identity,
    SupplyChain,
    Ledger,
}

impl PackageCategory {
    pub fn all() -> &'static [PackageCategory] {
        // All.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // &'static [PackageCategory].
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_package::category::all();

        // Return the static list of known values.
        &[
            Self::Ai,
            Self::Robotics,
            Self::Vision,
            Self::Navigation,
            Self::Manipulation,
            Self::Simulation,
            Self::Ros2,
            Self::Mqtt,
            Self::Hardware,
            Self::Sensors,
            Self::Actuators,
            Self::DigitalTwin,
            Self::Safety,
            Self::Hri,
            Self::Testing,
            Self::Provenance,
            Self::Identity,
            Self::SupplyChain,
            Self::Ledger,
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
            Self::Ai => "ai",
            Self::Robotics => "robotics",
            Self::Vision => "vision",
            Self::Navigation => "navigation",
            Self::Manipulation => "manipulation",
            Self::Simulation => "simulation",
            Self::Ros2 => "ros2",
            Self::Mqtt => "mqtt",
            Self::Hardware => "hardware",
            Self::Sensors => "sensors",
            Self::Actuators => "actuators",
            Self::DigitalTwin => "digital-twin",
            Self::Safety => "safety",
            Self::Hri => "hri",
            Self::Testing => "testing",
            Self::Provenance => "provenance",
            Self::Identity => "identity",
            Self::SupplyChain => "supply-chain",
            Self::Ledger => "ledger",
        }
    }
}

impl FromStr for PackageCategory {
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
        // let result = spanda_package::category::from_str(s);

        // Match on s and handle each case.
        match s {
            "ai" => Ok(Self::Ai),
            "robotics" => Ok(Self::Robotics),
            "vision" => Ok(Self::Vision),
            "navigation" => Ok(Self::Navigation),
            "manipulation" => Ok(Self::Manipulation),
            "simulation" => Ok(Self::Simulation),
            "ros2" => Ok(Self::Ros2),
            "mqtt" => Ok(Self::Mqtt),
            "hardware" => Ok(Self::Hardware),
            "sensors" => Ok(Self::Sensors),
            "actuators" => Ok(Self::Actuators),
            "digital-twin" => Ok(Self::DigitalTwin),
            "safety" => Ok(Self::Safety),
            "hri" => Ok(Self::Hri),
            "testing" => Ok(Self::Testing),
            "provenance" => Ok(Self::Provenance),
            "identity" => Ok(Self::Identity),
            "supply-chain" => Ok(Self::SupplyChain),
            "ledger" => Ok(Self::Ledger),
            other => Err(format!("unknown package category '{other}'")),
        }
    }
}

impl std::fmt::Display for PackageCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Fmt.
        //
        // Parameters:
        // - `self` — method receiver
        // - `f` — input value
        //
        // Returns:
        // std::fmt::Result.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.fmt(f);

        f.write_str(self.as_str())
    }
}
