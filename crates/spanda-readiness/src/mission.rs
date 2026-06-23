//! Mission-level verification against hardware, capabilities, and safety.

use serde::{Deserialize, Serialize};
use spanda_ast::foundations::MissionDecl;
use spanda_ast::nodes::Program;
use spanda_capability::{check_minimum_capabilities, infer_robot_capabilities, lookup_capability};
use spanda_hardware::{verify_program_compatibility, CompatSeverity, VerifyOptions};

/// Mission verification result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionVerificationReport {
    pub achievable: bool,
    pub mission_name: Option<String>,
    pub robot: Option<String>,
    pub required_capabilities: Vec<String>,
    pub hardware_satisfied: bool,
    pub capabilities_satisfied: bool,
    pub connectivity_satisfied: bool,
    pub battery_sufficient: bool,
    pub compute_sufficient: bool,
    pub safety_satisfied: bool,
    pub issues: Vec<String>,
}

/// Verify that declared missions are achievable on configured robots.
pub fn verify_mission(program: &Program, target: Option<&str>) -> Vec<MissionVerificationReport> {
    let Program::Program { robots, .. } = program;
    let mut reports = Vec::new();
    let verify_opts = VerifyOptions {
        target: target.map(String::from),
        all_targets: target.is_none(),
        simulate: false,
        strict_certify: false,
    };
    let hw = verify_program_compatibility(program, &verify_opts);
    let minimum = check_minimum_capabilities(program);
    let robot_caps = infer_robot_capabilities(program);

    for robot in robots {
        let spanda_ast::nodes::RobotDecl::RobotDecl {
            name,
            mission,
            exposes_capabilities,
            ..
        } = robot;
        let Some(mission) = mission.as_ref() else {
            continue;
        };
        let MissionDecl::MissionDecl {
            name: mname,
            required_capabilities,
            ..
        } = mission;

        let mut issues = Vec::new();
        let mut caps_ok = true;
        for cap in required_capabilities {
            let has_cap = exposes_capabilities.iter().any(|c| c == cap)
                || robot_caps
                    .iter()
                    .find(|r| r.robot == *name)
                    .map(|r| {
                        r.declared.contains(cap)
                            || r.inferred.contains(cap)
                            || r.rows
                                .iter()
                                .any(|row| row.capability == *cap && row.status == "OK")
                    })
                    .unwrap_or(false);
            if !has_cap {
                caps_ok = false;
                issues.push(format!("Missing required capability: {cap}"));
            }
            if lookup_capability(cap).is_none() {
                issues.push(format!("Unknown capability in mission: {cap}"));
            }
        }

        let battery_ok = !hw.items.iter().any(|i| {
            i.message.to_lowercase().contains("battery") && i.severity == CompatSeverity::Error
        });
        if !battery_ok {
            issues.push("Insufficient battery for mission duration".into());
        }

        let connectivity_ok = !hw
            .items
            .iter()
            .any(|i| i.category == "connectivity" && i.severity == CompatSeverity::Error);
        if !connectivity_ok {
            issues.push("Required connectivity not available".into());
        }

        let safety_ok = minimum.compatible;
        if !safety_ok {
            issues.push("Safety capability requirements not satisfied".into());
        }

        let hardware_ok = hw.compatible;
        if !hardware_ok {
            issues.push("Hardware compatibility check failed".into());
        }

        let achievable = caps_ok
            && battery_ok
            && connectivity_ok
            && safety_ok
            && hardware_ok
            && issues.is_empty();

        reports.push(MissionVerificationReport {
            achievable,
            mission_name: mname.clone(),
            robot: Some(name.clone()),
            required_capabilities: required_capabilities.clone(),
            hardware_satisfied: hardware_ok,
            capabilities_satisfied: caps_ok,
            connectivity_satisfied: connectivity_ok,
            battery_sufficient: battery_ok,
            compute_sufficient: true,
            safety_satisfied: safety_ok,
            issues,
        });
    }

    if reports.is_empty() {
        reports.push(MissionVerificationReport {
            achievable: minimum.compatible && hw.compatible,
            mission_name: None,
            robot: None,
            required_capabilities: Vec::new(),
            hardware_satisfied: hw.compatible,
            capabilities_satisfied: minimum.compatible,
            connectivity_satisfied: true,
            battery_sufficient: true,
            compute_sufficient: true,
            safety_satisfied: minimum.compatible,
            issues: minimum.errors.clone(),
        });
    }

    reports
}

/// Verify mission from source.
pub fn verify_mission_source(
    source: &str,
    target: Option<&str>,
) -> Result<Vec<MissionVerificationReport>, spanda_error::SpandaError> {
    let tokens = spanda_lexer::tokenize(source)?;
    let program = spanda_parser::parse(tokens)?;
    Ok(verify_mission(&program, target))
}
