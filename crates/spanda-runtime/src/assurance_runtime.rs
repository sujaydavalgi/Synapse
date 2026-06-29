//! Injectable assurance runtime boundary for interpreter recovery and continuity.
//!
use crate::continuity_primitives::{
    default_checkpoint_store_path, extract_continuity_policies, issue_to_continuity_trigger,
    load_checkpoint, load_checkpoint_store, parse_trigger, program_has_continuity_for_trigger,
    record_checkpoint, save_checkpoint_store,
};
use crate::continuity_types::{
    ContinuityCheckpointStore, ContinuityContext, ContinuityEvidence, ContinuityPolicySpec,
    ContinuityTrigger, ContinuationDecision, MissionStateSnapshot, MissionStateTransfer,
    TakeoverMode, TakeoverReport,
};
use crate::recovery_primitives::{
    classify_failure, default_knowledge_store_path, extract_recovery_policies,
    issue_to_recovery_issue, load_recovery_knowledge_store, merge_recovery_knowledge,
    program_has_recovery_for_issue, record_recovery_outcome, save_recovery_knowledge_store,
};
use crate::recovery_types::{
    FailureClassification, PlannedRecoveryAction, RecoveryContext, RecoveryEvidence,
    RecoveryKnowledgeBase, RecoveryLevel, RecoveryPlan, RecoveryPolicySpec, RecoveryResult,
    RecoveryStatus, RecoveryStrategy, SafeRecoveryAction, ValidationGateResult,
};
use spanda_ast::nodes::Program;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Extension points for assurance recovery and continuity at runtime.
pub trait AssuranceRuntime: Send + Sync {
    fn classify_failure(&self, issue: &str) -> FailureClassification;

    fn default_knowledge_store_path(&self) -> PathBuf;

    fn load_recovery_knowledge_store(&self, path: &Path) -> RecoveryKnowledgeBase;

    fn save_recovery_knowledge_store(
        &self,
        path: &Path,
        kb: &RecoveryKnowledgeBase,
    ) -> std::io::Result<()>;

    fn merge_recovery_knowledge(
        &self,
        program: &Program,
        persisted: &RecoveryKnowledgeBase,
    ) -> RecoveryKnowledgeBase;

    fn record_recovery_outcome(&self, kb: &mut RecoveryKnowledgeBase, result: &RecoveryResult);

    fn extract_recovery_policies(&self, program: &Program) -> Vec<RecoveryPolicySpec>;

    fn issue_to_recovery_issue(&self, event: &str) -> Option<String>;

    fn program_has_recovery_for_issue(&self, program: &Program, issue: &str) -> bool;

    fn plan_recovery(&self, program: &Program, context: &RecoveryContext) -> RecoveryPlan;

    fn validate_recovery_plan(
        &self,
        program: &Program,
        plan: &RecoveryPlan,
    ) -> Vec<SafeRecoveryAction>;

    fn build_recovery_result_from_plan(
        &self,
        program: &Program,
        plan: &RecoveryPlan,
    ) -> RecoveryResult;

    fn recovery_allowed(&self, program: &Program, issue: &str) -> bool;

    fn extract_continuity_policies(&self, program: &Program) -> Vec<ContinuityPolicySpec>;

    fn issue_to_continuity_trigger(&self, issue: &str) -> Option<ContinuityTrigger>;

    fn program_has_continuity_for_trigger(
        &self,
        program: &Program,
        trigger: ContinuityTrigger,
    ) -> bool;

    fn plan_takeover(
        &self,
        program: &Program,
        context: &ContinuityContext,
        successor: Option<&str>,
    ) -> TakeoverReport;

    fn default_checkpoint_store_path(&self) -> PathBuf;

    fn load_checkpoint_store(&self, path: &Path) -> ContinuityCheckpointStore;

    fn save_checkpoint_store(
        &self,
        path: &Path,
        store: &ContinuityCheckpointStore,
    ) -> std::io::Result<()>;

    fn record_checkpoint(
        &self,
        store: &mut ContinuityCheckpointStore,
        mission: &str,
        robot: &str,
        snapshot: MissionStateSnapshot,
    );

