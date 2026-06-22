//! World model runtime method dispatch.
//!

use super::{Interpreter, RobotBackend, RuntimeError, RuntimeValue};
use spanda_ast::nodes::{Expr, UnitKind};
use spanda_error::SpandaError;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn eval_world_model_method(
        &mut self,
        method: &str,
        args: &[Expr],
        _named_args: &[spanda_ast::nodes::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Dispatch `world_model.update`, `belief`, and `export` calls.
        //
        // Parameters:
        // - `method` — method name on `world_model`
        // - `args` — call arguments
        // - `line` — source line for diagnostics
        //
        // Returns:
        // Method result or a runtime error.
        //
        // Options:
        // None.
        //
        // Example:
        // world_model.update(observation);

        match method {
            "update" => {
                let observation = if let Some(arg) = args.first() {
                    self.eval_expr(arg)?
                } else {
                    return Err(
                        RuntimeError::new("world_model.update requires an observation", line)
                            .into(),
                    );
                };
                let confidence = self.world_model.update(&observation);
                self.log(format!("world_model.update -> belief {confidence:.2}"));
                Ok(RuntimeValue::Number {
                    value: confidence,
                    unit: UnitKind::None,
                })
            }
            "belief" => Ok(RuntimeValue::Number {
                value: self.world_model.belief_confidence(),
                unit: UnitKind::None,
            }),
            "export" => {
                let json = self.world_model.export_json().to_string();
                Ok(RuntimeValue::String { value: json })
            }
            other => Err(RuntimeError::new(
                format!("unknown world_model method: {other}"),
                line,
            )
            .into()),
        }
    }
}
