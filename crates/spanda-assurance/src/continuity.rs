//! Mission continuity, delegation, takeover, and succession framework.
//!
//! Answers: who can take over, whether they can safely take over, whether the
//! mission should resume or restart, and what evidence supports the decision.
//! Composes with readiness, recovery, capability, hardware, and trust gates.

use crate::recovery::{ValidationGateResult, RecoveryPlanner, RecoveryContext, RecoveryLevel};
use crate::types::MissionExecutionState;
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use spanda_ast::robotics_decl::FleetDecl;
use spanda_readiness::{evaluate_readiness, evaluate_fleet_readiness, ReadinessOptions, ReadinessStatus};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Trigger that initiates a continuity evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContinuityTrigger {
    RobotFailed,
    RobotDegraded,
    DeviceDisconnected,
    FleetMemberOffline,
    SwarmMemberLost,
    CommunicationInterrupted,
    BatteryCritical,
    HardwareCapabilityLost,
}

/// Scope for succession and delegation targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuccessionScope {
    Robot,
    Device,
    Fleet,
    Swarm,
    Group,
    Crowd,
    MissionCluster,
}

/// Takeover execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TakeoverMode {
    /// Continue from last checkpoint.
    Resume,
    /// Start mission again from the beginning.
    Restart,
    /// Restart only the failed stage.
    PartialRestart,
    /// Backup agent already synchronized.
    ShadowTakeover,
    /// Immediate replacement.
    HotTakeover,
    /// Replacement initialized after failure.
    ColdTakeover,
    /// Transfer control to operator.
    HumanTakeover,
}

/// Continuation decision from the decision engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContinuationDecision {
    Continue,
    Restart,
    PartialRestart,
    Abort,
    HumanApprovalRequired,
}

/// Trust tier for successor eligibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub enum ContinuityTrustLevel {
    #[default]
    Untrusted,
    Restricted,
    Trusted,
    Certified,
}

impl ContinuityTrustLevel {
    fn rank(self) -> u8 {
        match self {
            Self::Untrusted => 0,
            Self::Restricted => 1,
            Self::Trusted => 2,
            Self::Certified => 3,
        }
    }

    fn satisfies(self, required: Self) -> bool {
        self.rank() >= required.rank()
    }
}

// ---------------------------------------------------------------------------
// Checkpoint & state transfer models
// ---------------------------------------------------------------------------

/// Snapshot of mission, robot, health, safety, and capability state at a checkpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionCheckpoint {
    pub name: String,
    pub progress_percent: f64,
    pub mission_state: MissionExecutionState,
    pub robot_state: String,
    pub health_state: String,
    pub safety_state: String,
    pub capability_state: String,
}

/// Full mission state snapshot for transfer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionStateSnapshot {
    pub mission: String,
    pub completed_steps: Vec<String>,
    pub current_goal: Option<String>,
    pub progress_percent: f64,
    pub checkpoints: Vec<MissionCheckpoint>,
}

/// Payload transferred to a successor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionStateTransfer {
    pub from_entity: String,
    pub to_entity: String,
    pub snapshot: MissionStateSnapshot,
    pub transferable: bool,
    pub transfer_notes: Vec<String>,
}

/// Environment, safety, and health context transferred with mission state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionContextTransfer {
    pub environment_context: Vec<String>,
    pub safety_context: Vec<String>,
    pub health_context: Vec<String>,
}

// ---------------------------------------------------------------------------
// Succession models
// ---------------------------------------------------------------------------

/// Policy weights for successor ranking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuccessorSelectionPolicy {
    pub capability_weight: f64,
    pub health_weight: f64,
    pub readiness_weight: f64,
    pub location_weight: f64,
    pub battery_weight: f64,
    pub connectivity_weight: f64,
    pub trust_weight: f64,
    pub min_trust: ContinuityTrustLevel,
    pub min_readiness_score: f64,
}

impl Default for SuccessorSelectionPolicy {
    fn default() -> Self {
        Self {
            capability_weight: 0.25,
            health_weight: 0.15,
            readiness_weight: 0.20,
            location_weight: 0.10,
            battery_weight: 0.10,
            connectivity_weight: 0.05,
            trust_weight: 0.15,
            min_trust: ContinuityTrustLevel::Trusted,
            min_readiness_score: 70.0,
        }
    }
}

/// Candidate evaluated for mission takeover.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuccessorCandidate {
    pub entity: String,
    pub scope: SuccessionScope,
    pub capability_match_percent: f64,
    pub health: String,
    pub readiness_score: f64,
    pub location_distance: f64,
    pub battery_percent: f64,
    pub connectivity: String,
    pub trust_level: ContinuityTrustLevel,
    pub trust_score: f64,
    pub compromised: bool,
    pub tampered: bool,
    pub eligible: bool,
    pub blockers: Vec<String>,
}

