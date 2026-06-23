//! Digital twin readiness comparison.

use crate::engine::evaluate_readiness;
use crate::types::{ReadinessOptions, ReadinessStatus, TwinReadinessStatus};
use spanda_ast::nodes::Program;
use spanda_capability::infer_robot_capabilities;
use spanda_runtime::replay::MissionTrace;
use std::path::Path;

/// Compare physical robot configuration against a digital twin trace export.
pub fn evaluate_twin_readiness(
    program: &Program,
    twin_trace_path: Option<&Path>,
) -> TwinReadinessStatus {
    let physical = evaluate_readiness(program, &ReadinessOptions::default());
    let caps = infer_robot_capabilities(program);

    let mut configuration_drift = Vec::new();
    let mut capability_drift = Vec::new();
    let mut health_drift = Vec::new();

    if let Some(path) = twin_trace_path {
        if let Ok(trace) = MissionTrace::load(path) {
            let provider_modules: std::collections::HashSet<String> = trace
                .frames
                .iter()
                .filter_map(|f| {
                    f.payload
                        .get("module")
                        .and_then(|v| v.as_str())
                        .map(String::from)
                })
                .collect();
            for report in &caps {
                for row in &report.rows {
                    if !row.required_components.is_empty()
                        && provider_modules.is_empty()
                        && row.status != "OK"
                    {
                        configuration_drift.push(format!(
                            "Twin trace missing provider activity for {}",
                            row.capability
                        ));
                    }
                }
            }
        } else {
            configuration_drift.push(format!("Could not load twin trace: {}", path.display()));
        }
    } else {
        let Program::Program { robots, .. } = program;
        let has_twin = robots.iter().any(|r| {
            let spanda_ast::nodes::RobotDecl::RobotDecl { twin, .. } = r;
            twin.is_some()
        });
        if !has_twin {
            configuration_drift.push("No twin declaration or trace export provided".into());
        }
    }

    for issue in &physical.issues {
        if issue.factor == "Health" {
            health_drift.push(issue.message.clone());
        }
        if issue.factor == "Capabilities" {
            capability_drift.push(issue.message.clone());
        }
    }

    let twin_ready = configuration_drift.is_empty() && capability_drift.is_empty();
    let overall = if physical.mission_ready && twin_ready {
        ReadinessStatus::Ready
    } else if physical.mission_ready || twin_ready {
        ReadinessStatus::Degraded
    } else {
        ReadinessStatus::NotReady
    };

    TwinReadinessStatus {
        physical_ready: physical.mission_ready,
        twin_ready,
        configuration_drift,
        capability_drift,
        health_drift,
        overall,
    }
}
