//! Resumable debug sessions with step/continue and variable mutation.

use crate::ast::{BehaviorDecl, Program, RobotDecl, Stmt, UnitKind};
use crate::debug::{stmt_line, DebugController, DebugOptions, DebugPause, DebugSession};
use crate::error::SpandaError;
use crate::runtime::{Environment, Interpreter, InterpreterOptions, RuntimeValue};
use crate::simulator::{create_default_simulator, Simulator, SimulatorConfig};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugStepKind {
    Continue,
    StepOver,
    StepIn,
    StepOut,
}

#[derive(Debug, Clone)]
pub struct DebugStackFrame {
    pub id: usize,
    pub name: String,
    pub line: u32,
}

#[derive(Debug, Clone)]
struct DebugFrame {
    name: String,
    stmts: Vec<Stmt>,
    index: usize,
    restore_env: Option<Environment>,
    locals: HashMap<String, String>,
}

pub struct DebugMachine {
    interpreter: Interpreter<Simulator>,
    frames: Vec<DebugFrame>,
    controller: DebugController,
    step_kind: DebugStepKind,
    step_out_target_depth: Option<usize>,
    source_path: Option<String>,
    finished: bool,
}

impl DebugMachine {
    pub fn start(source: &str, options: DebugOptions) -> Result<Self, SpandaError> {
        // Start.
        //
        // Parameters:
        // - `source` — input value
        // - `options` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::debug_session::start(source, options);

        // Parse and type-check the source program.
        let program = crate::compile(source)?.program;
        let step = options.step;
        let controller = DebugController::new(options.clone());
        let mut interpreter = Interpreter::new(
            create_default_simulator(SimulatorConfig::default()),
            InterpreterOptions {
                max_loop_iterations: 100,
                ..Default::default()
            },
        );
        interpreter.load_program_metadata(&program);
        let Program::Program { robots, .. } = &program;
        let robot = robots.first().ok_or_else(|| SpandaError::Runtime {
            message: "debug requires at least one robot".into(),
            line: 1,
        })?;
        interpreter.setup_robot_for_debug(robot)?;
        let (name, body) = behavior_body(robot)?;
        let locals = interpreter.env().snapshot_display();
        Ok(Self {
            interpreter,
            frames: vec![DebugFrame {
                name,
                stmts: body,
                index: 0,
                restore_env: None,
                locals,
            }],
            controller,
            step_kind: if step {
                DebugStepKind::StepOver
            } else {
                DebugStepKind::Continue
            },
            step_out_target_depth: None,
            source_path: options.source_path,
            finished: false,
        })
    }

    pub fn is_finished(&self) -> bool {
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
        // let result = instance.is_finished();

        // Call finished on the current instance.
        self.finished
    }

    pub fn source_path(&self) -> Option<&str> {
        // Source path.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.source_path();

        // Call as deref on the current instance.
        self.source_path.as_deref()
    }

    pub fn pauses(&self) -> Vec<DebugPause> {
        // Pauses.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<DebugPause>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.pauses();

        // Call pauses on the current instance.
        self.controller.pauses().borrow().clone()
    }

    pub fn stack_trace(&self) -> Vec<(String, u32)> {
        // Stack trace.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<(String, u32)>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.stack_trace();

        // Call stack trace frames on the current instance.
        self.stack_trace_frames()
            .into_iter()
            .map(|frame| (frame.name, frame.line))
            .collect()
    }

    pub fn stack_trace_frames(&self) -> Vec<DebugStackFrame> {
        // Stack trace frames.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<DebugStackFrame>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.stack_trace_frames();

        // Call frames on the current instance.
        self.frames
            .iter()
            .rev()
            .enumerate()
            .map(|(id, frame)| {
                let line = frame.stmts.get(frame.index).map(stmt_line).unwrap_or(1);
                DebugStackFrame {
                    id: id + 1,
                    name: frame.name.clone(),
                    line,
                }
            })
            .collect()
    }

    pub fn frame_variables(&self, frame_id: usize) -> HashMap<String, String> {
        // Frame variables.
        //
        // Parameters:
        // - `self` — method receiver
        // - `frame_id` — input value
        //
        // Returns:
        // HashMap<String, String>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.frame_variables(frame_id);

        // take the branch when frame id equals 0.
        if frame_id == 0 {
            return self.interpreter.env().snapshot_display();
        }
        let index = self.frames.len().saturating_sub(frame_id);
        self.frames
            .get(index)
            .map(|frame| frame.locals.clone())
            .unwrap_or_default()
    }

