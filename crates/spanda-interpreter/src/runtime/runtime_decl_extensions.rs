//! AST accessor helpers, task scheduling metadata, and declaration extension traits.

use spanda_ast::foundations::{TaskDecl, TaskPriority, TriggerKind};
use spanda_ast::nodes::{BehaviorDecl, Expr, RobotDecl, SafetyRule, SafetyZoneDecl, Stmt};

type BehaviorContracts = (Vec<Stmt>, Option<Expr>, Option<Expr>, Option<Expr>);
type TaskContracts = (Vec<Stmt>, f64, Option<Expr>, Option<Expr>, Option<Expr>);

pub(super) struct TaskSchedule {
    pub(super) name: String,
    pub(super) priority: TaskPriority,
    pub(super) interval_ms: f64,
    pub(super) deadline_ms: Option<f64>,
    pub(super) jitter_ms_max: Option<f64>,
    pub(super) isolated: bool,
    pub(super) next_due_ms: f64,
    pub(super) last_start_ms: Option<f64>,
    pub(super) body: Vec<Stmt>,
    pub(super) requires: Option<Expr>,
    pub(super) ensures: Option<Expr>,
    pub(super) invariant: Option<Expr>,
    pub(super) budget: Option<spanda_ast::foundations::ResourceBudgetDecl>,
}

pub(super) const RUNTIME_TASK_COST_MS: f64 = 5.0;

pub(super) fn task_budget_violation_kind(
    budget: &spanda_ast::foundations::ResourceBudgetDecl,
    duration_ms: f64,
    interval_ms: f64,
) -> Option<&'static str> {
    // Classify which resource budget limit a task execution violated.
    //
    // Parameters:
    // - `budget` — declared task resource limits
    // - `duration_ms` — measured execution time
    // - `interval_ms` — task scheduling interval
    //
    // Returns:
    // `"cpu"`, `"battery"`, or none when no limit was exceeded.
    //
    // Options:
    // None.
    //
    // Example:
    // let kind = task_budget_violation_kind(&budget, 12.0, 100.0);

    let spanda_ast::foundations::ResourceBudgetDecl::ResourceBudgetDecl {
        cpu_pct_max,
        battery_pct_max,
        ..
    } = budget;
    let interval = interval_ms.max(1.0);
    let duty_pct = (duration_ms / interval) * 100.0;

    if let Some(cpu_max) = cpu_pct_max {
        if duty_pct > *cpu_max {
            return Some("cpu");
        }
    }

    if let Some(bat_max) = battery_pct_max {
        if duty_pct > *bat_max {
            return Some("battery");
        }
    }
    None
}

impl TaskSchedule {
    pub(super) fn priority_rank(&self) -> u8 {
        // Rank scheduled tasks for preemption ordering.
        //
        // Parameters:
        // - `self` — task schedule metadata
        //
        // Returns:
        // Lower values indicate higher preemption priority.
        //
        // Options:
        // None.
        //
        // Example:
        // let rank = schedule.priority_rank();

        let isolation_rank = if self.isolated { 0 } else { 1 };
        let priority_rank = match self.priority {
            TaskPriority::Critical => 0,
            TaskPriority::High => 1,
            TaskPriority::Normal => 2,
            TaskPriority::Low => 3,
        };
        isolation_rank * 10 + priority_rank
    }
}

pub(super) fn priority_label(priority: TaskPriority) -> &'static str {
    // Map a task priority enum to its diagnostic label.
    //
    // Parameters:
    // - `priority` — task priority tier
    //
    // Returns:
    // Stable lowercase label for logs and telemetry.
    //
    // Options:
    // None.
    //
    // Example:
    // let label = priority_label(TaskPriority::High);

    match priority {
        TaskPriority::Critical => "critical",
        TaskPriority::High => "high",
        TaskPriority::Normal => "normal",
        TaskPriority::Low => "low",
    }
}

