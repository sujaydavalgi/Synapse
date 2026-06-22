//! Unified trigger-based execution model for Spanda autonomous systems.
//!
//! Triggers unify events, messages, timers, conditions, state transitions, safety,
//! hardware, AI, verification, and digital-twin reactive handlers under one registry.

use spanda_ast::foundations::{TaskPriority, TriggerHandlerDecl, TriggerKind};
use spanda_ast::nodes::{Span, SpandaType, Stmt};
use std::collections::{HashMap, HashSet};

/// Maximum trigger dispatches per scheduler tick (prevents trigger storms).
pub const MAX_TRIGGERS_PER_TICK: usize = 64;

/// Registered trigger handler with stable id for metrics.
#[derive(Debug, Clone)]
pub struct RegisteredTrigger {
    pub id: usize,
    pub name: String,
    pub kind: TriggerKind,
    pub priority: TaskPriority,
    pub body: Vec<Stmt>,

    /// Agent scope when declared inside an agent block.
    pub agent: Option<String>,
}

/// Unified registry for all trigger categories.
#[derive(Debug, Default)]
pub struct TriggerRegistry {
    handlers: Vec<RegisteredTrigger>,
    event_index: HashMap<String, usize>,
    next_id: usize,
}

impl TriggerRegistry {
    pub fn new() -> Self {
        // Create a new instance.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_runtime::triggers::new();

        // Build the result via default.
        Self::default()
    }

    pub fn register(&mut self, decl: &TriggerHandlerDecl, agent: Option<String>) {
        // Register the value.
        //
        // Parameters:
        // - `self` — method receiver
        // - `decl` — input value
        // - `agent` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.register(decl, agent);

        // Compute TriggerHandlerDecl for the following logic.
        let TriggerHandlerDecl::TriggerHandlerDecl {
            trigger_kind,
            priority,
            body,
            span,
            ..
        } = decl;
        let name = trigger_display_name(trigger_kind, agent.as_deref());
        let id = self.next_id;
        self.next_id += 1;

        // For event triggers, record the event name in the index.
        if let TriggerKind::Event { name: event_name } = trigger_kind {
            self.event_index.insert(event_name.clone(), id);
        }
        self.handlers.push(RegisteredTrigger {
            id,
            name,
            kind: trigger_kind.clone(),
            priority: *priority,
            body: body.clone(),
            agent,
        });
        let _ = span;
    }

    pub fn register_legacy_event(&mut self, event_name: String, body: Vec<Stmt>) {
        // Register legacy event.
        //
        // Parameters:
        // - `self` — method receiver
        // - `event_name` — input value
        // - `body` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.register_legacy_event(event_name, body);

        // Register the value handler.
        self.register(
            &TriggerHandlerDecl::TriggerHandlerDecl {
                trigger_kind: TriggerKind::Event {
                    name: event_name.clone(),
                },
                priority: TaskPriority::Normal,
                return_type: SpandaType::Void,
                body,
                span: Span::default(),
            },
            None,
        );
    }

    pub fn handler_count(&self) -> usize {
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Numeric result.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.handler_count();

        // Call len on the current instance.
        self.handlers.len()
    }

    pub fn all(&self) -> &[RegisteredTrigger] {
        // All.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // &[RegisteredTrigger].
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.all();

        // Return handlers from this handle.
        &self.handlers
    }

    pub fn get(&self, id: usize) -> Option<&RegisteredTrigger> {
        // Get.
        //
        // Parameters:
        // - `self` — method receiver
        // - `id` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.get(id);

        // Iterate over handlers.
        self.handlers.iter().find(|h| h.id == id)
    }

    pub fn event_handler_body(&self, event_name: &str) -> Option<&[Stmt]> {
        // Event handler body.
        //
        // Parameters:
        // - `self` — method receiver
        // - `event_name` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.event_handler_body(event_name);

        // Call event index on the current instance.
        self.event_index
            .get(event_name)
            .and_then(|id| self.get(*id))
            .map(|h| h.body.as_slice())
    }

    pub fn handlers_for_event(&self, event_name: &str) -> Vec<&RegisteredTrigger> {
        // Handlers for event.
        //
        // Parameters:
        // - `self` — method receiver
        // - `event_name` — input value
        //
        // Returns:
        // Vec<&RegisteredTrigger>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.handlers_for_event(event_name);

        // Call handlers on the current instance.
        self.handlers
            .iter()
            .filter(|h| matches!(&h.kind, TriggerKind::Event { name } if name == event_name))
            .collect()
    }

    pub fn handlers_for_message(
        &self,
        topic_name: &str,
        topic_path: &str,
    ) -> Vec<&RegisteredTrigger> {
        // Handlers for message.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic_name` — input value
        // - `topic_path` — input value
        //
        // Returns:
        // Vec<&RegisteredTrigger>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.handlers_for_message(topic_name, topic_path);

        // Call handlers on the current instance.
        self.handlers
            .iter()
            .filter(|h| match &h.kind {
                TriggerKind::Message { topic } => {
                    topic == topic_name
                        || topic == topic_path
                        || topic_path.ends_with(&format!("/{topic}"))
                        || format!("/{topic}") == topic_path
                }
                _ => false,
            })
            .collect()
    }

