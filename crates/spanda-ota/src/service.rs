//! OTA rollout planning, state tracking, and certification gates.
//!
use crate::types::*;
use std::fs;
use std::path::{Path, PathBuf};

/// Block rollout when `--require-certify` is set and strict proof failed.
pub fn validate_rollout_certification(
    plan: &DeployPlan,
    options: &RolloutOptions,
) -> Result<(), String> {
    // Enforce strict certification proof before OTA rollout proceeds.
    //
    // Parameters:
    // - `plan` — deployment plan with embedded proof summary
    // - `options` — rollout options including `require_certify`
    //
    // Returns:
    // Ok when certification is not required or strict proof passed.
    //
    // Options:
    // None.
    //
    // Example:
    // validate_rollout_certification(&plan, &options)?;

    if !options.require_certify {
        return Ok(());
    }
    let Some(proof) = &plan.certification_proof else {
        return Err("Deploy plan missing certification proof summary".into());
    };
    if !proof.passed_strict {
        return Err(format!(
            "Deploy blocked — strict certification proof failed: {}",
            proof.summary
        ));
    }
    Ok(())
}

fn assignment_key(robot: &str, hardware: &str) -> String {
    format!("{robot}@{hardware}")
}

/// Stable deploy target key for robot/hardware pairs (`Robot@Hardware`).
pub fn deploy_target_key(robot: &str, hardware: &str) -> String {
    assignment_key(robot, hardware)
}

/// Compute a SHA-256 hex digest of a program artifact on disk.
pub fn hash_program_artifact(program_path: &str) -> Option<String> {
    // Hash the deployment source file when it exists locally.
    //
    // Parameters:
    // - `program_path` — path to the Spanda program file
    //
    // Returns:
    // Lowercase hex SHA-256 digest, or none when the file is unreadable.
    //
    // Options:
    // None.
    //
    // Example:
    // let hash = hash_program_artifact("rover.sd");

    let path = Path::new(program_path);
    if !path.exists() {
        return None;
    }
    let bytes = fs::read(path).ok()?;
    use sha2::{Digest, Sha256};
    Some(hex::encode(Sha256::digest(bytes)))
}

/// Plan which targets receive an update under the chosen rollout strategy.
pub fn plan_rollout(plan: &DeployPlan, options: &RolloutOptions) -> RolloutResult {
    // Select rollout targets without mutating persistent deploy state.
    if validate_rollout_certification(plan, options).is_err() {
        return RolloutResult {
            strategy: options.strategy,
            version: options.version.clone(),
            dry_run: options.dry_run,
            steps: vec![],
            success: false,
        };
    }
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
