//! Agent capability checks and secure-operation helpers.
//!

use super::{IntoSpandaError, Interpreter, RobotBackend, RuntimeError};
use crate::error::SpandaError;
use spanda_security::{SecurePolicy, TrustLevel};

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn check_agent_capability(
        &self,
        agent: &str,
        action: &str,
        target: Option<&str>,
        line: u32,
    ) -> Result<(), SpandaError> {
        // Check agent capability.
        //
        // Parameters:
        // - `self` — method receiver
        // - `agent` — input value
        // - `action` — input value
        // - `target` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.check_agent_capability(agent, action, target, line);

        // Compute caps for the following logic.
        let caps = self
            .agent_capabilities
            .get(agent)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        // Skip further work when caps is empty.
        if caps.is_empty() {
            return Ok(());
        }
        let allowed = caps
            .iter()
            .any(|c| c.action == action && (target.is_none() || c.target.as_deref() == target));

        // Take the branch when allowed is false.
        if !allowed {
            return Err(RuntimeError::new(
                format!(
                    "Agent '{agent}' lacks capability {action}{}",
                    target.map(|t| format!("({t})")).unwrap_or_default()
                ),
                line,
            )
            .into_spanda());
        }
        Ok(())
    }

    pub(super) fn publish_source_id(&self) -> String {
        if let Some(agent) = &self.current_agent {
            return agent.clone();
        }
        self.security
            .identity
            .as_ref()
            .map(|id| id.id().to_string())
            .unwrap_or_else(|| "robot".into())
    }

    pub(super) fn secure_policy_from_block(block: &spanda_ast::foundations::SecureBlockDecl) -> SecurePolicy {
        // Secure policy from block.
        //
        // Parameters:
        // - `block` — input value
        //
        // Returns:
        // SecurePolicy.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::runtime::secure_policy_from_block(block);

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
        // Resolve signing key.
        //
        // Parameters:
        // - `self` — method receiver
        // - `key` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.resolve_signing_key(key);

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

    pub(super) fn security_error(&self, err: spanda_security::SecurityError, line: u32) -> SpandaError {
        // Security error.
        //
        // Parameters:
        // - `self` — method receiver
        // - `err` — input value
        // - `line` — input value
        //
        // Returns:
        // SpandaError.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.security_error(err, line);

        // Produce into spanda as the result.
        RuntimeError::new(err.to_string(), line).into_spanda()
    }

}