    pub fn timer_handlers(&self) -> Vec<&RegisteredTrigger> {
        // Timer handlers.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<&RegisteredTrigger>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.timer_handlers();

        // Call handlers on the current instance.
        self.handlers
            .iter()
            .filter(|h| matches!(h.kind, TriggerKind::Timer { .. }))
            .collect()
    }

    pub fn condition_handlers(&self) -> Vec<&RegisteredTrigger> {
        // Condition handlers.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<&RegisteredTrigger>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.condition_handlers();

        // Call handlers on the current instance.
        self.handlers
            .iter()
            .filter(|h| matches!(h.kind, TriggerKind::Condition { .. }))
            .collect()
    }

    pub fn handlers_for_state_entered(&self, state: &str) -> Vec<&RegisteredTrigger> {
        // Handlers for state entered.
        //
        // Parameters:
        // - `self` — method receiver
        // - `state` — input value
        //
        // Returns:
        // Vec<&RegisteredTrigger>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.handlers_for_state_entered(state);

        // Call handlers on the current instance.
        self.handlers
            .iter()
            .filter(|h| {
                matches!(
                    &h.kind,
                    TriggerKind::StateEntered { state: s } if s == state
                )
            })
            .collect()
    }

    pub fn handlers_for_state_exited(&self, state: &str) -> Vec<&RegisteredTrigger> {
        // Handlers for state exited.
        //
        // Parameters:
        // - `self` — method receiver
        // - `state` — input value
        //
        // Returns:
        // Vec<&RegisteredTrigger>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.handlers_for_state_exited(state);

        // Call handlers on the current instance.
        self.handlers
            .iter()
            .filter(|h| {
                matches!(
                    &h.kind,
                    TriggerKind::StateExited { state: s } if s == state
                )
            })
            .collect()
    }

    pub fn handlers_for_category(
        &self,
        category: SystemTriggerCategory,
        event: &str,
    ) -> Vec<&RegisteredTrigger> {
        // Handlers for category.
        //
        // Parameters:
        // - `self` — method receiver
        // - `category` — input value
        // - `event` — input value
        //
        // Returns:
        // Vec<&RegisteredTrigger>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.handlers_for_category(category, event);

        // Call handlers on the current instance.
        self.handlers
            .iter()
            .filter(|h| match (&h.kind, category) {
                (TriggerKind::Safety { event: e }, SystemTriggerCategory::Safety) => e == event,
                (TriggerKind::Hardware { event: e }, SystemTriggerCategory::Hardware) => e == event,
                (TriggerKind::Ai { event: e }, SystemTriggerCategory::Ai) => e == event,
                (TriggerKind::Verification { event: e }, SystemTriggerCategory::Verification) => {
                    e == event
                }
                (TriggerKind::Twin { event: e }, SystemTriggerCategory::Twin) => e == event,
                _ => false,
            })
            .collect()
    }

    pub fn handlers_for_connectivity(&self, domain: &str, event: &str) -> Vec<&RegisteredTrigger> {
        self.handlers
            .iter()
            .filter(|h| {
                matches!(
                    &h.kind,
                    TriggerKind::Connectivity { domain: d, event: e }
                        if d == domain && e == event
                )
            })
            .collect()
    }

    pub fn handlers_for_geofence(&self, name: &str, phase: &str) -> Vec<&RegisteredTrigger> {
        self.handlers
            .iter()
            .filter(|h| {
                matches!(
                    &h.kind,
                    TriggerKind::Geofence { name: n, phase: p }
                        if n == name && p == phase
                )
            })
            .collect()
    }

    pub fn handlers_for_sensor_event(&self, sensor: &str, event: &str) -> Vec<&RegisteredTrigger> {
        self.handlers
            .iter()
            .filter(|h| {
                matches!(
                    &h.kind,
                    TriggerKind::SensorEvent { sensor: s, event: e }
                        if s == sensor && e == event
                )
            })
            .collect()
    }

