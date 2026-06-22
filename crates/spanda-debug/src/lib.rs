//! debug support for Spanda.
//!
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebugCommand {
    Continue,
    Step,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugPause {
    pub line: u32,
    pub reason: String,
    #[serde(default)]
    pub variables: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct DebugOptions {
    pub breakpoints: HashSet<u32>,
    pub step: bool,
    pub source_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSession {
    pub pauses: Vec<DebugPause>,
}

impl DebugSession {
    pub fn paused(&self) -> bool {
        // Paused.
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
        // let result = instance.paused();

        // Produce is empty as the result.
        !self.pauses.is_empty()
    }
}

#[derive(Clone)]
pub struct DebugController {
    breakpoints: HashSet<u32>,
    step: RefCell<bool>,
    pauses: Rc<RefCell<Vec<DebugPause>>>,
}

impl DebugController {
    pub fn new(options: DebugOptions) -> Self {
        // Create a new instance.
        //
        // Parameters:
        // - `options` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_debug::new(options);

        // Assemble the struct fields and return it.
        Self {
            breakpoints: options.breakpoints,
            step: RefCell::new(options.step),
            pauses: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn pauses(&self) -> Rc<RefCell<Vec<DebugPause>>> {
        // Pauses.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Rc<RefCell<Vec<DebugPause>>>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.pauses();

        // Call clone on the current instance.
        self.pauses.clone()
    }

    pub fn should_pause(&self, line: u32) -> bool {
        // Should pause.
        //
        // Parameters:
        // - `self` — method receiver
        // - `line` — input value
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.should_pause(line);

        // take this path when *self.step.borrow().
        if *self.step.borrow() {
            *self.step.borrow_mut() = false;
            return true;
        }
        self.breakpoints.contains(&line)
    }

    pub fn record_pause(
        &self,
        line: u32,
        reason: &str,
        variables: std::collections::HashMap<String, String>,
    ) {
        // Record pause.
        //
        // Parameters:
        // - `self` — method receiver
        // - `line` — input value
        // - `reason` — input value
        // - `variables` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.record_pause(line, reason, variables);

        // Append into self.
        self.pauses.borrow_mut().push(DebugPause {
            line,
            reason: reason.to_string(),
            variables,
        });
    }

    pub fn command(&self, cmd: DebugCommand) {
        // Command.
        //
        // Parameters:
        // - `self` — method receiver
        // - `cmd` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.command(cmd);

        // keep entries that match the expected pattern.
        if matches!(cmd, DebugCommand::Step) {
            *self.step.borrow_mut() = true;
        }
    }
}

pub fn stmt_line(stmt: &spanda_ast::nodes::Stmt) -> u32 {
    // Stmt line.
    //
    // Parameters:
    // - `stmt` — input value
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_debug::stmt_line(stmt);

    // Import the items needed by the logic below.
    use spanda_ast::nodes::Stmt;

    // Match on stmt and handle each case.
    match stmt {
        Stmt::VarDecl { span, .. }
        | Stmt::IfStmt { span, .. }
        | Stmt::LoopStmt { span, .. }
        | Stmt::ExprStmt { span, .. }
        | Stmt::ReturnStmt { span, .. }
        | Stmt::PublishStmt { span, .. }
        | Stmt::ServiceCallStmt { span, .. }
        | Stmt::ActionSendStmt { span, .. }
        | Stmt::EmergencyStopStmt { span, .. }
        | Stmt::ResetEmergencyStopStmt { span, .. }
        | Stmt::EmitStmt { span, .. }
        | Stmt::EnterStmt { span, .. }
        | Stmt::RememberStmt { span, .. }
        | Stmt::SubscribeStmt { span, .. }
        | Stmt::ExecuteStmt { span, .. }
        | Stmt::DiscoverStmt { span, .. }
        | Stmt::ReceiveStmt { span, .. }
        | Stmt::SpawnStmt { span, .. }
        | Stmt::SelectStmt { span, .. }
        | Stmt::ParallelStmt { span, .. }
        | Stmt::EnterModeStmt { span, .. }
        | Stmt::UseFallbackStmt { span, .. }
        | Stmt::StopAllActuatorsStmt { span, .. }
        | Stmt::RunPipelineStmt { span, .. }
        | Stmt::NavigateStmt { span, .. }
        | Stmt::ExpectCompileErrorStmt { span, .. } => span.start.line,
    }
}
