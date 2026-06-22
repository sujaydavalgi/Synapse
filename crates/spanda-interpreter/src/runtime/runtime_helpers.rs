//! Shared interpreter helpers used across runtime child modules.
//!

use super::{Interpreter, RobotBackend, RuntimeValue};
use crate::ai::MemoryStore;
use spanda_ast::nodes::{AgentDecl, Expr};
use spanda_error::SpandaError;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn goal_text_from_value(value: &RuntimeValue) -> Option<String> {
        // Goal text from value.
        //
        // Parameters:
        // - `value` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::runtime::goal_text_from_value(value);

        // Match on value and handle each case.
        match value {
            RuntimeValue::Goal { text } => Some(text.clone()),
            RuntimeValue::String { value } => Some(value.clone()),
            _ => None,
        }
    }

    pub(super) fn resolve_reason_goal(
        &mut self,
        named_args: &[spanda_ast::nodes::NamedArg],
        line: u32,
    ) -> Result<Option<String>, SpandaError> {
        // Resolve reason goal.
        //
        // Parameters:
        // - `self` — method receiver
        // - `named_args` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.resolve_reason_goal(named_args, line);

        // handle the success value from get named arg value.
        if let Ok(value) = self.get_named_arg_value(named_args, "goal") {
            // Keep entries that match the expected pattern.
            if !matches!(value, RuntimeValue::Void) {
                return Ok(Self::goal_text_from_value(&value));
            }
        }

        // Emit output when as deref provides a agent name.
        if let Some(agent_name) = self.current_agent.as_deref() {
            // Emit output when get provides a agent.
            if let Some(agent) = self.agents.get(agent_name) {
                let text = match &agent.decl {
                    AgentDecl::AgentDecl { goal, .. } => goal.clone(),
                };

                // Skip further work when !text is empty.
                if !text.is_empty() {
                    return Ok(Some(text));
                }
            }
        }
        let _ = line;
        Ok(None)
    }

    pub(super) fn enrich_reason_goal(&self, goal: Option<String>) -> Option<String> {
        // Enrich reason goal.
        //
        // Parameters:
        // - `self` — method receiver
        // - `goal` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.enrich_reason_goal(goal);

        // Create mutable parts for accumulating results.
        let mut parts = Vec::new();

        // Emit output when is empty provides a g.
        if let Some(g) = goal.filter(|s| !s.is_empty()) {
            parts.push(g);
        }

        // Emit output when as deref provides a agent name.
        if let Some(agent_name) = self.current_agent.as_deref() {
            // Emit output when self provides a summary.
            if let Some(summary) = self
                .agents
                .get(agent_name)
                .and_then(|a| a.memory.as_ref())
                .and_then(MemoryStore::summary_for_prompt)
            {
                parts.push(summary);
            }
        }

        // Skip further work when parts is empty.
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("\n"))
        }
    }

    pub(super) fn expr_path_string(expr: &Expr) -> String {
        // Expr path string.
        //
        // Parameters:
        // - `expr` — input value
        //
        // Returns:
        // Text result.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::runtime::expr_path_string(expr);

        // Match on expr and handle each case.
        match expr {
            Expr::IdentExpr { name, .. } => name.clone(),
            Expr::MemberExpr {
                object, property, ..
            } => {
                format!("{}.{}", Self::expr_path_string(object), property)
            }
            _ => String::new(),
        }
    }

    pub(super) fn runtime_value_payload(value: &RuntimeValue) -> String {
        // Runtime value payload.
        //
        // Parameters:
        // - `value` — input value
        //
        // Returns:
        // Text result.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::runtime::runtime_value_payload(value);

        // Match on value and handle each case.
        match value {
            RuntimeValue::String { value } => value.clone(),
            RuntimeValue::Number { value, .. } => value.to_string(),
            RuntimeValue::Bool { value } => value.to_string(),
            RuntimeValue::Pose { x, y, theta, z } => {
                format!(r#"{{"x":{x},"y":{y},"theta":{theta},"z":{z}}}"#)
            }
            RuntimeValue::SafeAction { linear, angular } => {
                format!(r#"{{"linear":{linear},"angular":{angular}}}"#)
            }
            RuntimeValue::ActionProposal {
                linear,
                angular,
                source,
                ..
            } => format!(r#"{{"linear":{linear},"angular":{angular},"source":"{source}"}}"#),
            _ => format!("{value:?}"),
        }
    }

}