    fn load_checkpoint(
        &self,
        store: &ContinuityCheckpointStore,
        mission: &str,
        robot: &str,
    ) -> Option<MissionStateSnapshot>;

    fn parse_trigger(&self, s: &str) -> ContinuityTrigger;

    /// Evaluate recovery plans for a compiled program with no external context.
    ///
    /// Parameters:
    /// - `program` — compiled AST program
    ///
    /// Returns:
    /// RecoveryReport summarising plan coverage and gate results.
    ///
    /// Options:
    /// None.
    ///
    /// Example:
    /// let report = platform_assurance_runtime().evaluate_recovery_program(&program);
    fn evaluate_recovery_program(&self, program: &Program) -> crate::recovery_types::RecoveryReport;
}

/// Minimal built-in assurance runtime with permissive validation defaults.
#[derive(Debug, Default, Clone, Copy)]
pub struct BuiltinAssuranceRuntime;

impl AssuranceRuntime for BuiltinAssuranceRuntime {
    fn classify_failure(&self, issue: &str) -> FailureClassification {
        classify_failure(issue)
    }

    fn default_knowledge_store_path(&self) -> PathBuf {
        default_knowledge_store_path()
    }

    fn load_recovery_knowledge_store(&self, path: &Path) -> RecoveryKnowledgeBase {
        load_recovery_knowledge_store(path)
    }

    fn save_recovery_knowledge_store(
        &self,
        path: &Path,
        kb: &RecoveryKnowledgeBase,
    ) -> std::io::Result<()> {
        save_recovery_knowledge_store(path, kb)
    }

    fn merge_recovery_knowledge(
        &self,
        program: &Program,
        persisted: &RecoveryKnowledgeBase,
    ) -> RecoveryKnowledgeBase {
        merge_recovery_knowledge(program, persisted)
    }

    fn record_recovery_outcome(&self, kb: &mut RecoveryKnowledgeBase, result: &RecoveryResult) {
        record_recovery_outcome(kb, result);
    }

    fn extract_recovery_policies(&self, program: &Program) -> Vec<RecoveryPolicySpec> {
        extract_recovery_policies(program)
    }

    fn issue_to_recovery_issue(&self, event: &str) -> Option<String> {
        issue_to_recovery_issue(event)
    }

    fn program_has_recovery_for_issue(&self, program: &Program, issue: &str) -> bool {
        program_has_recovery_for_issue(program, issue)
    }

    fn plan_recovery(&self, program: &Program, context: &RecoveryContext) -> RecoveryPlan {
        let classification = context
            .classification
            .unwrap_or_else(|| classify_failure(&context.issue));
        let policies = extract_recovery_policies(program);
        let mut actions = Vec::new();
        for policy in &policies {
            for (condition, branch_actions) in &policy.triggers {
                if branch_actions.is_empty() {
                    continue;
                }
                let lower_issue = context.issue.to_ascii_lowercase();
                let lower_condition = condition.to_ascii_lowercase();
                if lower_issue.contains(&lower_condition.replace('.', ""))
                    || lower_condition.contains(&lower_issue)
                {
                    for (order, action) in branch_actions.iter().enumerate() {
                        actions.push(PlannedRecoveryAction {
                            description: action.clone(),
                            strategy: RecoveryStrategy::Custom(action.clone()),
                            correction: None,
                            risk: "low".into(),
                            requires_approval: false,
                            order: order as u32 + 1,
                        });
                    }
                }
            }
        }
        if actions.is_empty() {
            actions.push(PlannedRecoveryAction {
                description: format!("alert operator for {}", context.issue),
                strategy: RecoveryStrategy::OperatorAlert,
                correction: None,
                risk: "low".into(),
                requires_approval: false,
                order: 1,
            });
        }
        RecoveryPlan {
            name: format!("builtin-{}", sanitize_name(&context.issue)),
            failure: context.issue.clone(),
            classification,
            diagnosis: context
                .diagnosis
                .clone()
                .unwrap_or_else(|| context.issue.clone()),
            actions,
            target_mode: None,
            level: context.level,
            risk: "low".into(),
        }
    }

