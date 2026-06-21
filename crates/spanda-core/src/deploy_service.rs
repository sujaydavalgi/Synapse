//! OTA deployment planning, rollout, rollback, and state tracking for Spanda programs.
//!
//! Production fleets use `spanda deploy plan|rollout|rollback|status` against declared
//! `deploy Robot to Hardware` bindings. This module is the in-process deploy runtime service.

use crate::ast::Program;
use crate::foundations::DeployDecl;
use crate::robotics_platform::CertifyDecl;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
    pub assignments: Vec<DeployAssignment>,
    pub certifications: Vec<String>,
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
}

impl Default for RolloutOptions {
    fn default() -> Self {
        Self {
            strategy: RolloutStrategy::All,
            canary_percent: 10,
            staged_phases: vec![10, 50, 100],
            version: "1.0.0".into(),
            dry_run: false,
        }
    }
}

fn assignment_key(robot: &str, hardware: &str) -> String {
    format!("{robot}@{hardware}")
}

/// Build a deployment plan from a parsed program.
pub fn build_deploy_plan(program: &Program, program_path: &str, version: &str) -> DeployPlan {
    // Extract deploy targets and certification metadata from the program AST.
    //
    // Parameters:
    // - `program` — parsed Spanda program
    // - `program_path` — source file path for reporting
    // - `version` — release version label
    //
    // Returns:
    // Deployment plan with robot/hardware assignments.
    //
    // Options:
    // None.
    //
    // Example:
    // let plan = build_deploy_plan(&program, "rover.sd", "1.2.0");

    let Program::Program {
        deployments,
        certifications,
        ..
    } = program;
    let mut assignments = Vec::new();
    for deploy in deployments {
        let DeployDecl::DeployDecl {
            robot_name,
            targets,
            ..
        } = deploy;
        for hardware in targets {
            assignments.push(DeployAssignment {
                robot_name: robot_name.clone(),
                hardware: hardware.clone(),
            });
        }
    }
    assignments.sort_by(|a, b| {
        a.robot_name
            .cmp(&b.robot_name)
            .then(a.hardware.cmp(&b.hardware))
    });
    let certs = certifications
        .iter()
        .map(|c| {
            let CertifyDecl::CertifyDecl {
                standard, level, ..
            } = c;
            match level {
                Some(l) => format!("{}:{}", standard.as_str(), l),
                None => standard.as_str().to_string(),
            }
        })
        .collect();
    DeployPlan {
        program: program_path.to_string(),
        version: version.to_string(),
        assignments,
        certifications: certs,
    }
}

/// Plan which targets receive an update under the chosen rollout strategy.
pub fn plan_rollout(plan: &DeployPlan, options: &RolloutOptions) -> RolloutResult {
    // Select rollout targets without mutating persistent deploy state.
    let total = plan.assignments.len();
    let mut steps = Vec::new();

    if total == 0 {
        return RolloutResult {
            strategy: options.strategy,
            version: options.version.clone(),
            dry_run: options.dry_run,
            steps,
            success: true,
        };
    }

    match options.strategy {
        RolloutStrategy::All => {
            for assignment in &plan.assignments {
                steps.push(RolloutStep {
                    robot_name: assignment.robot_name.clone(),
                    hardware: assignment.hardware.clone(),
                    status: if options.dry_run {
                        RolloutStepStatus::Pending
                    } else {
                        RolloutStepStatus::Deployed
                    },
                    version: options.version.clone(),
                    phase_percent: Some(100),
                });
            }
        }
        RolloutStrategy::Canary => {
            let pct = options.canary_percent.clamp(1, 100);
            let canary_count = ((total as f64 * pct as f64 / 100.0).ceil() as usize).max(1);
            for (idx, assignment) in plan.assignments.iter().enumerate() {
                let deploy = idx < canary_count;
                steps.push(RolloutStep {
                    robot_name: assignment.robot_name.clone(),
                    hardware: assignment.hardware.clone(),
                    status: if deploy {
                        if options.dry_run {
                            RolloutStepStatus::Pending
                        } else {
                            RolloutStepStatus::Deployed
                        }
                    } else {
                        RolloutStepStatus::Skipped
                    },
                    version: options.version.clone(),
                    phase_percent: Some(if deploy { pct } else { 0 }),
                });
            }
        }
        RolloutStrategy::Staged => {
            let phases = if options.staged_phases.is_empty() {
                vec![100]
            } else {
                options.staged_phases.clone()
            };
            let final_phase = *phases.last().unwrap_or(&100);
            let deploy_count =
                ((total as f64 * final_phase as f64 / 100.0).ceil() as usize).max(1);
            for (idx, assignment) in plan.assignments.iter().enumerate() {
                let deploy = idx < deploy_count;
                steps.push(RolloutStep {
                    robot_name: assignment.robot_name.clone(),
                    hardware: assignment.hardware.clone(),
                    status: if deploy {
                        if options.dry_run {
                            RolloutStepStatus::Pending
                        } else {
                            RolloutStepStatus::Deployed
                        }
                    } else {
                        RolloutStepStatus::Skipped
                    },
                    version: options.version.clone(),
                    phase_percent: Some(final_phase),
                });
            }
        }
    }

    RolloutResult {
        strategy: options.strategy,
        version: options.version.clone(),
        dry_run: options.dry_run,
        success: !steps
            .iter()
            .any(|s| s.status == RolloutStepStatus::Failed),
        steps,
    }
}

