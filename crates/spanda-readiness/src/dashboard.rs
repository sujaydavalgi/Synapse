//! Dashboard data models for future operational UI.

use serde::{Deserialize, Serialize};

use crate::types::{FleetReadinessReport, ReadinessReport};

/// Readiness dashboard aggregate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadinessDashboard {
    pub overall_score: u32,
    pub mission_ready_count: u32,
    pub degraded_count: u32,
    pub not_ready_count: u32,
    pub top_issues: Vec<String>,
    pub reports: Vec<ReadinessReport>,
}

/// Fleet operations dashboard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetDashboard {
    pub fleet_score: u32,
    pub healthy_robots: u32,
    pub degraded_robots: u32,
    pub mission_capacity_percent: u32,
    pub fleet_report: FleetReadinessReport,
}

/// Health operations dashboard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthDashboard {
    pub overall_status: String,
    pub check_count: u32,
    pub critical_count: u32,
    pub degraded_count: u32,
    pub policies_active: u32,
}

/// Mission operations dashboard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionDashboard {
    pub missions_total: u32,
    pub missions_achievable: u32,
    pub blocked_missions: u32,
    pub approval_gates: u32,
}

impl ReadinessDashboard {
    pub fn from_reports(reports: Vec<ReadinessReport>) -> Self {
        let mission_ready_count = reports.iter().filter(|r| r.mission_ready).count() as u32;
        let degraded_count = reports
            .iter()
            .filter(|r| r.status == crate::types::ReadinessStatus::Degraded)
            .count() as u32;
        let not_ready_count = reports.len() as u32 - mission_ready_count - degraded_count;
        let overall_score = if reports.is_empty() {
            0
        } else {
            reports.iter().map(|r| r.score.total).sum::<u32>() / reports.len() as u32
        };
        let mut top_issues: Vec<String> = reports
            .iter()
            .flat_map(|r| r.issues.iter().map(|i| i.message.clone()))
            .collect();
        top_issues.truncate(10);
        Self {
            overall_score,
            mission_ready_count,
            degraded_count,
            not_ready_count,
            top_issues,
            reports,
        }
    }
}

impl FleetDashboard {
    pub fn from_fleet_report(report: FleetReadinessReport) -> Self {
        Self {
            fleet_score: report.fleet_score,
            healthy_robots: report.healthy_robots,
            degraded_robots: report.degraded_robots,
            mission_capacity_percent: report.mission_capacity_percent,
            fleet_report: report,
        }
    }
}
