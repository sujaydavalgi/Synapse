//! Runtime fault polling during simulation — records faults and dispatches triggers.

use super::{Interpreter, RobotBackend};
use spanda_ast::fault_decl::RuntimeFaultTriggerDecl;
use spanda_ast::nodes::{Program, RobotDecl};
use spanda_runtime_faults::{
    faults_from_hardware_signals, record_fault_in_trace, scan_program_faults, FaultScanOptions,
    RuntimeFault,
};

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn cache_fault_program(&mut self, program: &Program) {
        // Cache program metadata when runtime fault monitors are declared.
        //
        // Parameters:
        // - `program` — parsed program AST
        //
        // Returns:
        // None (updates interpreter state).
        //
        // Options:
        // None.
        //
        // Example:
        // self.cache_fault_program(&program);

        let Program::Program {
            runtime_fault_triggers,
            robots,
            ..
        } = program;
        let robot_faults = robots.iter().any(|robot| {
            let RobotDecl::RobotDecl {
                heartbeats,
                memory_watches,
                resource_watches,
                restart_policies,
                ..
            } = robot;
            !heartbeats.is_empty()
                || !memory_watches.is_empty()
                || !resource_watches.is_empty()
                || !restart_policies.is_empty()
        });
        let has_fault_config = robot_faults || !runtime_fault_triggers.is_empty();
        self.fault_program = has_fault_config.then(|| program.clone());
        self.seen_fault_keys.clear();
    }

    pub(super) fn poll_runtime_fault_changes(&mut self) {
        // Poll hardware monitor and configured fault policies for new runtime faults.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // None (records faults to mission trace and dispatches triggers).
        //
        // Options:
        // None.
        //
        // Example:
        // self.poll_runtime_fault_changes();

        let Some(program) = self.fault_program.clone() else {
            return;
        };
        let hw_faults = self.hardware_monitor.runtime_faults();
        let hw_events = self.hardware_monitor.runtime_events();
        let mut faults = faults_from_hardware_signals(&hw_faults, &hw_events, self.sim_time_ms);

        let scan_options = FaultScanOptions {
            sim_time_ms: self.sim_time_ms,
            ..FaultScanOptions::default()
        };
        let scan = scan_program_faults(&program, "runtime", &scan_options);
        for fault in scan.faults {
            if fault.detected_at_ms == 0.0 {
                faults.push(fault);
            }
        }

        for fault in faults {
            let key = format!("{}:{}", fault.kind.as_str(), fault.target);
            if !self.seen_fault_keys.insert(key) {
                continue;
            }
            self.log(format!(
                "runtime fault: {} on {} ({})",
                fault.kind.as_str(),
                fault.target,
                fault.status.as_str()
            ));
            self.record_runtime_fault(&fault);
            self.dispatch_runtime_fault_triggers(&program, &fault);
            let _ = self.try_invoke_continuity_for_event(&fault.target);
            let _ = self.invoke_recovery_for_event(&fault.target);
        }
    }

    fn record_runtime_fault(&mut self, fault: &RuntimeFault) {
        // Append a runtime fault frame to the active mission trace when recording.
        //
        // Parameters:
        // - `fault` — detected runtime fault
        //
        // Returns:
        // None.
        //
        // Options:
        // None.
        //
        // Example:
        // self.record_runtime_fault(&fault);

        if let Some(trace) = self.mission_trace.as_mut() {
            record_fault_in_trace(trace, fault, self.sim_time_ms);
        }
    }

    fn dispatch_runtime_fault_triggers(&mut self, program: &Program, fault: &RuntimeFault) {
        // Execute program-level runtime fault trigger actions for a detected fault.
        //
        // Parameters:
        // - `program` — parsed program with fault triggers
        // - `fault` — detected runtime fault
        //
        // Returns:
        // None.
        //
        // Options:
        // None.
        //
        // Example:
        // self.dispatch_runtime_fault_triggers(&program, &fault);

        let Program::Program {
            runtime_fault_triggers,
            ..
        } = program;
        for trigger in runtime_fault_triggers {
            let RuntimeFaultTriggerDecl::RuntimeFaultTriggerDecl { event, body, .. } = trigger;
            if !fault_trigger_matches(event, fault) {
                continue;
            }
            for action in body {
                self.log(format!("runtime fault action: {action}"));
            }
        }
    }
}

fn fault_trigger_matches(event: &str, fault: &RuntimeFault) -> bool {
    let event_lower = event.to_ascii_lowercase();
    let kind = fault.kind.as_str();
    event_lower.contains(kind)
        || (event_lower.contains("crash") && kind.contains("crash"))
        || (event_lower.contains("memory") && kind.contains("memory"))
        || (event_lower.contains("reboot") && kind.contains("reboot"))
        || (event_lower.contains("restart") && kind.contains("restart"))
        || (event_lower.contains("watchdog") && kind.contains("watchdog"))
        || (event_lower.contains("heartbeat") && kind.contains("heartbeat"))
}