pub(super) fn trigger_category_label(kind: &TriggerKind) -> &'static str {
    // Map a trigger kind to its diagnostic category label.
    //
    // Parameters:
    // - `kind` — parsed trigger variant
    //
    // Returns:
    // Stable lowercase label for logs and telemetry.
    //
    // Options:
    // None.
    //
    // Example:
    // let label = trigger_category_label(&trigger.kind);

    match kind {
        TriggerKind::Event { .. } => "event",
        TriggerKind::Message { .. } => "message",
        TriggerKind::Timer { .. } => "timer",
        TriggerKind::Condition { .. } => "condition",
        TriggerKind::StateEntered { .. } => "state_entered",
        TriggerKind::StateExited { .. } => "state_exited",
        TriggerKind::Safety { .. } => "safety",
        TriggerKind::Hardware { .. } => "hardware",
        TriggerKind::Ai { .. } => "ai",
        TriggerKind::Verification { .. } => "verification",
        TriggerKind::Twin { .. } => "twin",
        TriggerKind::LogMatch { .. } => "log_match",
        TriggerKind::MessageMatch { .. } => "message_match",
        TriggerKind::Connectivity { .. } => "connectivity",
        TriggerKind::Geofence { .. } => "geofence",
        TriggerKind::SensorEvent { .. } => "sensor_event",
        TriggerKind::KillSwitch { .. } => "kill_switch",
    }
}

pub(super) trait RobotDeclExt {
    fn first_behavior_name(&self) -> Option<String>;
    fn behavior_with_contracts(&self, name: &str) -> Option<BehaviorContracts>;
    fn task_with_contracts(&self, name: &str) -> Option<TaskContracts>;
    fn all_task_schedules(&self) -> Vec<TaskSchedule>;
}

impl RobotDeclExt for RobotDecl {
    fn first_behavior_name(&self) -> Option<String> {
        // Return the first behavior or task name declared on a robot.
        //
        // Parameters:
        // - `self` — robot declaration
        //
        // Returns:
        // First behavior name, or the first task name when no behavior exists.
        //
        // Options:
        // None.
        //
        // Example:
        // let name = robot.first_behavior_name();

        let RobotDecl::RobotDecl {
            behaviors, tasks, ..
        } = self;

        if let Some(b) = behaviors.first() {
            return match b {
                BehaviorDecl::BehaviorDecl { name, .. } => Some(name.clone()),
            };
        }
        tasks.first().map(|t| match t {
            TaskDecl::TaskDecl { name, .. } => name.clone(),
        })
    }

    fn behavior_with_contracts(&self, name: &str) -> Option<BehaviorContracts> {
        // Look up a behavior body and contract clauses by name.
        //
        // Parameters:
        // - `self` — robot declaration
        // - `name` — behavior identifier
        //
        // Returns:
        // Behavior body plus optional requires/ensures/invariant clauses.
        //
        // Options:
        // None.
        //
        // Example:
        // let contracts = robot.behavior_with_contracts("run");

        let RobotDecl::RobotDecl { behaviors, .. } = self;
        behaviors.iter().find_map(|b| match b {
            BehaviorDecl::BehaviorDecl {
                name: n,
                requires,
                ensures,
                invariant,
                body,
                ..
            } if n == name => Some((
                body.clone(),
                requires.clone(),
                ensures.clone(),
                invariant.clone(),
            )),
            _ => None,
        })
    }

    fn task_with_contracts(&self, name: &str) -> Option<TaskContracts> {
        // Look up a task body and contract clauses by name.
        //
        // Parameters:
        // - `self` — robot declaration
        // - `name` — task identifier
        //
        // Returns:
        // Task body, interval, and optional contract clauses.
        //
        // Options:
        // None.
        //
        // Example:
        // let contracts = robot.task_with_contracts("poll");

        let RobotDecl::RobotDecl { tasks, .. } = self;
        tasks.iter().find_map(|t| match t {
            TaskDecl::TaskDecl {
                name: n,
                priority: _priority,
                interval_ms,
                requires,
                ensures,
                invariant,
                body,
                ..
            } if n == name => Some((
                body.clone(),
                *interval_ms,
                requires.clone(),
                ensures.clone(),
                invariant.clone(),
            )),
            _ => None,
        })
    }

