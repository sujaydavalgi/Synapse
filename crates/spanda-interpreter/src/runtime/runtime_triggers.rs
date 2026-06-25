//! Trigger dispatch, polling, and periodic maintenance for the interpreter.
//!

use super::{trigger_category_label, Interpreter, RobotBackend, RuntimeValue};
use spanda_ast::foundations::TriggerKind;
use spanda_ast::nodes::Expr;
use spanda_comm::CommBus;
use spanda_error::SpandaError;
use spanda_runtime::triggers::SystemTriggerCategory;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn dispatch_system_trigger(
        &mut self,
        category: SystemTriggerCategory,
        event: &str,
    ) -> Result<(), SpandaError> {
        // Description:
        //     Dispatch system trigger.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     category: SystemTriggerCategory
        //         Caller-supplied category.
        //     even: &str
        //         Caller-supplied even.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `dispatch_system_trigger`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_triggers::dispatch_system_trigger(&mut self, category, even);

        let ids: Vec<usize> = self
            .trigger_registry
            .handlers_for_category(category, event)
            .iter()
            .map(|h| h.id)
            .collect();
        if ids.is_empty() {
            return Ok(());
        }
        self.log(format!("system trigger: {:?}:{event}", category));
        self.execute_trigger_handlers(ids)
    }

    pub(super) fn dispatch_message_triggers(
        &mut self,
        topic_name: &str,
        topic_path: &str,
    ) -> Result<(), SpandaError> {
        // Description:
        //     Dispatch message triggers.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     opic_name: &str
        //         Caller-supplied opic name.
        //     opic_path: &str
        //         Caller-supplied opic path.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `dispatch_message_triggers`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_triggers::dispatch_message_triggers(&mut self, opic_name, opic_path);

        let ids: Vec<usize> = self
            .trigger_registry
            .handlers_for_message(topic_name, topic_path)
            .iter()
            .map(|h| h.id)
            .collect();
        if ids.is_empty() {
            return Ok(());
        }
        self.execute_trigger_handlers(ids)
    }

    pub(super) fn run_condition_triggers(&mut self) -> Result<(), SpandaError> {
        // Description:
        //     Run condition triggers.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `run_condition_triggers`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_triggers::run_condition_triggers(&mut self);

        let handlers: Vec<(usize, Expr, bool)> = self
            .trigger_registry
            .condition_handlers()
            .iter()
            .filter_map(|handler| {
                if let TriggerKind::Condition { expr, level } = &handler.kind {
                    Some((handler.id, expr.clone(), *level))
                } else {
                    None
                }
            })
            .collect();
        let mut to_run = Vec::new();
        for (id, expr, level) in handlers {
            let active = matches!(
                self.eval_expr(&expr)?,
                RuntimeValue::Bool { value: true, .. }
            );
            if level {
                if active {
                    to_run.push(id);
                }
            } else if self.condition_trigger_state.should_fire(id, active) {
                to_run.push(id);
            }
        }
        self.execute_trigger_handlers(to_run)
    }

    pub(super) fn run_trigger_maintenance(&mut self) -> Result<(), SpandaError> {
        // Description:
        //     Run trigger maintenance.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `run_trigger_maintenance`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_triggers::run_trigger_maintenance(&mut self);

        self.run_hardware_triggers()?;
        self.run_connectivity_triggers()?;
        self.run_geofence_triggers()?;
        self.poll_transport_inbound_triggers()?;
        self.run_twin_fault_triggers()?;
        self.poll_runtime_health_changes();
        self.poll_runtime_fault_changes();
        Ok(())
    }

    fn run_hardware_triggers(&mut self) -> Result<(), SpandaError> {
        // Description:
        //     Run hardware triggers.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `run_hardware_triggers`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_triggers::run_hardware_triggers(&mut self);

        for event in self.hardware_monitor.poll_new_events() {
            self.dispatch_system_trigger(SystemTriggerCategory::Hardware, &event)?;
            if let Some((domain, evt)) = self.host.hardware_event_to_connectivity(&event) {
                self.dispatch_connectivity_trigger(domain, evt)?;
            }
            self.invoke_recovery_for_event(&event)?;
            let _ = self.try_invoke_continuity_for_event(&event);
        }
        Ok(())
    }

    pub(super) fn dispatch_sensor_event_trigger(
        &mut self,
        sensor: &str,
        event: &str,
    ) -> Result<(), SpandaError> {
        // Description:
        //     Dispatch sensor event trigger.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     sensor: &str
        //         Caller-supplied sensor.
        //     even: &str
        //         Caller-supplied even.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `dispatch_sensor_event_trigger`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_triggers::dispatch_sensor_event_trigger(&mut self, sensor, even);

        let ids: Vec<usize> = self
            .trigger_registry
            .handlers_for_sensor_event(sensor, event)
            .iter()
            .map(|h| h.id)
            .collect();
        if ids.is_empty() {
            return Ok(());
        }
        self.log(format!("sensor trigger: {sensor}.{event}"));
        self.execute_trigger_handlers(ids)
    }

    fn poll_transport_inbound_triggers(&mut self) -> Result<(), SpandaError> {
        // Description:
        //     Poll transport inbound triggers.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `poll_transport_inbound_triggers`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_triggers::poll_transport_inbound_triggers(&mut self);

        let inbound = self.comm_bus.poll_inbound(self.default_transport);
        for (topic_path, envelope) in inbound {
            let payload = Self::runtime_value_payload(&envelope.value);
            if let Err(e) = self.security.verify_inbound_message(
                &topic_path,
                &payload,
                envelope.source_id.as_deref(),
                None,
                self.topic_path_to_message_type
                    .get(&topic_path)
                    .map(String::as_str)
                    .unwrap_or("Unknown"),
            ) {
                if let Some(rt) = self.audit_runtime.as_mut() {
                    let _ = self.security.audit_security_event(
                        rt,
                        "inbound_denied",
                        &format!("topic={topic_path} reason={e}"),
                    );
                }
                continue;
            }
            let topic_name = self
                .topic_path_to_name
                .get(&topic_path)
                .cloned()
                .unwrap_or_else(|| topic_path.trim_start_matches('/').replace('/', "."));
            self.dispatch_message_triggers(&topic_name, &topic_path)?;
        }
        Ok(())
    }

    fn run_twin_fault_triggers(&mut self) -> Result<(), SpandaError> {
        // Description:
        //     Run twin fault triggers.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `run_twin_fault_triggers`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_triggers::run_twin_fault_triggers(&mut self);

        for fault in self.comm_bus.active_faults() {
            let fault_lower = fault.to_ascii_lowercase();
            if (fault_lower.contains("fault")
                || fault_lower.contains("failure")
                || fault_lower.contains("divergence"))
                && self.twin_faults_dispatched.insert(fault.clone())
            {
                let event = if fault_lower.contains("divergence") {
                    "DivergenceDetected"
                } else {
                    "FaultInjected"
                };
                self.dispatch_system_trigger(SystemTriggerCategory::Twin, event)?;
            }
        }
        Ok(())
    }

    pub(super) fn run_timer_triggers(&mut self, sim_time: f64) -> Result<(), SpandaError> {
        // Description:
        //     Run timer triggers.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     sim_time: f64
        //         Caller-supplied sim time.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `run_timer_triggers`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_triggers::run_timer_triggers(&mut self, sim_time);

        let mut to_run = Vec::new();
        for schedule in &mut self.trigger_timers {
            if schedule.next_due_ms <= sim_time {
                if sim_time > schedule.next_due_ms + schedule.interval_ms {
                    if let Some(handler) = self.trigger_registry.get(schedule.trigger_id) {
                        self.telemetry.record_trigger_missed_deadline(
                            &handler.name,
                            trigger_category_label(&handler.kind),
                            handler.priority,
                        );
                    }
                }
                to_run.push(schedule.trigger_id);
                schedule.next_due_ms = sim_time + schedule.interval_ms;
            }
        }
        self.execute_trigger_handlers(to_run)
    }
}