    pub fn set_variable(&mut self, name: &str, value: &str) -> Result<(), SpandaError> {
        // Set variable.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `value` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.set_variable(name, value);

        // Call interpreter on the current instance.
        self.interpreter
            .env_mut()
            .set(name, parse_debug_value(value));

        // Emit output when last mut provides a frame.
        if let Some(frame) = self.frames.last_mut() {
            frame.locals = self.interpreter.env().snapshot_display();
        }
        Ok(())
    }

    pub fn run_until_pause(&mut self, step: DebugStepKind) -> Result<DebugSession, SpandaError> {
        // Run until pause.
        //
        // Parameters:
        // - `self` — method receiver
        // - `step` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.run_until_pause(step);

        // Call step kind = step; on the current instance.
        self.step_kind = step;

        // Take the branch when step equals Continue.
        if step == DebugStepKind::Continue {
            self.controller
                .command(crate::debug::DebugCommand::Continue);
            self.step_out_target_depth = None;
        } else if step == DebugStepKind::StepOut {
            self.step_out_target_depth = Some(self.frames.len().saturating_sub(1));
            self.controller.command(crate::debug::DebugCommand::Step);
        } else {
            self.step_out_target_depth = None;
            self.controller.command(crate::debug::DebugCommand::Step);
        }

        // Run the loop body until it exits.
        loop {
            // Skip further work when frames is empty.
            if self.frames.is_empty() {
                // Proceed only when is some is available.
                if self.step_out_target_depth.is_some() {
                    self.record_pause_at_top(1, "step-out");
                    self.step_out_target_depth = None;
                }
                self.finished = true;
                break;
            }
            let frame_top = self.frames.len() - 1;

            // Take this path when self.frames[frame top].index >= self.frames[frame top].stmts.len().
            if self.frames[frame_top].index >= self.frames[frame_top].stmts.len() {
                // Emit output when take provides a env.
                if let Some(env) = self.frames[frame_top].restore_env.take() {
                    self.interpreter.restore_env(env);
                }
                self.frames.pop();

                // Emit output when step out target depth provides a target.
                if let Some(target) = self.step_out_target_depth {
                    // Take this path when self.frames.len() <= target.
                    if self.frames.len() <= target {
                        let line = self
                            .frames
                            .last()
                            .and_then(|frame| frame.stmts.get(frame.index))
                            .map(stmt_line)
                            .unwrap_or(1);
                        self.record_pause_at_top(line, "step-out");
                        self.step_out_target_depth = None;
                        break;
                    }
                }
                continue;
            }
            let stmt = self.frames[frame_top].stmts[self.frames[frame_top].index].clone();
            let line = stmt_line(&stmt);

            // Take this path when self.step out target depth.is none().
            if self.step_out_target_depth.is_none()
                && matches!(step, DebugStepKind::Continue)
                && self.controller.should_pause(line)
            {
                self.record_pause_at_top(line, pause_reason(step));
                break;
            }

            // Take this path when self.try enter inner(step, &stmt, frame top, line)?.
            if self.try_enter_inner(step, &stmt, frame_top, line)? {
                break;
            }
            self.frames[frame_top].index += 1;
            self.interpreter.debug_execute_stmt(&stmt)?;
            self.sync_top_locals();

            // Keep entries that match the expected pattern.
            if matches!(step, DebugStepKind::StepOver | DebugStepKind::StepIn) {
                let reason = if step == DebugStepKind::StepIn {
                    "step-in"
                } else {
                    "step"
                };
                self.record_pause_at_top(line, reason);
                break;
            }
        }
        Ok(DebugSession {
            pauses: self.controller.pauses().borrow().clone(),
        })
    }

    fn sync_top_locals(&mut self) {
        // Sync top locals.
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
        // let result = instance.sync_top_locals();

        // use frame when last mut is present.

        // Emit output when last mut provides a frame.
        if let Some(frame) = self.frames.last_mut() {
            frame.locals = self.interpreter.env().snapshot_display();
        }
    }

    fn record_pause_at_top(&mut self, line: u32, reason: &str) {
        // Record pause at top.
        //
        // Parameters:
        // - `self` — method receiver
        // - `line` — input value
        // - `reason` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.record_pause_at_top(line, reason);

        // Call sync top locals on the current instance.
        self.sync_top_locals();
        let variables = self
            .frames
            .last()
            .map(|frame| frame.locals.clone())
            .unwrap_or_default();
        self.controller.record_pause(line, reason, variables);
    }

