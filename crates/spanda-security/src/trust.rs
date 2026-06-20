//! trust support for Spanda.
//!
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::str::FromStr;

/// Runtime trust tier for devices, packages, and communication endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    #[default]
    Untrusted,
    Restricted,
    Trusted,
    Certified,
}

impl TrustLevel {
    pub fn all() -> &'static [TrustLevel] {
        // All.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // &'static [TrustLevel].
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_security::trust::all();

        // Return the static list of known values.
        &[
            Self::Untrusted,
            Self::Restricted,
            Self::Trusted,
            Self::Certified,
        ]
    }

    pub fn rank(self) -> u8 {
        // Rank.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // u8.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.rank();

        // Dispatch based on the enum variant or current state.
        match self {
            Self::Untrusted => 0,
            Self::Restricted => 1,
            Self::Trusted => 2,
            Self::Certified => 3,
        }
    }

    pub fn as_str(self) -> &'static str {
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
            Self::Untrusted => "untrusted",
            Self::Restricted => "restricted",
            Self::Trusted => "trusted",
            Self::Certified => "certified",
        }
    }

    pub fn satisfies(self, required: TrustLevel) -> bool {
        // Satisfies.
        //
        // Parameters:
        // - `self` — method receiver
        // - `required` — input value
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.satisfies(required);

        // Call rank on the current instance.
        self.rank().cmp(&required.rank()) != Ordering::Less
    }
}

impl FromStr for TrustLevel {
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
        // let result = spanda_security::trust::from_str(s);

        // Match on s and handle each case.
        match s {
            "untrusted" => Ok(Self::Untrusted),
            "restricted" => Ok(Self::Restricted),
            "trusted" => Ok(Self::Trusted),
            "certified" => Ok(Self::Certified),
            other => Err(format!("unknown trust level '{other}'")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trust_ordering() {
        // Trust ordering.
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
        // let result = spanda_security::trust::trust_ordering();

        assert!(TrustLevel::Certified.satisfies(TrustLevel::Trusted));
        assert!(!TrustLevel::Restricted.satisfies(TrustLevel::Trusted));
    }
}
