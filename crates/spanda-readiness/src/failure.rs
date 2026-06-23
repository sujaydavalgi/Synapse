//! Failure impact analysis for component and provider outages.

use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use spanda_capability::infer_robot_capabilities;

/// Impact of a single failure scenario.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailureImpact {
    pub component: String,
    pub consequence: String,
    pub mitigation: String,
    pub severity: String,
}

/// Failure analysis report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailureAnalysisReport {
    pub robot: Option<String>,
    pub impacts: Vec<FailureImpact>,
}

const FAILURE_SCENARIOS: &[(&str, &str, &str, &str)] = &[
    (
        "GPS",
        "Navigation degraded; position uncertainty increases",
        "Switch to visual odometry",
        "High",
    ),
    (
        "Camera",
        "Obstacle avoidance degraded; perception limited",
        "Reduce speed and rely on Lidar",
        "High",
    ),
    (
        "Lidar",
        "Obstacle avoidance offline; collision risk elevated",
        "Halt autonomous motion; require operator takeover",
        "Critical",
    ),
    (
        "LTE",
        "Cloud telemetry and remote commands unavailable",
        "Offline mode activated; queue telemetry locally",
        "Medium",
    ),
    (
        "WiFi",
        "Local network commands unavailable",
        "Fall back to LTE or autonomous mode",
        "Medium",
    ),
    (
        "Battery",
        "Mission endurance reduced; forced return-to-base likely",
        "Return to charging dock; reduce mission scope",
        "High",
    ),
    (
        "Provider",
        "Dependent capability unavailable at runtime",
        "Use bundled fallback provider or safe stop",
        "High",
    ),
    (
        "Package",
        "Imported capability module missing or outdated",
        "Pin package version or install from registry",
        "Medium",
    ),
];

/// Analyze failure impacts for robots in a program.
pub fn analyze_failure(program: &Program) -> FailureAnalysisReport {
    let Program::Program { robots, .. } = program;
    let robot_name = robots.first().map(|r| {
        let spanda_ast::nodes::RobotDecl::RobotDecl { name, .. } = r;
        name.clone()
    });

    let caps = infer_robot_capabilities(program);
    let mut present_components = std::collections::HashSet::new();
    for report in &caps {
        for row in &report.rows {
            present_components.extend(row.required_components.iter().cloned());
        }
    }

    let impacts: Vec<FailureImpact> = FAILURE_SCENARIOS
        .iter()
        .filter(|(component, ..)| {
            present_components
                .iter()
                .any(|c| c.to_lowercase().contains(&component.to_lowercase()))
                || matches!(
                    *component,
                    "Provider" | "Package" | "Battery" | "LTE" | "WiFi"
                )
        })
        .map(
            |(component, consequence, mitigation, severity)| FailureImpact {
                component: (*component).into(),
                consequence: (*consequence).into(),
                mitigation: (*mitigation).into(),
                severity: (*severity).into(),
            },
        )
        .collect();

    FailureAnalysisReport {
        robot: robot_name,
        impacts,
    }
}

/// Analyze failure from source.
pub fn analyze_failure_source(
    source: &str,
) -> Result<FailureAnalysisReport, spanda_error::SpandaError> {
    let tokens = spanda_lexer::tokenize(source)?;
    let program = spanda_parser::parse(tokens)?;
    Ok(analyze_failure(&program))
}
