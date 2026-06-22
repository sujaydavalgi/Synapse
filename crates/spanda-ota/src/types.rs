//! Shared OTA deploy types for plans, rollouts, and certification summaries.
//!
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rollout strategy for OTA updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RolloutStrategy {
    All,
    Canary,
    Staged,
}

/// A single robot-to-hardware deployment assignment from the program AST.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeployAssignment {
    pub robot_name: String,
    pub hardware: String,
}

/// Deployment plan extracted from a Spanda program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeployPlan {
    pub program: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program_hash: Option<String>,
    pub assignments: Vec<DeployAssignment>,
    pub certifications: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certification_proof: Option<CertificationProofSummary>,
}

/// Structured certification proof summary embedded in deploy plans.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificationProofSummary {
    pub passed: bool,
    pub passed_strict: bool,
    pub summary: String,
    pub error_count: u32,
    pub warning_count: u32,
}

/// Status of one rollout step on a target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RolloutStepStatus {
    Pending,
    Deployed,
    RolledBack,
    Skipped,
    Failed,
}

/// One step in an OTA rollout or rollback.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RolloutStep {
    pub robot_name: String,
    pub hardware: String,
    pub status: RolloutStepStatus,
    pub version: String,
    pub phase_percent: Option<u8>,
}

/// Result of planning or executing a rollout/rollback.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RolloutResult {
    pub strategy: RolloutStrategy,
    pub version: String,
    pub dry_run: bool,
    pub steps: Vec<RolloutStep>,
    pub success: bool,
}

/// Persistent OTA state for rollback and audit.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeployState {
    pub current_version: HashMap<String, String>,
    pub previous_version: HashMap<String, String>,
    pub history: Vec<RolloutResult>,
}

/// Options controlling rollout behavior.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RolloutOptions {
    pub strategy: RolloutStrategy,
    pub canary_percent: u8,
    pub staged_phases: Vec<u8>,
    pub version: String,
    pub dry_run: bool,
    #[serde(default)]
    pub require_certify: bool,
}

impl Default for RolloutOptions {
    fn default() -> Self {
        Self {
            strategy: RolloutStrategy::All,
            canary_percent: 10,
            staged_phases: vec![10, 50, 100],
            version: "1.0.0".into(),
            dry_run: false,
            require_certify: false,
        }
    }
}
