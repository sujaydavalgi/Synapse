//! Runtime operational policy enforcement hooks for interpreter motion.
//!

use super::{Interpreter, MotionCommand, RobotBackend};
use spanda_runtime::operational_policy::RuntimePolicyMonitor;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn load_runtime_policy(
        &mut self,
        monitor: RuntimePolicyMonitor,
    ) -> Result<(), spanda_error::SpandaError> {
        // Attach a compiled runtime policy monitor to the interpreter.
        //
        // Parameters:
        // - `monitor` — compiled policy rules for motion gating
        //
        // Returns:
        // Ok after storing the monitor.
        //
        // Options:
        // None.
        //
        // Example:
        // self.load_runtime_policy(monitor)?;

        self.log(format!(
            "runtime policy '{}' enabled (max_speed={:?}, operation_hours={:?})",
            monitor.policy_name, monitor.max_speed_mps, monitor.operation_hours
        ));
        self.runtime_policy = Some(monitor);
        Ok(())
    }

    pub(super) fn check_runtime_policy_before_motion(&self, linear_mps: f64) -> Option<String> {
        // Return a block reason when runtime policy forbids the requested motion.
        //
        // Parameters:
        // - `linear_mps` — requested linear speed in meters per second
        //
        // Returns:
        // Human-readable block reason, or None when allowed.
        //
        // Options:
        // None.
        //
        // Example:
        // if let Some(reason) = self.check_runtime_policy_before_motion(0.8) { ... }

        let monitor = self.runtime_policy.as_ref()?;
        match spanda_runtime::operational_policy::check_runtime_policy_motion(monitor, linear_mps) {
            Ok(()) => None,
            Err(violation) => Some(format!(
                "Policy {}:{} — {}",
                violation.policy, violation.rule, violation.message
            )),
        }
    }

    pub(super) fn block_motion_for_policy(
        &mut self,
        actuator: &str,
        reason: String,
        _line: u32,
    ) -> Result<super::RuntimeValue, spanda_error::SpandaError> {
        // Stop actuators and surface a policy violation to callers when needed.
        //
        // Parameters:
        // - `actuator` — actuator name receiving the command
        // - `reason` — policy block explanation
        // - `line` — source line for errors
        //
        // Returns:
        // Void when motion is silently blocked, or a runtime error for strict mode.
        //
        // Options:
        // None.
        //
        // Example:
        // return self.block_motion_for_policy("wheels", reason, line);

        if let Some(cb) = &self.options.on_motion_blocked {
            cb(reason.clone());
        }
        self.log(reason.clone());
        self.backend.execute_motion(MotionCommand::Stop {
            actuator: actuator.to_string(),
        });
        Ok(super::RuntimeValue::Void)
    }
}
