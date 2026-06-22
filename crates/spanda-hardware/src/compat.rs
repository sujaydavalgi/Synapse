//! Hardware compatibility report item types shared by verify pipelines.
//!
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompatSeverity {
    Pass,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompatItem {
    pub category: String,
    pub message: String,
    pub severity: CompatSeverity,
    pub line: u32,
    pub column: u32,
}