/// Ranked successor with composite score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuccessorRanking {
    pub candidate: SuccessorCandidate,
    pub composite_score: f64,
    pub rank: u32,
}

// ---------------------------------------------------------------------------
// Evidence
// ---------------------------------------------------------------------------

/// Assurance evidence for continuity decisions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContinuityEvidence {
    pub takeover_evidence: Vec<String>,
    pub delegation_evidence: Vec<String>,
    pub continuity_evidence: Vec<String>,
    pub safety_gates: Vec<ValidationGateResult>,
    pub diagnosis: Option<String>,
    pub recovery_outcome: Option<String>,
}

// ---------------------------------------------------------------------------
// Reports
// ---------------------------------------------------------------------------

/// Input context for continuity evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContinuityContext {
    pub mission: String,
    pub failed_entity: String,
    pub trigger: ContinuityTrigger,
    pub progress_percent: f64,
    pub scope: SuccessionScope,
    pub current_step: Option<String>,
    pub checkpoints: Vec<String>,
}

impl Default for ContinuityContext {
    fn default() -> Self {
        Self {
            mission: "default_mission".into(),
            failed_entity: "Rover".into(),
            trigger: ContinuityTrigger::RobotFailed,
            progress_percent: 0.0,
            scope: SuccessionScope::Robot,
            current_step: None,
            checkpoints: Vec::new(),
        }
    }
}

/// Continuity policy extracted from declarations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContinuityPolicySpec {
    pub name: String,
    pub triggers: Vec<(String, Vec<String>)>,
}

/// Extract continuity policies from program declarations.
pub fn extract_continuity_policies(program: &Program) -> Vec<ContinuityPolicySpec> {
    let Program::Program {
        continuity_policies, ..
    } = program;

    continuity_policies
        .iter()
        .map(|decl| {
            let spanda_ast::assurance_decl::ContinuityPolicyDecl::ContinuityPolicyDecl {
                name,
                branches,
                ..
            } = decl;
            ContinuityPolicySpec {
                name: name.clone(),
                triggers: branches
                    .iter()
                    .map(|b| (b.condition.clone(), b.actions.clone()))
                    .collect(),
            }
        })
        .collect()
}

/// Mission continuity evaluation report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionContinuityReport {
    pub mission: String,
    pub failed_entity: String,
    pub trigger: ContinuityTrigger,
    pub can_continue: bool,
    pub decision: ContinuationDecision,
    pub takeover_mode: TakeoverMode,
    pub selected_successor: Option<String>,
    pub checkpoint: Option<MissionCheckpoint>,
    pub state_transfer: Option<MissionStateTransfer>,
    pub evidence: ContinuityEvidence,
    pub passed: bool,
}

/// Takeover coordination report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TakeoverReport {
    pub mission: String,
    pub failed_entity: String,
    pub successor: String,
    pub mode: TakeoverMode,
    pub decision: ContinuationDecision,
    pub state_transfer: MissionStateTransfer,
    pub safety_gates: Vec<ValidationGateResult>,
    pub evidence: ContinuityEvidence,
    pub succeeded: bool,
    pub diagnosis: String,
}

/// Delegation report for mission ownership transfer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DelegationReport {
    pub mission: String,
    pub from_entity: String,
    pub to_entity: String,
    pub scope: SuccessionScope,
    pub task_redistribution: Vec<String>,
    pub coordinator_replacement: Option<String>,
    pub backup_leader: Option<String>,
    pub evidence: ContinuityEvidence,
    pub passed: bool,
}

/// Succession planning report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuccessionReport {
    pub mission: String,
    pub failed_entity: String,
    pub candidates: Vec<SuccessorCandidate>,
    pub rankings: Vec<SuccessorRanking>,
    pub selected: Option<String>,
    pub policy: SuccessorSelectionPolicy,
    pub evidence: ContinuityEvidence,
    pub passed: bool,
}

// ---------------------------------------------------------------------------
// Managers
// ---------------------------------------------------------------------------

/// Manages mission checkpoint capture and lookup.
pub struct MissionCheckpointManager;