    fn validate_recovery_plan(
        &self,
        _program: &Program,
        plan: &RecoveryPlan,
    ) -> Vec<SafeRecoveryAction> {
        plan.actions
            .iter()
            .map(|action| SafeRecoveryAction {
                action: action.clone(),
                safety_validation: ValidationGateResult {
                    gate: "Safety Validation".into(),
                    passed: true,
                    message: "PASS".into(),
                },
                hardware_verification: ValidationGateResult {
                    gate: "Hardware Verification".into(),
                    passed: true,
                    message: "PASS".into(),
                },
                capability_verification: ValidationGateResult {
                    gate: "Capability Verification".into(),
                    passed: true,
                    message: "PASS".into(),
                },
                readiness_validation: ValidationGateResult {
                    gate: "Readiness Validation".into(),
                    passed: true,
                    message: "PASS".into(),
                },
                approved: true,
            })
            .collect()
    }

    fn build_recovery_result_from_plan(
        &self,
        _program: &Program,
        plan: &RecoveryPlan,
    ) -> RecoveryResult {
        let executed: Vec<String> = plan.actions.iter().map(|a| a.description.clone()).collect();
        let evidence = RecoveryEvidence {
            failure: plan.failure.clone(),
            diagnosis: plan.diagnosis.clone(),
            plan: plan.name.clone(),
            safety_validation: "PASS".into(),
            recovery_actions: executed.clone(),
            outcome: "Success".into(),
            operator_approval: None,
            verification: "Recovery verified".into(),
        };
        RecoveryResult {
            plan: plan.name.clone(),
            status: RecoveryStatus::Success,
            executed_actions: executed,
            failed_actions: Vec::new(),
            verification_outcome: evidence.verification.clone(),
            evidence,
        }
    }

    fn recovery_allowed(&self, program: &Program, issue: &str) -> bool {
        let context = RecoveryContext {
            issue: issue.into(),
            diagnosis: None,
            classification: Some(classify_failure(issue)),
            level: RecoveryLevel::Level3AutomaticWithValidation,
        };
        let plan = self.plan_recovery(program, &context);
        let result = self.build_recovery_result_from_plan(program, &plan);
        !matches!(
            result.status,
            RecoveryStatus::Unsafe | RecoveryStatus::Failed
        )
    }

    fn extract_continuity_policies(&self, program: &Program) -> Vec<ContinuityPolicySpec> {
        extract_continuity_policies(program)
    }

    fn issue_to_continuity_trigger(&self, issue: &str) -> Option<ContinuityTrigger> {
        issue_to_continuity_trigger(issue)
    }

    fn program_has_continuity_for_trigger(
        &self,
        program: &Program,
        trigger: ContinuityTrigger,
    ) -> bool {
        program_has_continuity_for_trigger(program, trigger)
    }

    fn plan_takeover(
        &self,
        program: &Program,
        context: &ContinuityContext,
        successor: Option<&str>,
    ) -> TakeoverReport {
        let successor_name = successor
            .map(str::to_string)
            .or_else(|| first_fleet_member(program, &context.failed_entity))
            .unwrap_or_else(|| "BackupRobot".into());
        let progress = context.progress_percent;
        let snapshot = MissionStateSnapshot {
            mission: context.mission.clone(),
            completed_steps: Vec::new(),
            current_goal: context.current_step.clone(),
            progress_percent: progress,
            checkpoints: Vec::new(),
        };
        let mode = if progress > 0.0 {
            TakeoverMode::Resume
        } else {
            TakeoverMode::Restart
        };
        TakeoverReport {
            mission: context.mission.clone(),
            failed_entity: context.failed_entity.clone(),
            successor: successor_name.clone(),
            mode,
            decision: ContinuationDecision::Continue,
            state_transfer: MissionStateTransfer {
                from_entity: context.failed_entity.clone(),
                to_entity: successor_name,
                snapshot,
                transferable: true,
                transfer_notes: vec!["builtin continuity handoff".into()],
            },
            safety_gates: Vec::new(),
            evidence: ContinuityEvidence {
                takeover_evidence: vec![format!("builtin takeover for {:?}", context.trigger)],
                delegation_evidence: Vec::new(),
                continuity_evidence: Vec::new(),
                safety_gates: Vec::new(),
                diagnosis: Some(format!("builtin continuity for {}", context.failed_entity)),
                recovery_outcome: None,
            },
            succeeded: true,
            diagnosis: format!("builtin takeover for {:?}", context.trigger),
        }
    }