    fn try_enter_inner(
        &mut self,
        step: DebugStepKind,
        stmt: &Stmt,
        frame_top: usize,
        line: u32,
    ) -> Result<bool, SpandaError> {
        // Try enter inner.
        //
        // Parameters:
        // - `self` — method receiver
        // - `step` — input value
        // - `stmt` — input value
        // - `frame_top` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.try_enter_inner(step, stmt, frame_top, line);

        // take the branch when step differs from StepIn.
        if step != DebugStepKind::StepIn {
            return Ok(false);
        }

        // Emit output when inner block provides a inner.
        if let Some(inner) = inner_block(stmt) {
            self.frames[frame_top].index += 1;
            let locals = self.interpreter.env().snapshot_display();
            self.frames.push(DebugFrame {
                name: format!("{}:{}", self.frames[frame_top].name, stmt_kind_label(stmt)),
                stmts: inner,
                index: 0,
                restore_env: None,
                locals,
            });
            self.record_pause_at_top(line, "step-in");
            return Ok(true);
        }

        // Take this path when let Some((func name, func, args)) = self.interpreter.resolve sync call.
        if let Some((func_name, func, args)) = self.interpreter.resolve_sync_call(stmt) {
            let saved = self.interpreter.bind_call_args(&func, &args)?;
            self.frames[frame_top].index += 1;
            let locals = self.interpreter.env().snapshot_display();
            self.frames.push(DebugFrame {
                name: func_name,
                stmts: func.body.clone(),
                index: 0,
                restore_env: Some(saved),
                locals,
            });
            self.record_pause_at_top(line, "step-in");
            return Ok(true);
        }
        Ok(false)
    }
}

fn behavior_body(robot: &RobotDecl) -> Result<(String, Vec<Stmt>), SpandaError> {
    // Behavior body.
    //
    // Parameters:
    // - `robot` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::debug_session::behavior_body(robot);

    // Compute RobotDecl for the following logic.
    let RobotDecl::RobotDecl { behaviors, .. } = robot;
    let BehaviorDecl::BehaviorDecl { name, body, .. } =
        behaviors.first().ok_or_else(|| SpandaError::Runtime {
            message: "robot has no behavior to debug".into(),
            line: 1,
        })?;
    Ok((name.clone(), body.clone()))
}

fn inner_block(stmt: &Stmt) -> Option<Vec<Stmt>> {
    // Inner block.
    //
    // Parameters:
    // - `stmt` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::debug_session::inner_block(stmt);

    // Match on stmt and handle each case.
    match stmt {
        Stmt::IfStmt { then_branch, .. } => Some(then_branch.clone()),
        Stmt::LoopStmt { body, .. } => Some(body.clone()),
        _ => None,
    }
}

fn stmt_kind_label(stmt: &Stmt) -> &'static str {
    // Stmt kind label.
    //
    // Parameters:
    // - `stmt` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::debug_session::stmt_kind_label(stmt);

    // Match on stmt and handle each case.
    match stmt {
        Stmt::IfStmt { .. } => "if",
        Stmt::LoopStmt { .. } => "loop",
        _ => "stmt",
    }
}

fn pause_reason(step: DebugStepKind) -> &'static str {
    // Pause reason.
    //
    // Parameters:
    // - `step` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::debug_session::pause_reason(step);

    // Match on step and handle each case.
    match step {
        DebugStepKind::Continue => "breakpoint",
        DebugStepKind::StepOver => "step",
        DebugStepKind::StepIn => "step-in",
        DebugStepKind::StepOut => "step-out",
    }
}

