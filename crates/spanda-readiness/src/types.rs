//! Core readiness types shared across evaluation modules.

use serde::{Deserialize, Serialize};

/// Overall readiness gate status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ReadinessStatus {
    Ready,
    Degraded,
    NotReady,
    Unknown,
}

/// Severity for readiness issues and audit findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ReadinessSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Weighted scoring policy for readiness factors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadinessPolicy {
    pub minimum_score: u32,
    pub weights: ReadinessWeights,
}

impl Default for ReadinessPolicy {
    fn default() -> Self {
        Self {
            minimum_score: 80,
            weights: ReadinessWeights::default(),
        }
    }
}

/// Per-factor weights (must sum to 100 for normalized scoring).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadinessWeights {
    pub hardware: u32,
    pub capabilities: u32,
    pub health: u32,
    pub connectivity: u32,
    pub safety: u32,
    pub battery: u32,
    pub storage: u32,
    pub compute: u32,
    pub packages: u32,
    pub providers: u32,
    pub mission: u32,
}

impl Default for ReadinessWeights {
    fn default() -> Self {
        Self {
            hardware: 12,
            capabilities: 12,
            health: 12,
            connectivity: 8,
            safety: 12,
            battery: 8,
            storage: 4,
            compute: 6,
            packages: 8,
            providers: 8,
            mission: 10,
        }
    }
}

/// Numeric readiness score with factor breakdown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadinessScore {
    pub total: u32,
    pub maximum: u32,
    pub factors: Vec<ReadinessFactorScore>,
}

/// Score for a single readiness factor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadinessFactorScore {
    pub factor: String,
    pub score: u32,
    pub weight: u32,
    pub weighted: f64,
}

/// Single readiness issue affecting score or mission go/no-go.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadinessIssue {
    pub factor: String,
    pub severity: ReadinessSeverity,
    pub message: String,
    pub suggested_action: Option<String>,
}

/// Full readiness evaluation report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadinessReport {
    pub status: ReadinessStatus,
    pub mission_ready: bool,
    pub score: ReadinessScore,
    pub issues: Vec<ReadinessIssue>,
    pub policy: ReadinessPolicy,
    pub target: Option<String>,
    pub robots: Vec<String>,
}

/// Output format for CLI reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReportFormat {
    #[default]
    Text,
    Json,
    Markdown,
    Html,
}

/// Options for readiness evaluation.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReadinessOptions {
    pub target: Option<String>,
    pub policy: Option<ReadinessPolicy>,
    pub simulate: bool,
    pub strict: bool,
}

/// Twin readiness comparison status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwinReadinessStatus {
    pub physical_ready: bool,
    pub twin_ready: bool,
    pub configuration_drift: Vec<String>,
    pub capability_drift: Vec<String>,
    pub health_drift: Vec<String>,
    pub overall: ReadinessStatus,
}

/// Fleet-level readiness summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetReadinessReport {
    pub fleet_score: u32,
    pub healthy_robots: u32,
    pub degraded_robots: u32,
    pub not_ready_robots: u32,
    pub mission_capacity_percent: u32,
    pub robot_reports: Vec<ReadinessReport>,
    pub issues: Vec<ReadinessIssue>,
}