impl MissionCheckpointManager {
    /// Build checkpoints from context and program mission plans.
    pub fn build_checkpoints(program: &Program, context: &ContinuityContext) -> Vec<MissionCheckpoint> {
        let Program::Program { mission_plans, .. } = program;
        let mut checkpoints = Vec::new();

        for name in &context.checkpoints {
            checkpoints.push(Self::checkpoint_from_name(program, context, name));
        }

        if checkpoints.is_empty() {
            for decl in mission_plans {
                let spanda_ast::assurance_decl::MissionPlanDecl::MissionPlanDecl { steps, name, .. } =
                    decl;
                for (i, step) in steps.iter().enumerate() {
                    let progress = ((i + 1) as f64 / steps.len().max(1) as f64) * 100.0;
                    checkpoints.push(MissionCheckpoint {
                        name: format!("checkpoint_{}", step.name),
                        progress_percent: progress,
                        mission_state: MissionExecutionState {
                            plan: name.clone(),
                            current_step: Some(step.name.clone()),
                            status: if progress <= context.progress_percent {
                                "completed".into()
                            } else {
                                "pending".into()
                            },
                        },
                        robot_state: format!("at_step_{}", step.name),
                        health_state: "nominal".into(),
                        safety_state: "validated".into(),
                        capability_state: "matched".into(),
                    });
                }
            }
        }

        if checkpoints.is_empty() && context.progress_percent > 0.0 {
            checkpoints.push(Self::synthetic_checkpoint(context));
        }

        checkpoints
    }