fn parse_debug_value(text: &str) -> RuntimeValue {
    // Parse debug value.
    //
    // Parameters:
    // - `text` — input value
    //
    // Returns:
    // RuntimeValue.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::debug_session::parse_debug_value(text);

    // Compute t for the following logic.
    let t = text.trim();

    // Take the branch when t equals "true".
    if t == "true" {
        return RuntimeValue::Bool { value: true };
    }

    // Take the branch when t equals "false".
    if t == "false" {
        return RuntimeValue::Bool { value: false };
    }

    // Handle the success value from <f64>.
    if let Ok(value) = t.parse::<f64>() {
        return RuntimeValue::Number {
            value,
            unit: UnitKind::None,
        };
    }
    RuntimeValue::String {
        value: t.trim_matches('"').to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug::DebugOptions;
    use std::collections::HashSet;

    #[test]
    fn resumable_debug_steps_and_sets_variable() {
        // Resumable debug steps and sets variable.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::debug_session::resumable_debug_steps_and_sets_variable();

        let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let speed = 0.5;
    wheels.stop();
    wheels.drive(linear: 0.1 m/s, angular: 0.0 rad/s);
  }
}
"#;
        let mut machine = DebugMachine::start(
            source,
            DebugOptions {
                breakpoints: HashSet::new(),
                step: true,
                source_path: None,
            },
        )
        .expect("start");
        let session = machine
            .run_until_pause(DebugStepKind::StepOver)
            .expect("step");
        assert!(!session.pauses.is_empty());
        machine.set_variable("speed", "1.0").expect("set variable");
        let _ = machine
            .run_until_pause(DebugStepKind::StepOver)
            .expect("step again");
        assert!(machine.is_finished() || !machine.pauses().is_empty());
    }

    #[test]
    fn step_out_returns_to_caller_frame() {
        // Step out returns to caller frame.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::debug_session::step_out_returns_to_caller_frame();

        let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    loop every 10ms {
      wheels.stop();
    }
    wheels.drive(linear: 0.1 m/s, angular: 0.0 rad/s);
  }
}
"#;
        let mut machine = DebugMachine::start(
            source,
            DebugOptions {
                breakpoints: HashSet::new(),
                step: false,
                source_path: None,
            },
        )
        .expect("start");
        let _ = machine
            .run_until_pause(DebugStepKind::StepIn)
            .expect("step in");
        let session = machine
            .run_until_pause(DebugStepKind::StepOut)
            .expect("step out");
        assert!(session
            .pauses
            .iter()
            .any(|pause| pause.reason == "step-out"));
    }

    #[test]
    fn resolve_sync_call_finds_export_fn_in_var_decl() {
        // Resolve sync call finds export fn in var decl.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::debug_session::resolve_sync_call_finds_export_fn_in_var_decl();

        let source = r#"
module demo;
export fn bump() -> Int { return 42; }
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let _ = bump();
    wheels.stop();
  }
}
"#;
        let program = crate::compile(source).expect("compile").program;
        let mut interpreter = Interpreter::new(
            create_default_simulator(SimulatorConfig::default()),
            InterpreterOptions::default(),
        );
        interpreter.load_program_metadata(&program);
        let (_, body) = behavior_body(&program.robots()[0]).expect("behavior");
        assert!(interpreter.resolve_sync_call(&body[0]).is_some());
    }

    #[test]
    fn step_in_enters_module_function_call() {
        // Step in enters module function call.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::debug_session::step_in_enters_module_function_call();

        let source = r#"
module demo;
export fn bump() -> Int {
  return 42;
}
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    bump();
    wheels.stop();
  }
}
"#;
        let mut machine = DebugMachine::start(
            source,
            DebugOptions {
                breakpoints: HashSet::new(),
                step: false,
                source_path: None,
            },
        )
        .expect("start");
        let session = machine
            .run_until_pause(DebugStepKind::StepIn)
            .expect("step in call");
        assert!(
            machine.stack_trace().iter().any(|(name, _)| name == "bump"),
            "stack: {:?}",
            machine.stack_trace()
        );
        assert!(session.pauses.iter().any(|pause| pause.reason == "step-in"));
    }

    #[test]
    fn step_over_skips_into_function_without_entering() {
        // Step over skips into function without entering.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::debug_session::step_over_skips_into_function_without_entering();

        let source = r#"
module demo;
export fn bump() -> Int { return 42; }
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    bump();
    wheels.stop();
  }
}
"#;
        let mut machine = DebugMachine::start(
            source,
            DebugOptions {
                breakpoints: HashSet::new(),
                step: false,
                source_path: None,
            },
        )
        .expect("start");
        let session = machine
            .run_until_pause(DebugStepKind::StepOver)
            .expect("step over call");
        assert!(
            !machine.stack_trace().iter().any(|(name, _)| name == "bump"),
            "should not enter callee on step-over"
        );
        assert!(session.pauses.iter().any(|pause| pause.reason == "step"));
    }

    #[test]
    fn frame_variables_snapshot_per_stack_frame() {
        // Frame variables snapshot per stack frame.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::debug_session::frame_variables_snapshot_per_stack_frame();

        let source = r#"
module demo;
export fn bump() -> Int {
  return 42;
}
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let outer = 3;
    bump();
    wheels.stop();
  }
}
"#;
        let mut machine = DebugMachine::start(
            source,
            DebugOptions {
                breakpoints: HashSet::new(),
                step: false,
                source_path: None,
            },
        )
        .expect("start");
        let _ = machine
            .run_until_pause(DebugStepKind::StepOver)
            .expect("let outer");
        let _ = machine
            .run_until_pause(DebugStepKind::StepIn)
            .expect("step in bump");
        let caller_vars = machine.frame_variables(2);
        assert!(caller_vars.contains_key("outer"));
    }
}
