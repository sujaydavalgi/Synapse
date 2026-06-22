//! Twin shadow sync, trigger registration, and state-machine enter hooks.
//!

use super::{
    priority_label, trigger_category_label, IntoSpandaError, Interpreter, RobotBackend,
    RuntimeError, RuntimeValue,
};
use crate::error::SpandaError;
use spanda_ast::foundations::{TriggerHandlerDecl, TriggerKind};
use spanda_runtime::triggers::{priority_rank, trigger_display_name, SystemTriggerCategory};
use crate::twin::TwinRuntime;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn update_twin_snapshot(&mut self) {
        // Update twin snapshot.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.update_twin_snapshot();

        // Compute divergence threshold for the following logic.
        let divergence_threshold = 0.15;

        // Emit output when twin provides a twin.
        if let Some(twin) = &self.twin {
            let state = self.backend.get_state();
            let live = TwinRuntime::live_mirrored_fields(
                (
                    state.pose.x,
                    state.pose.y,
                    state.pose.theta,
                    state.pose.z.unwrap_or(0.0),
                ),
                (state.velocity.linear, state.velocity.angular),
                &twin.mirrors,
            );

            // Take this path when twin.detect divergence(&live, divergence threshold).
            if twin.detect_divergence(&live, divergence_threshold) {
                let _ =
                    self.dispatch_system_trigger(SystemTriggerCategory::Twin, "DivergenceDetected");
            }
        }
        self.refresh_twin_shadow_from_backend();
        let Some(twin) = &mut self.twin else {
            return;
        };
        twin.commit_frame();
        let twin_name = twin.name.clone();
        let field_count = twin.shadow.len();
        let replay_frames = twin.replay_frame_count();

        // Take this path when field count > 0 || twin.telemetry sync.
        if field_count > 0 || twin.telemetry_sync {
            self.log(format!(
                "twin {twin_name} mirrored {field_count} field(s), replay frames={replay_frames}"
            ));
        }
    }

    pub(super) fn refresh_twin_shadow_from_backend(&mut self) {
        // Refresh twin shadow from backend.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.refresh_twin_shadow_from_backend();

        // Compute Some for the following logic.
        let Some(twin) = &mut self.twin else {
            return;
        };
        let state = self.backend.get_state();

        // Take the branch when any equals "pose").
        if twin.mirrors.iter().any(|m| m == "pose") {
            twin.snapshot(
                "pose",
                RuntimeValue::Pose {
                    x: state.pose.x,
                    y: state.pose.y,
                    theta: state.pose.theta,
                    z: state.pose.z.unwrap_or(0.0),
                },
            );
        }

        // Take the branch when any equals "velocity").
        if twin.mirrors.iter().any(|m| m == "velocity") {
            twin.snapshot(
                "velocity",
                RuntimeValue::Velocity {
                    linear: state.velocity.linear,
                    angular: state.velocity.angular,
                },
            );
        }
    }

    pub(super) fn has_standalone_triggers(&self) -> bool {
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.has_standalone_triggers();

        // skip further work when trigger timers is empty.
        if !self.trigger_timers.is_empty() {
            return true;
        }
        self.trigger_registry
            .condition_handlers()
            .iter()
            .any(|h| matches!(h.kind, TriggerKind::Condition { level: true, .. }))
    }

    pub(super) fn register_trigger_decl(&mut self, trigger: &TriggerHandlerDecl, agent: Option<String>) {
        // Register trigger decl.
        //
        // Parameters:
        // - `self` — method receiver
        // - `trigger` — input value
        // - `agent` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.register_trigger_decl(trigger, agent);

        // Compute TriggerHandlerDecl for the following logic.
        let TriggerHandlerDecl::TriggerHandlerDecl {
            trigger_kind,
            priority,
            body,
            span,
        } = trigger;
        let final_kind = if let TriggerKind::Event { name } = trigger_kind {
            // Check membership before continuing.
            if self.declared_topic_names.contains(name) && !self.declared_event_names.contains(name)
            {
                TriggerKind::Message {
                    topic: name.clone(),
                }
            } else {
                (*trigger_kind).clone()
            }
        } else {
            (*trigger_kind).clone()
        };
        let decl = TriggerHandlerDecl::TriggerHandlerDecl {
            trigger_kind: final_kind.clone(),
            priority: *priority,
            body: body.clone(),
            span: *span,
        };
        let name = trigger_display_name(&final_kind, agent.as_deref());
        self.trigger_registry.register(&decl, agent);
        self.log(format!(
            "trigger registered: {name} priority={}",
            priority_label(*priority)
        ));
    }

    pub(super) fn can_dispatch_trigger(&mut self) -> bool {
        // Can dispatch trigger.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.can_dispatch_trigger();

        // Call max triggers per tick on the current instance.
        self.triggers_dispatched_this_tick < self.options.max_triggers_per_tick
    }

    pub(super) fn execute_trigger_handlers(&mut self, handler_ids: Vec<usize>) -> Result<(), SpandaError> {
        // Execute trigger handlers.
        //
        // Parameters:
        // - `self` — method receiver
        // - `handler_ids` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_trigger_handlers(handler_ids);

        // Create mutable ids for accumulating results.
        let mut ids = handler_ids;
        ids.sort_by_key(|id| {
            self.trigger_registry
                .get(*id)
                .map(|h| priority_rank(h.priority))
                .unwrap_or(u8::MAX)
        });

        // Iterate over ids.
        for id in ids {
            // Take the branch when execute trigger body by id is false.
            if !self.execute_trigger_body_by_id(id)? {
                break;
            }

            // Take this path when self.
            if self
                .safety_monitor
                .as_ref()
                .map(|m| m.is_emergency_stop())
                .unwrap_or(false)
            {
                break;
            }
        }
        Ok(())
    }

    pub(super) fn execute_trigger_body_by_id(&mut self, handler_id: usize) -> Result<bool, SpandaError> {
        // Execute trigger body by id.
        //
        // Parameters:
        // - `self` — method receiver
        // - `handler_id` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_trigger_body_by_id(handler_id);

        // Bind a local value for the next steps.
        let (name, kind, priority, body, agent) = {
            let handler = self
                .trigger_registry
                .get(handler_id)
                .ok_or_else(|| RuntimeError::new("unknown trigger handler", 0).into_spanda())?;
            (
                handler.name.clone(),
                handler.kind.clone(),
                handler.priority,
                handler.body.clone(),
                handler.agent.clone(),
            )
        };

        // Take the branch when can dispatch trigger is false.
        if !self.can_dispatch_trigger() {
            self.trace_trigger_log(format!("{name} suppressed (trigger storm limit)"));
            return Ok(false);
        }
        self.triggers_dispatched_this_tick += 1;
        let start = std::time::Instant::now();
        let saved_agent = self.current_agent.clone();

        // Emit output when agent provides a agent.
        if let Some(agent) = &agent {
            self.current_agent = Some(agent.clone());
        }
        let category = trigger_category_label(&kind);
        self.trace_trigger_log(format!(
            "dispatch {name} priority={} category={category}",
            priority_label(priority)
        ));

        // Keep entries that match the expected pattern.
        if matches!(kind, TriggerKind::Event { .. }) {
            self.trace_event_log(format!("dispatch {name}"));
        }
        let result = self.execute_block(&body);
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        let failed = result.is_err();
        self.telemetry
            .record_trigger_execution(&name, category, priority, duration_ms, failed);
        self.current_agent = saved_agent;
        result?;
        Ok(true)
    }

    pub(super) fn dispatch_event(&mut self, event_name: &str) -> Result<(), SpandaError> {
        //
        // Parameters:
        // - `self` — method receiver
        // - `event_name` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.dispatch_event(event_name);

        // Compute ids for the following logic.
        let ids: Vec<usize> = self
            .trigger_registry
            .handlers_for_event(event_name)
            .iter()
            .map(|h| h.id)
            .collect();

        // Skip further work when !ids is empty.
        if !ids.is_empty() {
            self.trace_event_log(format!("emit {event_name}"));
            self.log(format!("emit {event_name}"));
            return self.execute_trigger_handlers(ids);
        }

        // Emit output when to vec provides a body.
        if let Some(body) = self.event_bus.handler_body(event_name).map(|b| b.to_vec()) {
            self.trace_event_log(format!("emit {event_name} (legacy)"));
            self.log(format!("emit {event_name}"));
            self.execute_block(&body)?;
        } else {
            self.log(format!("emit {event_name} (no handler)"));
        }
        Ok(())
    }

    pub(super) fn execute_enter(&mut self, state_name: &str, line: u32) -> Result<(), SpandaError> {
        // Execute enter.
        //
        // Parameters:
        // - `self` — method receiver
        // - `state_name` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_enter(state_name, line);

        // Create mutable logs for accumulating results.
        let mut logs = Vec::new();
        let mut transitioned = false;
        let mut previous_states = Vec::new();

        // Iterate over state machines with destructured elements.
        for (sm_name, sm) in &mut self.state_machines {
            // Emit output when try enter provides a previous.
            if let Some(previous) = sm.try_enter(state_name) {
                logs.push(format!(
                    "state_machine {sm_name}: {previous} -> {state_name}"
                ));
                previous_states.push(previous);
                transitioned = true;
            }
        }

        // Process each log.
        for msg in logs {
            self.log(msg);
        }

        // Take the branch when transitioned is false.
        if !transitioned {
            return Err(RuntimeError::new(
                format!("No valid transition to state '{state_name}'"),
                line,
            )
            .into_spanda());
        }

        // Process each previous state.
        for previous in previous_states {
            let ids: Vec<usize> = self
                .trigger_registry
                .handlers_for_state_exited(&previous)
                .iter()
                .map(|h| h.id)
                .collect();
            self.execute_trigger_handlers(ids)?;
        }
        let ids: Vec<usize> = self
            .trigger_registry
            .handlers_for_state_entered(state_name)
            .iter()
            .map(|h| h.id)
            .collect();
        self.execute_trigger_handlers(ids)?;
        Ok(())
    }

}
