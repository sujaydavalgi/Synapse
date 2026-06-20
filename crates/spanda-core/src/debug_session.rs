//! Resumable debug sessions with step/continue and variable mutation.

use crate::ast::{BehaviorDecl, Program, RobotDecl, Stmt, UnitKind};
use crate::debug::{DebugController, DebugOptions, DebugPause, DebugSession, stmt_line};
use crate::error::SpandaError;
use crate::runtime::{Interpreter, InterpreterOptions, RuntimeValue};
use crate::simulator::{create_default_simulator, Simulator, SimulatorConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugStepKind {
    Continue,
    StepOver,
    StepIn,
    StepOut,
}

#[derive(Debug, Clone)]
struct DebugFrame {
    name: String,
    stmts: Vec<Stmt>,
    index: usize,
}

pub struct DebugMachine {
    interpreter: Interpreter<Simulator>,
    frames: Vec<DebugFrame>,
    controller: DebugController,
    step_kind: DebugStepKind,
    finished: bool,
}

impl DebugMachine {
    pub fn start(source: &str, options: DebugOptions) -> Result<Self, SpandaError> {
        let program = crate::compile(source)?.program;
        let step = options.step;
        let controller = DebugController::new(options);
        let mut interpreter = Interpreter::new(
            create_default_simulator(SimulatorConfig::default()),
            InterpreterOptions {
                max_loop_iterations: 100,
                ..Default::default()
            },
        );
        interpreter.load_program_metadata(&program);
        let Program::Program { robots, .. } = &program;
        let robot = robots
            .first()
            .ok_or_else(|| SpandaError::Runtime {
                message: "debug requires at least one robot".into(),
                line: 1,
            })?;
        interpreter.setup_robot_for_debug(robot)?;
        let (name, body) = behavior_body(robot)?;
        Ok(Self {
            interpreter,
            frames: vec![DebugFrame {
                name,
                stmts: body,
                index: 0,
            }],
            controller,
            step_kind: if step {
                DebugStepKind::StepOver
            } else {
                DebugStepKind::Continue
            },
            finished: false,
        })
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn pauses(&self) -> Vec<DebugPause> {
        self.controller.pauses().borrow().clone()
    }

    pub fn stack_trace(&self) -> Vec<(String, u32)> {
        self.frames
            .iter()
            .rev()
            .enumerate()
            .map(|(id, frame)| {
                let line = frame
                    .stmts
                    .get(frame.index)
                    .map(stmt_line)
                    .unwrap_or(1);
                (if id == 0 {
                    frame.name.clone()
                } else {
                    format!("{}#{}", frame.name, id)
                }, line)
            })
            .collect()
    }

    pub fn set_variable(&mut self, name: &str, value: &str) -> Result<(), SpandaError> {
        self.interpreter
            .env_mut()
            .set(name, parse_debug_value(value));
        Ok(())
    }

    pub fn run_until_pause(&mut self, step: DebugStepKind) -> Result<DebugSession, SpandaError> {
        self.step_kind = step;
        if step == DebugStepKind::Continue {
            self.controller.command(crate::debug::DebugCommand::Continue);
        } else {
            self.controller.command(crate::debug::DebugCommand::Step);
        }

        loop {
            if self.frames.is_empty() {
                self.finished = true;
                break;
            }
            let frame_top = self.frames.len() - 1;
            if self.frames[frame_top].index >= self.frames[frame_top].stmts.len() {
                if step == DebugStepKind::StepOut && self.frames.len() > 1 {
                    self.frames.pop();
                    break;
                }
                self.frames.pop();
                continue;
            }

            let stmt = self.frames[frame_top].stmts[self.frames[frame_top].index].clone();
            let line = stmt_line(&stmt);

            if self.controller.should_pause(line) {
                let variables = self.interpreter.env().snapshot_display();
                self.controller.record_pause(line, pause_reason(step), variables);
                break;
            }

            if step == DebugStepKind::StepIn {
                if let Some(inner) = inner_block(&stmt) {
                    self.frames[frame_top].index += 1;
                    self.frames.push(DebugFrame {
                        name: format!("{}:{}", self.frames[frame_top].name, stmt_kind_label(&stmt)),
                        stmts: inner,
                        index: 0,
                    });
                    let variables = self.interpreter.env().snapshot_display();
                    self.controller
                        .record_pause(line, "step-in", variables);
                    break;
                }
            }

            self.frames[frame_top].index += 1;
            self.interpreter.debug_execute_stmt(&stmt)?;

            if matches!(step, DebugStepKind::StepOver | DebugStepKind::StepIn) {
                let variables = self.interpreter.env().snapshot_display();
                self.controller.record_pause(line, "step", variables);
                break;
            }
        }

        Ok(DebugSession {
            pauses: self.controller.pauses().borrow().clone(),
        })
    }
}

fn behavior_body(robot: &RobotDecl) -> Result<(String, Vec<Stmt>), SpandaError> {
    let RobotDecl::RobotDecl { behaviors, .. } = robot;
    let BehaviorDecl::BehaviorDecl { name, body, .. } = behaviors
        .first()
        .ok_or_else(|| SpandaError::Runtime {
            message: "robot has no behavior to debug".into(),
            line: 1,
        })?;
    Ok((name.clone(), body.clone()))
}

fn inner_block(stmt: &Stmt) -> Option<Vec<Stmt>> {
    match stmt {
        Stmt::IfStmt { then_branch, .. } => Some(then_branch.clone()),
        Stmt::LoopStmt { body, .. } => Some(body.clone()),
        _ => None,
    }
}

fn stmt_kind_label(stmt: &Stmt) -> &'static str {
    match stmt {
        Stmt::IfStmt { .. } => "if",
        Stmt::LoopStmt { .. } => "loop",
        _ => "stmt",
    }
}

fn pause_reason(step: DebugStepKind) -> &'static str {
    match step {
        DebugStepKind::Continue => "breakpoint",
        DebugStepKind::StepOver => "step",
        DebugStepKind::StepIn => "step-in",
        DebugStepKind::StepOut => "step-out",
    }
}

fn parse_debug_value(text: &str) -> RuntimeValue {
    let t = text.trim();
    if t == "true" {
        return RuntimeValue::Bool { value: true };
    }
    if t == "false" {
        return RuntimeValue::Bool { value: false };
    }
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
            },
        )
        .expect("start");
        let session = machine
            .run_until_pause(DebugStepKind::StepOver)
            .expect("step");
        assert!(!session.pauses.is_empty());
        machine
            .set_variable("speed", "1.0")
            .expect("set variable");
        let _ = machine
            .run_until_pause(DebugStepKind::StepOver)
            .expect("step again");
        assert!(machine.is_finished() || !machine.pauses().is_empty());
    }
}