/// Apply a successful rollout to persistent deploy state.
pub fn apply_rollout(state: &mut DeployState, result: &RolloutResult) {
    // Record deployed versions and append rollout history.
    if result.dry_run {
        return;
    }
    for step in &result.steps {
        if step.status != RolloutStepStatus::Deployed {
            continue;
        }
        let key = assignment_key(&step.robot_name, &step.hardware);
        if let Some(prev) = state.current_version.get(&key) {
            state.previous_version.insert(key.clone(), prev.clone());
        }
        state
            .current_version
            .insert(key, step.version.clone());
    }
    state.history.push(result.clone());
}

/// Roll back deployed targets to the previous recorded version.
pub fn rollback_targets(
    state: &mut DeployState,
    plan: &DeployPlan,
    to_previous: bool,
) -> RolloutResult {
    // Revert each assignment that has a known previous version.
    let mut steps = Vec::new();
    for assignment in &plan.assignments {
        let key = assignment_key(&assignment.robot_name, &assignment.hardware);
        let target_version = if to_previous {
            state.previous_version.get(&key).cloned()
        } else {
            state.current_version.get(&key).cloned()
        };
        let (status, version) = match target_version {
            Some(v) => (RolloutStepStatus::RolledBack, v),
            None => (RolloutStepStatus::Skipped, "unknown".into()),
        };
        if status == RolloutStepStatus::RolledBack {
            if let Some(cur) = state.current_version.get(&key) {
                state.previous_version.insert(key.clone(), cur.clone());
            }
            state.current_version.insert(key, version.clone());
        }
        steps.push(RolloutStep {
            robot_name: assignment.robot_name.clone(),
            hardware: assignment.hardware.clone(),
            status,
            version,
            phase_percent: None,
        });
    }
    let result = RolloutResult {
        strategy: RolloutStrategy::All,
        version: "rollback".into(),
        dry_run: false,
        success: steps
            .iter()
            .any(|s| s.status == RolloutStepStatus::RolledBack),
        steps,
    };
    state.history.push(result.clone());
    result
}

/// Default path for OTA state under `.spanda/deploy-state.json`.
pub fn default_state_path() -> PathBuf {
    PathBuf::from(".spanda/deploy-state.json")
}

/// Load deploy state from disk, or return default when missing.
pub fn load_deploy_state(path: &Path) -> DeployState {
    if !path.exists() {
        return DeployState::default();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Persist deploy state to disk.
pub fn save_deploy_state(path: &Path, state: &DeployState) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Span;
    use crate::foundations::DeployDecl;

    fn empty_span() -> Span {
        Span {
            start: crate::ast::SourceLocation {
                line: 1,
                column: 1,
                offset: 0,
            },
            end: crate::ast::SourceLocation {
                line: 1,
                column: 1,
                offset: 0,
            },
        }
    }

    fn sample_program() -> Program {
        Program::Program {
            module_name: None,
            imports: vec![],
            functions: vec![],
            tests: vec![],
            extern_functions: vec![],
            structs: vec![],
            enums: vec![],
            traits: vec![],
            hardware_profiles: vec![],
            deployments: vec![DeployDecl::DeployDecl {
                robot_name: "Rover".into(),
                targets: vec!["Jetson".into(), "Orin".into()],
                span: empty_span(),
            }],
            requires_hardware: None,
            requires_network: None,
            requires_connectivity: None,
            geofences: vec![],
            fleets: vec![],
            program_safety_zones: vec![],
            certifications: vec![],
            connectivity_policies: vec![],
            ble_services: vec![],
            simulate_compatibility: None,
            messages: vec![],
            validate_rules: vec![],
            robots: vec![],
            span: empty_span(),
        }
    }

    #[test]
    fn build_plan_from_deployments() {
        let plan = build_deploy_plan(&sample_program(), "ota.sd", "2.0.0");
        assert_eq!(plan.assignments.len(), 2);
        assert_eq!(plan.version, "2.0.0");
    }

    #[test]
    fn canary_rollout_deploys_subset() {
        let plan = build_deploy_plan(&sample_program(), "ota.sd", "1.0.0");
        let result = plan_rollout(
            &plan,
            &RolloutOptions {
                strategy: RolloutStrategy::Canary,
                canary_percent: 50,
                version: "1.1.0".into(),
                dry_run: false,
                ..Default::default()
            },
        );
        let deployed = result
            .steps
            .iter()
            .filter(|s| s.status == RolloutStepStatus::Deployed)
            .count();
        assert_eq!(deployed, 1);
    }

    #[test]
    fn rollback_restores_previous_version() {
        let plan = build_deploy_plan(&sample_program(), "ota.sd", "1.0.0");
        let mut state = DeployState::default();
        state
            .current_version
            .insert("Rover@Jetson".into(), "2.0.0".into());
        state
            .previous_version
            .insert("Rover@Jetson".into(), "1.0.0".into());
        let rollback = rollback_targets(&mut state, &plan, true);
        assert!(rollback.success);
        assert_eq!(
            state.current_version.get("Rover@Jetson"),
            Some(&"1.0.0".into())
        );
    }
}