    pub fn sorted_by_priority(handlers: Vec<&RegisteredTrigger>) -> Vec<&RegisteredTrigger> {
        // Sorted by priority.
        //
        // Parameters:
        // - `handlers` — input value
        //
        // Returns:
        // Vec<&RegisteredTrigger>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_runtime::triggers::sorted_by_priority(handlers);

        // Create mutable sorted for accumulating results.
        let mut sorted = handlers;
        sorted.sort_by_key(|h| priority_rank(h.priority));
        sorted
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemTriggerCategory {
    Safety,
    Hardware,
    Verification,
    Ai,
    Twin,
}

pub fn priority_rank(priority: TaskPriority) -> u8 {
    // Priority rank.
    //
    // Parameters:
    // - `priority` — input value
    //
    // Returns:
    // u8.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_runtime::triggers::priority_rank(priority);

    // Match on priority and handle each case.
    match priority {
        TaskPriority::Critical => 0,
        TaskPriority::High => 1,
        TaskPriority::Normal => 2,
        TaskPriority::Low => 3,
    }
}

pub fn trigger_display_name(kind: &TriggerKind, agent: Option<&str>) -> String {
    // Trigger display name.
    //
    // Parameters:
    // - `kind` — input value
    // - `agent` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_runtime::triggers::trigger_display_name(kind, agent);

    // Compute base for the following logic.
    let base = match kind {
        TriggerKind::Event { name } => format!("event:{name}"),
        TriggerKind::Message { topic } => format!("message:{topic}"),
        TriggerKind::Timer { interval_ms } => format!("timer:{interval_ms}ms"),
        TriggerKind::Condition { .. } => "condition".into(),
        TriggerKind::StateEntered { state } => format!("state_entered:{state}"),
        TriggerKind::StateExited { state } => format!("state_exited:{state}"),
        TriggerKind::Safety { event } => format!("safety:{event}"),
        TriggerKind::Hardware { event } => format!("hardware:{event}"),
        TriggerKind::Ai { event } => format!("ai:{event}"),
        TriggerKind::Verification { event } => format!("verification:{event}"),
        TriggerKind::Twin { event } => format!("twin:{event}"),
        TriggerKind::LogMatch { pattern } => format!("log_match:/{}/", pattern.source),
        TriggerKind::MessageMatch { field, pattern } => {
            format!("message_match:{field}:/{}/", pattern.source)
        }
        TriggerKind::Connectivity { domain, event } => format!("connectivity:{domain}.{event}"),
        TriggerKind::Geofence { name, phase } => format!("geofence:{name}:{phase}"),
        TriggerKind::SensorEvent { sensor, event } => format!("sensor:{sensor}.{event}"),
    };

    // Emit output when agent provides a agent.
    if let Some(agent) = agent {
        format!("{agent}/{base}")
    } else {
        base
    }
}

/// Per-trigger runtime schedule state for timer triggers.
#[derive(Debug, Clone)]
pub struct TriggerTimerSchedule {
    pub trigger_id: usize,
    pub interval_ms: f64,
    pub next_due_ms: f64,
}

impl TriggerTimerSchedule {
    pub fn from_handler(handler: &RegisteredTrigger) -> Option<Self> {
        // Construct from handler.
        //
        // Parameters:
        // - `handler` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_runtime::triggers::from_handler(handler);

        // take this path when let TriggerKind::Timer { interval ms } = handler.kind.
        if let TriggerKind::Timer { interval_ms } = handler.kind {
            Some(Self {
                trigger_id: handler.id,
                interval_ms,
                next_due_ms: 0.0,
            })
        } else {
            None
        }
    }
}

/// Tracks edge state for condition triggers (fire on transition to true).
#[derive(Debug, Default)]
pub struct ConditionTriggerState {
    was_active: HashSet<usize>,
}

impl ConditionTriggerState {
    pub fn should_fire(&mut self, trigger_id: usize, active: bool) -> bool {
        // Should fire.
        //
        // Parameters:
        // - `self` — method receiver
        // - `trigger_id` — input value
        // - `active` — input value
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.should_fire(trigger_id, active);

        // Compute was for the following logic.
        let was = self.was_active.contains(&trigger_id);

        // Take this path when active.
        if active {
            self.was_active.insert(trigger_id);
            !was
        } else {
            self.was_active.remove(&trigger_id);
            false
        }
    }

    pub fn is_level_active(&self, trigger_id: usize) -> bool {
        //
        // Parameters:
        // - `self` — method receiver
        // - `trigger_id` — input value
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.is_level_active(trigger_id);

        // Call contains on the current instance.
        self.was_active.contains(&trigger_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_ast::foundations::TriggerHandlerDecl;

    #[test]
    fn registers_and_sorts_by_priority() {
        // Registers and sorts by priority.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_runtime::triggers::registers_and_sorts_by_priority();

        let mut registry = TriggerRegistry::new();
        registry.register(
            &TriggerHandlerDecl::TriggerHandlerDecl {
                trigger_kind: TriggerKind::Safety {
                    event: "EmergencyStop".into(),
                },
                priority: TaskPriority::Normal,
                return_type: SpandaType::Void,
                body: vec![],
                span: Span::default(),
            },
            None,
        );
        registry.register(
            &TriggerHandlerDecl::TriggerHandlerDecl {
                trigger_kind: TriggerKind::Safety {
                    event: "EmergencyStop".into(),
                },
                priority: TaskPriority::Critical,
                return_type: SpandaType::Void,
                body: vec![],
                span: Span::default(),
            },
            None,
        );
        let handlers =
            registry.handlers_for_category(SystemTriggerCategory::Safety, "EmergencyStop");
        let sorted = TriggerRegistry::sorted_by_priority(handlers);
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].priority, TaskPriority::Critical);
    }

    #[test]
    fn condition_edge_detection() {
        // Condition edge detection.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_runtime::triggers::condition_edge_detection();

        let mut state = ConditionTriggerState::default();
        assert!(state.should_fire(1, true));
        assert!(!state.should_fire(1, true));
        assert!(!state.should_fire(1, false));
        assert!(state.should_fire(1, true));
    }
}
