//! Runtime state for watchdogs, pipelines, retries, and recovery handlers.

use crate::ast::Stmt;
use crate::foundations::{PipelineDecl, RecoverDecl, RetryDecl, WatchdogDecl};
use std::collections::HashMap;

/// Loaded watchdog handler ready for sim-time evaluation.
#[derive(Debug, Clone)]
pub struct WatchdogRuntime {
    pub name: String,
    pub target: Option<String>,
    pub timeout_ms: f64,
    pub body: Vec<Stmt>,
    pub last_fired_at_ms: Option<f64>,
}

/// Loaded latency-budget pipeline.
#[derive(Debug, Clone)]
pub struct PipelineRuntime {
    pub name: String,
    pub budget_ms: f64,
    pub body: Vec<Stmt>,
}

/// Loaded retry policy with optional fallback block.
#[derive(Debug, Clone)]
pub struct RetryRuntime {
    pub attempts: u32,
    pub backoff_ms: f64,
    pub body: Vec<Stmt>,
    pub fallback: Vec<Stmt>,
    pub attempt: u32,
    pub exhausted: bool,
}

impl WatchdogRuntime {
    pub fn from_decl(decl: &WatchdogDecl) -> Self {
        // Build runtime watchdog state from a parsed declaration.
        //
        // Parameters:
        // - `decl` — parsed watchdog block
        //
        // Returns:
        // Watchdog runtime entry.
        //
        // Options:
        // None.
        //
        // Example:
        // let wd = WatchdogRuntime::from_decl(&watchdog_decl);

        // Copy declaration fields into the runtime container.
        let WatchdogDecl::WatchdogDecl {
            name,
            target,
            timeout_ms,
            body,
            ..
        } = decl;
        Self {
            name: name.clone(),
            target: target.clone(),
            timeout_ms: *timeout_ms,
            body: body.clone(),
            last_fired_at_ms: None,
        }
    }
}

impl PipelineRuntime {
    pub fn from_decl(decl: &PipelineDecl) -> Self {
        // Build runtime pipeline state from a parsed declaration.
        //
        // Parameters:
        // - `decl` — parsed pipeline block
        //
        // Returns:
        // Pipeline runtime entry.
        //
        // Options:
        // None.
        //
        // Example:
        // let pipeline = PipelineRuntime::from_decl(&pipeline_decl);

        // Copy declaration fields into the runtime container.
        let PipelineDecl::PipelineDecl {
            name,
            budget_ms,
            body,
            ..
        } = decl;
        Self {
            name: name.clone(),
            budget_ms: *budget_ms,
            body: body.clone(),
        }
    }
}

impl RetryRuntime {
    pub fn from_decl(decl: &RetryDecl) -> Self {
        // Build runtime retry state from a parsed declaration.
        //
        // Parameters:
        // - `decl` — parsed retry block
        //
        // Returns:
        // Retry runtime entry.
        //
        // Options:
        // None.
        //
        // Example:
        // let retry = RetryRuntime::from_decl(&retry_decl);

        // Copy declaration fields into the runtime container.
        let RetryDecl::RetryDecl {
            attempts,
            backoff_ms,
            body,
            fallback,
            ..
        } = decl;
        Self {
            attempts: *attempts,
            backoff_ms: *backoff_ms,
            body: body.clone(),
            fallback: fallback.clone(),
            attempt: 0,
            exhausted: false,
        }
    }
}

/// Recovery handlers keyed by error or hardware event name.
pub type RecoverHandlers = HashMap<String, Vec<Stmt>>;

pub fn recover_handlers_from_decls(recovers: &[RecoverDecl]) -> RecoverHandlers {
    // Index recovery handlers by declared error name.
    //
    // Parameters:
    // - `recovers` — parsed recover blocks
    //
    // Returns:
    // Map from error name to handler body statements.
    //
    // Options:
    // None.
    //
    // Example:
    // let handlers = recover_handlers_from_decls(&robot.recovers);

    // Build a lookup table for runtime dispatch.
    let mut handlers = HashMap::new();
    for decl in recovers {
        let RecoverDecl::RecoverDecl {
            error_name, body, ..
        } = decl;
        handlers.insert(error_name.clone(), body.clone());
    }
    handlers
}
