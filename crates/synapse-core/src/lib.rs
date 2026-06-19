pub mod ast;
pub mod lexer;
pub mod parser;
pub mod types;
pub mod runtime;
pub mod safety;
pub mod ai;
pub mod simulator;
pub mod hal;
pub mod soc;
pub mod lib_registry;
mod error;

pub use ast::*;
pub use error::*;

use runtime::{Interpreter, InterpreterOptions, RobotBackend};
use simulator::{create_default_simulator, Obstacle, SimulatorConfig};
use std::cell::RefCell;
use std::rc::Rc;

pub fn compile(source: &str) -> Result<CompileResult, SynapseError> {
    let tokens = lexer::tokenize(source)?;
    let program = parser::parse(tokens)?;
    types::type_check(&program)?;
    Ok(CompileResult {
        program,
        source: source.to_string(),
    })
}

pub fn check(source: &str) -> Result<(), SynapseError> {
    let tokens = lexer::tokenize(source)?;
    let program = parser::parse(tokens)?;
    types::check(&program)
}

pub fn run(source: &str, options: RunOptions) -> Result<RunResult, SynapseError> {
    let compiled = compile(source)?;
    run_program(&compiled.program, options)
}

pub fn run_program(program: &Program, options: RunOptions) -> Result<RunResult, SynapseError> {
    let obstacles: Vec<Obstacle> = options
        .obstacles
        .iter()
        .map(|o| Obstacle {
            x: o.x,
            y: o.y,
            radius: o.radius,
        })
        .collect();

    let initial_pose = options.initial_pose.clone().unwrap_or(PoseState {
        x: 0.0,
        y: 0.0,
        theta: 0.0,
        z: Some(0.0),
    });

    let sim_config = SimulatorConfig {
        obstacles: if obstacles.is_empty() {
            SimulatorConfig::default().obstacles
        } else {
            obstacles
        },
        initial_pose,
        lidar_range: options.lidar_range,
    };

    let sim = create_default_simulator(sim_config);
    let logs: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let logs_cb = logs.clone();

    let mut interp = Interpreter::new(
        sim,
        InterpreterOptions {
            max_loop_iterations: options.max_loop_iterations,
            on_log: Some(Rc::new(move |msg| logs_cb.borrow_mut().push(msg))),
            on_motion_blocked: None,
        },
    );

    let state = interp.run(program, options.entry_behavior.as_deref())?;
    let events = interp.robot_backend().event_log();

    let run_logs = logs.borrow().clone();

    Ok(RunResult {
        state,
        events,
        logs: run_logs,
    })
}