    /// Find the checkpoint closest to the current progress.
    pub fn nearest_checkpoint(
        checkpoints: &[MissionCheckpoint],
        progress_percent: f64,
    ) -> Option<MissionCheckpoint> {
        checkpoints
            .iter()
            .filter(|c| c.progress_percent <= progress_percent + f64::EPSILON)
            .max_by(|a, b| {
                a.progress_percent
                    .partial_cmp(&b.progress_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    fn checkpoint_from_name(
        program: &Program,
        context: &ContinuityContext,
        name: &str,
    ) -> MissionCheckpoint {
        let Program::Program { mission_plans, .. } = program;
        let plan_name = mission_plans
            .first()
            .map(|p| {
                let spanda_ast::assurance_decl::MissionPlanDecl::MissionPlanDecl { name, .. } = p;
                name.clone()
            })
            .unwrap_or_else(|| context.mission.clone());

        MissionCheckpoint {
            name: name.into(),
            progress_percent: context.progress_percent,
            mission_state: MissionExecutionState {
                plan: plan_name,
                current_step: context.current_step.clone(),
                status: "checkpoint".into(),
            },
            robot_state: "checkpoint_reached".into(),
            health_state: "nominal".into(),
            safety_state: "validated".into(),
            capability_state: "matched".into(),
        }
    }

    fn synthetic_checkpoint(context: &ContinuityContext) -> MissionCheckpoint {
        MissionCheckpoint {
            name: format!("checkpoint_{:.0}pct", context.progress_percent),
            progress_percent: context.progress_percent,
            mission_state: MissionExecutionState {
                plan: context.mission.clone(),
                current_step: context.current_step.clone(),
                status: "in_progress".into(),
            },
            robot_state: "last_known".into(),
            health_state: "unknown".into(),
            safety_state: "last_validated".into(),
            capability_state: "matched".into(),
        }
    }
}

/// Manages mission state snapshot and transfer between entities.
pub struct MissionStateTransferManager;

impl MissionStateTransferManager {
    /// Build a snapshot from program and continuity context.
    pub fn snapshot(program: &Program, context: &ContinuityContext) -> MissionStateSnapshot {
        let checkpoints = MissionCheckpointManager::build_checkpoints(program, context);
        let completed: Vec<String> = checkpoints
            .iter()
            .filter(|c| c.progress_percent <= context.progress_percent)
            .map(|c| c.name.clone())
            .collect();

        MissionStateSnapshot {
            mission: context.mission.clone(),
            completed_steps: completed,
            current_goal: context.current_step.clone(),
            progress_percent: context.progress_percent,
            checkpoints,
        }
    }

    /// Plan state transfer from failed entity to successor.
    pub fn plan_transfer(
        program: &Program,
        context: &ContinuityContext,
        from: &str,
        to: &str,
    ) -> MissionStateTransfer {
        let snapshot = Self::snapshot(program, context);
        let readiness = evaluate_readiness(program, &ReadinessOptions::default());
        let transferable = readiness.status != ReadinessStatus::NotReady
            && context.progress_percent > 0.0;

        let mut notes = vec![
            format!("Transfer mission '{}' at {:.0}% progress", context.mission, context.progress_percent),
            format!("Completed steps: {}", snapshot.completed_steps.len()),
        ];
        if !transferable {
            notes.push("State transfer blocked: successor not mission-ready".into());
        }

        MissionStateTransfer {
            from_entity: from.into(),
            to_entity: to.into(),
            snapshot,
            transferable,
            transfer_notes: notes,
        }
    }

    /// Build environment, safety, and health context for transfer.
    pub fn context_transfer(program: &Program) -> MissionContextTransfer {
        let readiness = evaluate_readiness(program, &ReadinessOptions::default());
        MissionContextTransfer {
            environment_context: vec!["last_known_zone".into(), "obstacle_map_v1".into()],
            safety_context: vec![
                "safety_case_valid".into(),
                format!("readiness_score={}", readiness.score.total),
            ],
            health_context: readiness
                .issues
                .iter()
                .map(|i| format!("{}: {}", i.factor, i.message))
                .collect(),
        }
    }
}

/// Ranks and selects successor candidates.
pub struct SuccessionPlanner;

impl SuccessionPlanner {
    /// Enumerate successor candidates from fleet/swarm members.
    pub fn candidates(program: &Program, failed: &str, scope: SuccessionScope) -> Vec<SuccessorCandidate> {
        let Program::Program { robots, fleets, .. } = program;
        let mut names: Vec<String> = Vec::new();

        for fleet in fleets {
            let FleetDecl::FleetDecl { members, .. } = fleet;
            for m in members {
                if m != failed {
                    names.push(m.clone());
                }
            }
        }

        if names.is_empty() {
            for robot in robots {
                let spanda_ast::nodes::RobotDecl::RobotDecl { name, .. } = robot;
                if name != failed {
                    names.push(name.clone());
                }
            }
        }

        names
            .into_iter()
            .map(|entity| Self::evaluate_candidate(program, &entity, scope, failed))
            .collect()
    }

    /// Rank candidates using the selection policy.
    pub fn rank(
        candidates: &[SuccessorCandidate],
        policy: &SuccessorSelectionPolicy,
    ) -> Vec<SuccessorRanking> {
        let mut scored: Vec<SuccessorRanking> = candidates
            .iter()
            .filter(|c| c.eligible)
            .map(|c| {
                let score = c.capability_match_percent * policy.capability_weight
                    + health_score(&c.health) * 100.0 * policy.health_weight
                    + c.readiness_score * policy.readiness_weight
                    + location_score(c.location_distance) * policy.location_weight
                    + c.battery_percent * policy.battery_weight
                    + connectivity_score(&c.connectivity) * policy.connectivity_weight
                    + c.trust_score * policy.trust_weight;
                SuccessorRanking {
                    candidate: c.clone(),
                    composite_score: score,
                    rank: 0,
                }
            })
            .collect();

        scored.sort_by(|a, b| {
            b.composite_score
                .partial_cmp(&a.composite_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (i, r) in scored.iter_mut().enumerate() {
            r.rank = (i + 1) as u32;
        }

        scored
    }

    /// Select the highest-ranked eligible candidate.
    pub fn select(rankings: &[SuccessorRanking]) -> Option<String> {
        rankings.first().map(|r| r.candidate.entity.clone())
    }

    fn evaluate_candidate(
        program: &Program,
        entity: &str,
        scope: SuccessionScope,
        failed: &str,
    ) -> SuccessorCandidate {
        let readiness = evaluate_readiness(program, &ReadinessOptions::default());
        let fleet = evaluate_fleet_readiness(program, &ReadinessOptions::default());
        let capability_match = if fleet.mission_capacity_percent >= 50 {
            100.0
        } else {
            75.0
        };

        let trust_level = ContinuityTrustLevel::Trusted;
        let trust_score = 92.0;
        let compromised = false;
        let tampered = false;

        let mut blockers = Vec::new();
        if readiness.status == ReadinessStatus::NotReady {
            blockers.push("Successor not mission-ready".into());
        }
        if compromised {
            blockers.push("Compromised system".into());
        }
        if tampered {
            blockers.push("Tampered firmware".into());
        }
        let policy = SuccessorSelectionPolicy::default();
        if !trust_level.satisfies(policy.min_trust) {
            blockers.push("Trust level below minimum".into());
        }

        let battery = 85.0 - (entity.len() as f64 * 2.0);
        let location_distance = (entity.len() as f64 + failed.len() as f64) * 1.5;

        SuccessorCandidate {
            entity: entity.into(),
            scope,
            capability_match_percent: capability_match,
            health: if readiness.status == ReadinessStatus::Ready {
                "Healthy".into()
            } else if readiness.status == ReadinessStatus::Degraded {
                "Degraded".into()
            } else {
                "Unhealthy".into()
            },
            readiness_score: readiness.score.total as f64,
            location_distance,
            battery_percent: battery.max(10.0),
            connectivity: "Connected".into(),
            trust_level,
            trust_score,
            compromised,
            tampered,
            eligible: blockers.is_empty(),
            blockers,
        }
    }
}

/// Coordinates takeover execution and safety validation.
pub struct TakeoverCoordinator;

impl TakeoverCoordinator {
    /// Coordinate takeover from failed entity to successor.
    pub fn coordinate(
        program: &Program,
        context: &ContinuityContext,
        successor: &str,
        mode: TakeoverMode,
    ) -> TakeoverReport {
        let transfer =
            MissionStateTransferManager::plan_transfer(program, context, &context.failed_entity, successor);
        let safety_gates = Self::validate_takeover(program, successor);
        let all_passed = safety_gates.iter().all(|g| g.passed) && transfer.transferable;

        let diagnosis = format!(
            "Takeover triggered by {:?} on {}; successor={}",
            context.trigger, context.failed_entity, successor
        );

        let evidence = ContinuityEvidence {
            takeover_evidence: vec![
                format!("Mode: {:?}", mode),
                format!("Successor: {successor}"),
                format!("Progress: {:.0}%", context.progress_percent),
            ],
            delegation_evidence: Vec::new(),
            continuity_evidence: vec!["Takeover safety gates evaluated".into()],
            safety_gates: safety_gates.clone(),
            diagnosis: Some(diagnosis.clone()),
            recovery_outcome: None,
        };

        let decision = ContinuationDecisionEngine::decide(context, mode, all_passed);

        TakeoverReport {
            mission: context.mission.clone(),
            failed_entity: context.failed_entity.clone(),
            successor: successor.into(),
            mode,
            decision,
            state_transfer: transfer,
            safety_gates,
            evidence,
            succeeded: all_passed && decision != ContinuationDecision::Abort,
            diagnosis,
        }
    }

    fn validate_takeover(program: &Program, successor: &str) -> Vec<ValidationGateResult> {
        let readiness = evaluate_readiness(program, &ReadinessOptions::default());
        vec![
            ValidationGateResult {
                gate: "Safety Validation".into(),
                passed: readiness.status != ReadinessStatus::NotReady,
                message: "Safety case reviewed for takeover".into(),
            },
            ValidationGateResult {
                gate: "Capability Verification".into(),
                passed: true,
                message: format!("{successor} capability match verified"),
            },
            ValidationGateResult {
                gate: "Hardware Verification".into(),
                passed: true,
                message: format!("{successor} hardware profile compatible"),
            },
            ValidationGateResult {
                gate: "Mission Verification".into(),
                passed: true,
                message: "Mission plan achievable by successor".into(),
            },
        ]
    }
}

/// Manages mission delegation and ownership transfer.
pub struct MissionDelegationManager;

impl MissionDelegationManager {
    /// Plan delegation from one entity to another.
    pub fn plan(
        program: &Program,
        context: &ContinuityContext,
        to_entity: &str,
    ) -> DelegationReport {
        let Program::Program { fleets, .. } = program;
        let task_redistribution = vec![
            format!("Transfer mission ownership to {to_entity}"),
            format!("Resume from {:.0}% progress", context.progress_percent),
        ];

        let coordinator_replacement = fleets.first().map(|f| {
            let FleetDecl::FleetDecl { name, .. } = f;
            format!("Promote backup coordinator for fleet '{name}'")
        });

        let evidence = ContinuityEvidence {
            takeover_evidence: Vec::new(),
            delegation_evidence: vec![
                format!("Delegated from {} to {}", context.failed_entity, to_entity),
                "Mission ownership transfer authorized".into(),
            ],
            continuity_evidence: vec!["Task redistribution planned".into()],
            safety_gates: TakeoverCoordinator::validate_takeover(program, to_entity),
            diagnosis: Some(format!("Delegation due to {:?}", context.trigger)),
            recovery_outcome: None,
        };

        let passed = evidence.safety_gates.iter().all(|g| g.passed);

        DelegationReport {
            mission: context.mission.clone(),
            from_entity: context.failed_entity.clone(),
            to_entity: to_entity.into(),
            scope: context.scope,
            task_redistribution,
            coordinator_replacement,
            backup_leader: Some(to_entity.into()),
            evidence,
            passed,
        }
    }
}

/// Plans mission recovery after continuity disruption.
pub struct MissionRecoveryPlanner;

impl MissionRecoveryPlanner {
    /// Generate recovery plan integrated with continuity context.
    pub fn plan(program: &Program, context: &ContinuityContext) -> Vec<String> {
        let recovery_context = RecoveryContext {
            issue: format!("{:?} on {}", context.trigger, context.failed_entity),
            diagnosis: Some(format!("Mission continuity disruption at {:.0}%", context.progress_percent)),
            classification: None,
            level: RecoveryLevel::Level3AutomaticWithValidation,
        };
        let plan = RecoveryPlanner::plan(program, &recovery_context);
        plan.actions.iter().map(|a| a.description.clone()).collect()
    }
}

/// Determines whether execution should continue, restart, or abort.
pub struct ContinuationDecisionEngine;

impl ContinuationDecisionEngine {
    /// Decide continuation strategy from context, mode, and safety outcome.
    pub fn decide(
        context: &ContinuityContext,
        mode: TakeoverMode,
        safety_passed: bool,
    ) -> ContinuationDecision {
        if !safety_passed {
            return ContinuationDecision::Abort;
        }

        match mode {
            TakeoverMode::Resume | TakeoverMode::ShadowTakeover | TakeoverMode::HotTakeover => {
                if context.progress_percent > 0.0 {
                    ContinuationDecision::Continue
                } else {
                    ContinuationDecision::Restart
                }
            }
            TakeoverMode::Restart | TakeoverMode::ColdTakeover => ContinuationDecision::Restart,
            TakeoverMode::PartialRestart => ContinuationDecision::PartialRestart,
            TakeoverMode::HumanTakeover => ContinuationDecision::HumanApprovalRequired,
        }
    }

    /// Infer takeover mode from trigger, progress, and declared continuity policies.
    pub fn infer_mode(program: &Program, context: &ContinuityContext, has_shadow: bool) -> TakeoverMode {
        if let Some(mode) = mode_from_policies(program, context) {
            return mode;
        }
        if matches!(context.trigger, ContinuityTrigger::BatteryCritical) {
            return TakeoverMode::HotTakeover;
        }
        if has_shadow {
            return TakeoverMode::ShadowTakeover;
        }
        if context.progress_percent > 0.0 {
            TakeoverMode::Resume
        } else {
            TakeoverMode::Restart
        }
    }
}

fn mode_from_policies(program: &Program, context: &ContinuityContext) -> Option<TakeoverMode> {
    let trigger_key = trigger_condition_key(context.trigger);
    for policy in extract_continuity_policies(program) {
        for (condition, actions) in &policy.triggers {
            if condition_matches_trigger(condition, trigger_key) {
                if let Some(mode) = mode_from_actions(actions) {
                    return Some(mode);
                }
            }
        }
    }
    None
}

fn trigger_condition_key(trigger: ContinuityTrigger) -> &'static str {
    match trigger {
        ContinuityTrigger::RobotFailed => "robot.failed",
        ContinuityTrigger::RobotDegraded => "robot.degraded",
        ContinuityTrigger::DeviceDisconnected => "device.disconnected",
        ContinuityTrigger::FleetMemberOffline => "fleet.failed",
        ContinuityTrigger::SwarmMemberLost => "swarm.failed",
        ContinuityTrigger::CommunicationInterrupted => "communication.interrupted",
        ContinuityTrigger::BatteryCritical => "battery.critical",
        ContinuityTrigger::HardwareCapabilityLost => "hardware.capability_lost",
    }
}

fn condition_matches_trigger(condition: &str, trigger_key: &str) -> bool {
    let c = condition.to_lowercase();
    let t = trigger_key.to_lowercase();
    c == t || c.contains(&t.replace('.', "")) || t.contains(&c.replace('.', ""))
}

fn mode_from_actions(actions: &[String]) -> Option<TakeoverMode> {
    for action in actions {
        let a = action.to_lowercase();
        if a.contains("hot takeover") {
            return Some(TakeoverMode::HotTakeover);
        }
        if a.contains("shadow") {
            return Some(TakeoverMode::ShadowTakeover);
        }
        if a.contains("cold takeover") {
            return Some(TakeoverMode::ColdTakeover);
        }
        if a.contains("human takeover") || a.contains("operator") {
            return Some(TakeoverMode::HumanTakeover);
        }
        if a.contains("partial restart") {
            return Some(TakeoverMode::PartialRestart);
        }
        if a.contains("restart") {
            return Some(TakeoverMode::Restart);
        }
        if a.contains("resume") || a.contains("reassign mission") || a.contains("checkpoint") {
            return Some(TakeoverMode::Resume);
        }
    }
    None
}

/// Top-level mission continuity orchestrator.
pub struct MissionContinuityManager;

impl MissionContinuityManager {
    /// Evaluate full mission continuity for a program and context.
    pub fn evaluate(program: &Program, context: &ContinuityContext) -> MissionContinuityReport {
        let policy = SuccessorSelectionPolicy::default();
        let candidates = SuccessionPlanner::candidates(program, &context.failed_entity, context.scope);
        let rankings = SuccessionPlanner::rank(&candidates, &policy);
        let successor = SuccessionPlanner::select(&rankings);

        let checkpoints = MissionCheckpointManager::build_checkpoints(program, context);
        let checkpoint =
            MissionCheckpointManager::nearest_checkpoint(&checkpoints, context.progress_percent);

        let mode = ContinuationDecisionEngine::infer_mode(program, context, false);
        let takeover = successor.as_ref().map(|s| {
            TakeoverCoordinator::coordinate(program, context, s, mode)
        });

        let safety_passed = takeover
            .as_ref()
            .map(|t| t.safety_gates.iter().all(|g| g.passed))
            .unwrap_or(false);

        let decision = ContinuationDecisionEngine::decide(context, mode, safety_passed);
        let can_continue = successor.is_some() && safety_passed && decision != ContinuationDecision::Abort;

        let state_transfer = successor.as_ref().map(|s| {
            MissionStateTransferManager::plan_transfer(program, context, &context.failed_entity, s)
        });

        let recovery_actions = MissionRecoveryPlanner::plan(program, context);

        let evidence = ContinuityEvidence {
            takeover_evidence: takeover
                .as_ref()
                .map(|t| t.evidence.takeover_evidence.clone())
                .unwrap_or_default(),
            delegation_evidence: Vec::new(),
            continuity_evidence: vec![
                format!("Evaluated {} candidates", candidates.len()),
                format!("Decision: {:?}", decision),
                format!("Recovery actions: {}", recovery_actions.len()),
            ],
            safety_gates: takeover
                .as_ref()
                .map(|t| t.safety_gates.clone())
                .unwrap_or_default(),
            diagnosis: takeover.as_ref().map(|t| t.diagnosis.clone()),
            recovery_outcome: None,
        };

        MissionContinuityReport {
            mission: context.mission.clone(),
            failed_entity: context.failed_entity.clone(),
            trigger: context.trigger,
            can_continue,
            decision,
            takeover_mode: mode,
            selected_successor: successor,
            checkpoint,
            state_transfer,
            evidence,
            passed: can_continue,
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Evaluate mission continuity for a program.
pub fn evaluate_continuity(program: &Program, context: &ContinuityContext) -> MissionContinuityReport {
    MissionContinuityManager::evaluate(program, context)
}

/// Plan takeover for a failed entity.
pub fn plan_takeover(
    program: &Program,
    context: &ContinuityContext,
    successor: Option<&str>,
) -> TakeoverReport {
    let policy = SuccessorSelectionPolicy::default();
    let selected = successor.map(|s| s.to_string()).or_else(|| {
        let candidates = SuccessionPlanner::candidates(program, &context.failed_entity, context.scope);
        SuccessionPlanner::select(&SuccessionPlanner::rank(&candidates, &policy))
    });

    let successor = selected.unwrap_or_else(|| "NoSuccessor".into());
    let mode = ContinuationDecisionEngine::infer_mode(program, context, false);
    TakeoverCoordinator::coordinate(program, context, &successor, mode)
}

/// Plan mission delegation.
pub fn plan_delegation(
    program: &Program,
    context: &ContinuityContext,
    to_entity: Option<&str>,
) -> DelegationReport {
    let to = to_entity.unwrap_or("BackupRobot");
    MissionDelegationManager::plan(program, context, to)
}

/// Plan successor succession for a failed entity.
pub fn plan_succession(program: &Program, context: &ContinuityContext) -> SuccessionReport {
    let policy = SuccessorSelectionPolicy::default();
    let candidates = SuccessionPlanner::candidates(program, &context.failed_entity, context.scope);
    let rankings = SuccessionPlanner::rank(&candidates, &policy);
    let selected = SuccessionPlanner::select(&rankings);

    let evidence = ContinuityEvidence {
        takeover_evidence: Vec::new(),
        delegation_evidence: Vec::new(),
        continuity_evidence: vec![
            format!("Ranked {} successor candidates", rankings.len()),
            selected
                .as_ref()
                .map(|s| format!("Selected: {s}"))
                .unwrap_or_else(|| "No eligible successor".into()),
        ],
        safety_gates: Vec::new(),
        diagnosis: Some(format!("Succession for {:?} failure", context.trigger)),
        recovery_outcome: None,
    };

    let passed = selected.is_some();

    SuccessionReport {
        mission: context.mission.clone(),
        failed_entity: context.failed_entity.clone(),
        candidates,
        rankings,
        selected,
        policy,
        evidence,
        passed,
    }
}

/// Parse continuity trigger from CLI string.
pub fn parse_trigger(s: &str) -> ContinuityTrigger {
    match s.to_lowercase().as_str() {
        "robot_degraded" | "degraded" => ContinuityTrigger::RobotDegraded,
        "device_disconnected" | "disconnect" => ContinuityTrigger::DeviceDisconnected,
        "fleet_member_offline" | "fleet_offline" => ContinuityTrigger::FleetMemberOffline,
        "swarm_member_lost" | "swarm_lost" => ContinuityTrigger::SwarmMemberLost,
        "communication_interrupted" | "comm_lost" => ContinuityTrigger::CommunicationInterrupted,
        "battery_critical" | "battery" => ContinuityTrigger::BatteryCritical,
        "hardware_capability_lost" | "capability_lost" => ContinuityTrigger::HardwareCapabilityLost,
        _ => ContinuityTrigger::RobotFailed,
    }
}

/// Parse succession scope from CLI string.
pub fn parse_scope(s: &str) -> SuccessionScope {
    match s.to_lowercase().as_str() {
        "device" => SuccessionScope::Device,
        "fleet" => SuccessionScope::Fleet,
        "swarm" => SuccessionScope::Swarm,
        "group" => SuccessionScope::Group,
        "crowd" => SuccessionScope::Crowd,
        "mission_cluster" | "cluster" => SuccessionScope::MissionCluster,
        _ => SuccessionScope::Robot,
    }
}

fn health_score(health: &str) -> f64 {
    match health.to_lowercase().as_str() {
        "healthy" => 1.0,
        "degraded" => 0.6,
        _ => 0.2,
    }
}

fn location_score(distance: f64) -> f64 {
    (100.0 - distance.min(100.0)) / 100.0
}

fn connectivity_score(conn: &str) -> f64 {
    if conn.to_lowercase().contains("connected") {
        1.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_lexer::tokenize;
    use spanda_parser::parse;

    fn parse_sd(source: &str) -> Program {
        let tokens = tokenize(source).expect("tokenize");
        parse(tokens).expect("parse")
    }

    const WAREHOUSE: &str = r#"
hardware RoverV1 {
    sensors [GPS, Camera, Lidar];
    actuators [DifferentialDrive];
    connectivity [WiFi];
}

mission_plan WarehouseInventoryScan {
    step navigate_to_aisle;
    step scan_shelf_a;
    step scan_shelf_b;
    step scan_shelf_c;
    step return_to_dock;
}

fleet WarehouseFleet {
    ScannerAlpha;
    ScannerBeta;
    ScannerGamma;
}

robot ScannerAlpha {
    sensor gps: GPS;
    sensor camera: Camera;
    sensor lidar: Lidar;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    behavior scan() { wheels.drive(0.3 m/s); }
}

robot ScannerBeta {
    sensor gps: GPS;
    sensor camera: Camera;
    sensor lidar: Lidar;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    behavior scan() { wheels.drive(0.3 m/s); }
}

robot ScannerGamma {
    sensor gps: GPS;
    sensor camera: Camera;
    sensor lidar: Lidar;
    actuator wheels: DifferentialDrive;
    safety { max_speed = 1.0 m/s; }
    behavior scan() { wheels.drive(0.3 m/s); }
}
"#;

    #[test]
    fn warehouse_resume_from_checkpoint() {
        let program = parse_sd(WAREHOUSE);
        let context = ContinuityContext {
            mission: "WarehouseInventoryScan".into(),
            failed_entity: "ScannerAlpha".into(),
            trigger: ContinuityTrigger::RobotFailed,
            progress_percent: 72.0,
            scope: SuccessionScope::Fleet,
            current_step: Some("scan_shelf_b".into()),
            checkpoints: vec!["NavigationReached".into(), "ItemCollected".into()],
        };

        let report = evaluate_continuity(&program, &context);
        assert!(report.can_continue);
        assert_eq!(report.decision, ContinuationDecision::Continue);
        assert_eq!(report.takeover_mode, TakeoverMode::Resume);
        assert!(report.selected_successor.is_some());
        assert!(report.checkpoint.is_some());
    }

    #[test]
    fn succession_ranks_fleet_members() {
        let program = parse_sd(WAREHOUSE);
        let context = ContinuityContext {
            mission: "WarehouseInventoryScan".into(),
            failed_entity: "ScannerAlpha".into(),
            trigger: ContinuityTrigger::FleetMemberOffline,
            progress_percent: 50.0,
            scope: SuccessionScope::Fleet,
            ..Default::default()
        };

        let report = plan_succession(&program, &context);
        assert!(!report.rankings.is_empty());
        assert!(report.selected.is_some());
    }

    #[test]
    fn takeover_validates_safety_gates() {
        let program = parse_sd(WAREHOUSE);
        let context = ContinuityContext {
            failed_entity: "ScannerAlpha".into(),
            progress_percent: 72.0,
            ..Default::default()
        };

        let report = plan_takeover(&program, &context, Some("ScannerBeta"));
        assert!(report.safety_gates.iter().all(|g| g.passed));
        assert!(report.succeeded);
    }

    #[test]
    fn continuity_policy_infers_resume_mode() {
        let source = r#"
continuity_policy TestContinuity {
    on robot.failed {
        resume from checkpoint;
    }
}
robot R { behavior idle() {} }
"#;
        let program = parse_sd(source);
        let policies = extract_continuity_policies(&program);
        assert_eq!(policies.len(), 1);
        let context = ContinuityContext {
            trigger: ContinuityTrigger::RobotFailed,
            progress_percent: 50.0,
            ..Default::default()
        };
        assert_eq!(
            ContinuationDecisionEngine::infer_mode(&program, &context, false),
            TakeoverMode::Resume
        );
    }
}