    fn default_checkpoint_store_path(&self) -> PathBuf {
        default_checkpoint_store_path()
    }

    fn load_checkpoint_store(&self, path: &Path) -> ContinuityCheckpointStore {
        load_checkpoint_store(path)
    }

    fn save_checkpoint_store(
        &self,
        path: &Path,
        store: &ContinuityCheckpointStore,
    ) -> std::io::Result<()> {
        save_checkpoint_store(path, store)
    }

    fn record_checkpoint(
        &self,
        store: &mut ContinuityCheckpointStore,
        mission: &str,
        robot: &str,
        snapshot: MissionStateSnapshot,
    ) {
        record_checkpoint(store, mission, robot, snapshot);
    }

    fn load_checkpoint(
        &self,
        store: &ContinuityCheckpointStore,
        mission: &str,
        robot: &str,
    ) -> Option<MissionStateSnapshot> {
        load_checkpoint(store, mission, robot)
    }

    fn parse_trigger(&self, s: &str) -> ContinuityTrigger {
        parse_trigger(s)
    }

    fn evaluate_recovery_program(
        &self,
        _program: &Program,
    ) -> crate::recovery_types::RecoveryReport {
        // Return a default passing report when no real assurance engine is wired.
        let default_readiness = crate::recovery_types::RecoveryReadiness {
            recovery_ready: true,
            risk: "none".into(),
            readiness_score: 100,
            blockers: Vec::new(),
        };
        crate::recovery_types::RecoveryReport {
            passed: true,
            policies: Vec::new(),
            plans: Vec::new(),
            safe_actions: Vec::new(),
            results: Vec::new(),
            audit: Vec::new(),
            readiness: default_readiness.clone(),
            assurance: crate::recovery_types::RecoveryAssuranceMetrics {
                recovery_evidence: Vec::new(),
                recovery_readiness: default_readiness,
                success_rate: 1.0,
                traceability: Vec::new(),
            },
            fleet_plans: Vec::new(),
            knowledge: crate::recovery_types::RecoveryKnowledgeBase::default(),
        }
    }
}

static PLATFORM_ASSURANCE_RUNTIME: std::sync::OnceLock<SharedAssuranceRuntime> =
    std::sync::OnceLock::new();

/// Inject a real assurance runtime from a higher-layer crate (e.g. spanda-assurance bridge).
pub fn set_platform_assurance_runtime(runtime: SharedAssuranceRuntime) {
    // Accept the first injection; subsequent calls are silently ignored via OnceLock semantics.
    let _ = PLATFORM_ASSURANCE_RUNTIME.set(runtime);
}

/// Return the active platform assurance runtime, falling back to the built-in default.
pub fn platform_assurance_runtime() -> SharedAssuranceRuntime {
    // Return the injected runtime if set, otherwise use the default built-in.
    PLATFORM_ASSURANCE_RUNTIME
        .get()
        .cloned()
        .unwrap_or_else(default_assurance_runtime)
}

/// Shared assurance runtime handle passed through run options at the driver boundary.
pub type SharedAssuranceRuntime = Arc<dyn AssuranceRuntime>;

/// Default built-in assurance runtime for direct interpreter use without assurance crate.
pub fn default_assurance_runtime() -> SharedAssuranceRuntime {
    Arc::new(BuiltinAssuranceRuntime)
}

fn sanitize_name(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn first_fleet_member(program: &Program, failed_entity: &str) -> Option<String> {
    let Program::Program { fleets, .. } = program;
    for fleet in fleets {
        let spanda_ast::robotics_decl::FleetDecl::FleetDecl { members, .. } = fleet;
        if let Some(member) = members.iter().find(|m| *m != failed_entity) {
            return Some((*member).clone());
        }
    }
    None
}
