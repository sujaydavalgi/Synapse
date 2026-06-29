//! Agent capability checks and secure-operation helpers.
//!

use super::{Interpreter, IntoSpandaError, RobotBackend, RuntimeError};
use spanda_error::SpandaError;
use spanda_security::{SecurePolicy, TrustLevel};
use spanda_runtime::tamper_policy::TamperSeverity;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn check_agent_capability(
        &mut self,
        agent: &str,
        action: &str,
        target: Option<&str>,
        line: u32,
    ) -> Result<(), SpandaError> {
        // Description:
        //     Check agent capability.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     agen: &str
        //         Caller-supplied agen.
        //     action: &str
        //         Caller-supplied action.
        //     arge: Option<&str>
        //         Caller-supplied arge.
        //     line: u32
        //         Caller-supplied line.
        //
        // Outputs:
        //     result: Result<(), SpandaError>
        //         Return value from `check_agent_capability`.
        //
        // Example:
        //     let result = spanda_interpreter::runtime_security::check_agent_capability(&mut self, agen, action, arge, line);

        // Compute caps for the following logic.
        let caps = self
            .agent_capabilities
            .get(agent)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let enforced = self
            .agent_capability_enforced
            .get(agent)
            .copied()
            .unwrap_or(false);

        // Deny high-risk actions when capability enforcement is declared but empty.
        if enforced && caps.is_empty() {
            if matches!(action, "execute" | "propose_motion") {
                self.log(format!(
                    "agent: denied {agent} {action} (capability_enforced)"
                ));
                return Err(RuntimeError::new(
                    format!("Agent '{agent}' declares can[] but lacks capability {action}"),
                    line,
                )
                .into_spanda());
            }
            return Ok(());
        }

        // Skip further work when caps is empty and enforcement is not declared.
        if caps.is_empty() {
            return Ok(());
        }
        let allowed = caps
            .iter()
            .any(|c| c.action == action && (target.is_none() || c.target.as_deref() == target));

        // Take the branch when allowed is false.
        if !allowed {
            self.log(format!(
                "agent: denied {agent} {action}{}",
                target.map(|t| format!("({t})")).unwrap_or_default()
            ));
            self.invoke_tamper_policies("agent_capability_denied", TamperSeverity::High);
            if let Some(rt) = self.audit_runtime.as_mut() {
                let _ = self.security.audit_security_event(
                    rt,
                    "agent_capability_denied",
                    &format!(
                        "agent={agent} action={action} target={}",
                        target.unwrap_or("none")
                    ),
                );
            }
            return Err(RuntimeError::new(
                format!(
                    "Agent '{agent}' lacks capability {action}{}",
                    target.map(|t| format!("({t})")).unwrap_or_default()
                ),
                line,
            )
            .into_spanda());
        }
        self.log(format!(
            "agent: allowed {agent} {action}{}",
            target.map(|t| format!("({t})")).unwrap_or_default()
        ));
        if let Some(rt) = self.audit_runtime.as_mut() {
            let _ = self.security.audit_security_event(
                rt,
                "agent_capability_granted",
                &format!(
                    "agent={agent} action={action} target={}",
                    target.unwrap_or("none")
                ),
            );
        }
        Ok(())
    }

    pub(super) fn publish_source_id(&self) -> String {
        // Description:
        //     Publish source id.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: String
        //         Return value from `publish_source_id`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_security::publish_source_id(&self);

        if let Some(agent) = &self.current_agent {
            return agent.clone();
        }
        self.security
            .identity
            .as_ref()
            .map(|id| id.id().to_string())
            .unwrap_or_else(|| "robot".into())
    }

    pub(super) fn secure_policy_from_block(
        block: &spanda_ast::foundations::SecureBlockDecl,
    ) -> SecurePolicy {
        // Description:
        //     Secure policy from block.
        //
        // Inputs:
        //     block: &spanda_ast::foundations::SecureBlockDecl
        //         Caller-supplied block.
        //
        // Outputs:
        //     result: SecurePolicy
        //         Return value from `secure_policy_from_block`.
        //
        // Example:
        //     let result = spanda_interpreter::runtime_security::secure_policy_from_block(block);

        // Produce SecurePolicy as the result.
        SecurePolicy {
            signed: block.signed,
            min_trust: block
                .min_trust
                .as_ref()
                .and_then(|s| s.parse::<TrustLevel>().ok()),
            requires: block.requires.clone(),
            encryption: block
                .encryption
                .as_deref()
                .and_then(|s| s.parse().ok())
                .unwrap_or_default(),
            authentication: block
                .authentication
                .as_deref()
                .and_then(|s| s.parse().ok())
                .unwrap_or_default(),
            integrity: block
                .integrity
                .as_deref()
                .and_then(|s| s.parse().ok())
                .unwrap_or_default(),
            trusted_sources: block.trusted_sources.clone(),
            reject_untrusted: block.reject_untrusted,
        }
    }

    pub(super) fn resolve_signing_key(&self, key: &str) -> Result<String, SpandaError> {
        // Description:
        //     Resolve signing key.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //     key: &str
        //         Caller-supplied key.
        //
        // Outputs:
        //     result: Result<String, SpandaError>
        //         Return value from `resolve_signing_key`.
        //
        // Example:
        //     let result = spanda_interpreter::runtime_security::resolve_signing_key(&self, key);

        // proceed only when is ok is available.
        if self.security.secrets.get(key).is_ok() {
            self.security
                .secrets
                .resolve(key)
                .map_err(|e| RuntimeError::new(e.to_string(), 0).into_spanda())
        } else {
            Ok(key.to_string())
        }
    }

    pub(super) fn security_error(
        &self,
        err: spanda_security::SecurityError,
        line: u32,
    ) -> SpandaError {
        // Description:
        //     Security error.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //     err: spanda_security::SecurityError
        //         Caller-supplied err.
        //     line: u32
        //         Caller-supplied line.
        //
        // Outputs:
        //     result: SpandaError
        //         Return value from `security_error`.
        //
        // Example:
        //     let result = spanda_interpreter::runtime_security::security_error(&self, err, line);

        // Produce into spanda as the result.
        RuntimeError::new(err.to_string(), line).into_spanda()
    }
}