    fn all_task_schedules(&self) -> Vec<TaskSchedule> {
        // Build runtime task schedules from robot task declarations.
        //
        // Parameters:
        // - `self` — robot declaration
        //
        // Returns:
        // Scheduler-ready metadata for every declared task.
        //
        // Options:
        // None.
        //
        // Example:
        // let schedules = robot.all_task_schedules();

        let RobotDecl::RobotDecl { tasks, .. } = self;
        tasks
            .iter()
            .map(|t| match t {
                TaskDecl::TaskDecl {
                    name,
                    priority,
                    interval_ms,
                    deadline_ms,
                    jitter_ms_max,
                    isolated,
                    requires,
                    ensures,
                    invariant,
                    budget,
                    body,
                    ..
                } => TaskSchedule {
                    name: name.clone(),
                    priority: *priority,
                    interval_ms: *interval_ms,
                    deadline_ms: *deadline_ms,
                    jitter_ms_max: *jitter_ms_max,
                    isolated: *isolated,
                    next_due_ms: 0.0,
                    last_start_ms: None,
                    body: body.clone(),
                    requires: requires.clone(),
                    ensures: ensures.clone(),
                    invariant: invariant.clone(),
                    budget: budget.clone(),
                },
            })
            .collect()
    }
}

pub(super) trait SocDeclExt {
    fn profile(&self) -> &str;
}

impl SocDeclExt for spanda_ast::nodes::SocDecl {
    fn profile(&self) -> &str {
        // Return the profile name from a SoC declaration.
        //
        // Parameters:
        // - `self` — SoC declaration
        //
        // Returns:
        // Profile identifier string.
        //
        // Options:
        // None.
        //
        // Example:
        // let profile = soc.profile();

        match self {
            spanda_ast::nodes::SocDecl::SocDecl { profile, .. } => profile,
        }
    }
}

pub(super) trait HalBlockExt {
    fn members(&self) -> &[spanda_ast::nodes::HalMemberDecl];
}

impl HalBlockExt for spanda_ast::nodes::HalBlock {
    fn members(&self) -> &[spanda_ast::nodes::HalMemberDecl] {
        // Return HAL member declarations from a HAL block.
        //
        // Parameters:
        // - `self` — HAL block declaration
        //
        // Returns:
        // Slice of declared HAL members.
        //
        // Options:
        // None.
        //
        // Example:
        // let members = hal.members();

        match self {
            spanda_ast::nodes::HalBlock::HalBlock { members, .. } => members,
        }
    }
}

pub(super) trait SafetyBlockExt {
    fn rules(&self) -> &[SafetyRule];
    fn zones(&self) -> &[SafetyZoneDecl];
}

impl SafetyBlockExt for spanda_ast::nodes::SafetyBlock {
    fn rules(&self) -> &[SafetyRule] {
        // Return safety rules declared in a safety block.
        //
        // Parameters:
        // - `self` — safety block declaration
        //
        // Returns:
        // Slice of safety rules.
        //
        // Options:
        // None.
        //
        // Example:
        // let rules = safety.rules();

        match self {
            spanda_ast::nodes::SafetyBlock::SafetyBlock { rules, .. } => rules,
        }
    }

    fn zones(&self) -> &[SafetyZoneDecl] {
        // Return safety zones declared in a safety block.
        //
        // Parameters:
        // - `self` — safety block declaration
        //
        // Returns:
        // Slice of safety zones.
        //
        // Options:
        // None.
        //
        // Example:
        // let zones = safety.zones();

        match self {
            spanda_ast::nodes::SafetyBlock::SafetyBlock { zones, .. } => zones,
        }
    }
}
