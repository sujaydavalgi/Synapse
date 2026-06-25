//! Runtime tamper policy dispatch during simulation and live execution.

use super::super::super::fleet_http::{ingest_fleet_tamper_trace, FleetTamperIngestRequest};
use super::{Interpreter, RobotBackend};
use spanda_ast::nodes::Program;
use spanda_tamper::{
    actions_for_tamper_event, extract_tamper_policies, MissionTrace, TamperSeverity, TraceFrame,
};

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn cache_tamper_policies(&mut self, program: &Program) {
        // Cache tamper policies when declared so runtime signals can dispatch responses.
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
        // self.cache_tamper_policies(&program);

        let Program::Program { tamper_policies, .. } = program;
        self.tamper_policies = if tamper_policies.is_empty() {
            Vec::new()
        } else {
            extract_tamper_policies(program)
        };
        self.applied_tamper_branches.clear();
    }

    pub(super) fn invoke_tamper_policies(
        &mut self,
        signal: &str,
        severity: TamperSeverity,
    ) {
        // Match tamper policies and dispatch declared response actions once per branch.
        //
        // Parameters:
        // - `signal` — runtime tamper signal label
        // - `severity` — tamper severity tier
        //
        // Returns:
        // None (dispatches recovery actions and records audit events).
        //
        // Options:
        // None.
        //
        // Example:
        // self.invoke_tamper_policies("agent_capability_denied", TamperSeverity::High);

        if self.tamper_policies.is_empty() {
            return;
        }

        let actions = actions_for_tamper_event(&self.tamper_policies, signal, severity);
        if actions.is_empty() {
            return;
        }

        self.log(format!(
            "tamper: signal '{signal}' severity {:?} matched {} action(s)",
            severity,
            actions.len()
        ));
        self.record_mission_event(
            "tamper_policy",
            serde_json::json!({
                "signal": signal,
                "severity": format!("{:?}", severity),
            }),
        );
        self.maybe_ingest_tamper_to_mesh(signal, severity);

        for action in actions {
            let branch_key = format!("{signal}:{action}");
            if !self.applied_tamper_branches.insert(branch_key) {
                continue;
            }
            if severity >= TamperSeverity::Critical && self.action_requires_tamper_approval(&action)
            {
                self.poll_recovery_approvals();
                if !self.operator_approval_granted(&action) {
                    self.request_operator_approval(&action);
                    self.log(format!(
                        "tamper: deferred critical action '{action}' pending operator approval"
                    ));
                    continue;
                }
            }
            self.log(format!("tamper: action {action}"));
            if let Err(error) = self.dispatch_recovery_action(&action) {
                self.log(format!("tamper: action failed: {error}"));
            }
            if action.contains("audit.record") {
                self.record_debug_event(1, "audit_record", &[("event", action.clone())]);
            }
        }
    }

    fn action_requires_tamper_approval(&self, action: &str) -> bool {
        let lower = action.to_ascii_lowercase();
        lower.contains("stop")
            || lower.contains("kill")
            || lower.contains("halt")
            || lower.contains("emergency_stop")
            || lower.contains("safe")
    }

    fn maybe_ingest_tamper_to_mesh(&self, signal: &str, severity: TamperSeverity) {
        let Some(mesh_url) = std::env::var("SPANDA_FLEET_MESH_URL").ok() else {
            return;
        };
        let robot_id = self.publish_source_id();
        let trace = MissionTrace {
            version: 1,
            source: robot_id.clone(),
            deterministic: true,
            frames: vec![TraceFrame {
                sim_time_ms: self.sim_time_ms,
                event: "security_audit".into(),
                payload: serde_json::json!({
                    "kind": signal,
                    "severity": format!("{:?}", severity),
                    "robot": robot_id,
                }),
            }],
        };
        let trace_json = serde_json::to_string(&trace).unwrap_or_default();
        let fleet_name = self.fleets.names().next().cloned();
        let request = FleetTamperIngestRequest {
            robot_id: robot_id.clone(),
            trace_json,
            fleet_name,
        };
        let token = std::env::var("SPANDA_FLEET_MESH_TOKEN").ok();
        if let Err(error) = ingest_fleet_tamper_trace(&mesh_url, &request, token.as_deref()) {
            eprintln!("fleet tamper ingest failed: {error}");
        }
    }
}
