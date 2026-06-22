//! Digital twin runtime method dispatch for the interpreter.
//!

use super::{
    get_number, get_string, IntoSpandaError, Interpreter, RobotBackend, RuntimeError, RuntimeValue,
};
use spanda_ast::nodes::{Expr, LiteralValue, UnitKind};
use spanda_error::SpandaError;
use spanda_runtime::twin::TwinRuntime;

impl<B: RobotBackend> Interpreter<B> {
    fn twin_runtime(&self, line: u32) -> Result<&TwinRuntime, SpandaError> {
        // Borrow the configured twin runtime or return a structured error.
        //
        // Parameters:
        // - `self` — interpreter state
        // - `line` — source line for diagnostics
        //
        // Returns:
        // Twin runtime reference when configured.
        //
        // Options:
        // None.
        //
        // Example:
        // let twin = self.twin_runtime(line)?;

        self.twin.as_ref().ok_or_else(|| {
            RuntimeError::new("No digital twin configured", line).into_spanda()
        })
    }

    pub(super) fn eval_twin_method(
        &mut self,
        method: &str,
        args: &[Expr],
        named_args: &[spanda_ast::nodes::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Eval twin method.
        //
        // Parameters:
        // - `self` — method receiver
        // - `method` — input value
        // - `args` — input value
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
        // let result = instance.eval_twin_method(method, args, named_args, line);

        // Require a configured twin before dispatching methods.
        let _ = self.twin_runtime(line)?;
        self.refresh_twin_shadow_from_backend();

        // Match on method and handle each case.
        match method {
            "frame_count" => {
                let count = self.twin_runtime(line)?.replay_frame_count();
                Ok(RuntimeValue::Number {
                    value: count as f64,
                    unit: UnitKind::None,
                })
            }
            "mirror" => {
                let field = self.twin_field_name(args, named_args, line)?;
                self.twin_runtime(line)?
                    .shadow_field(&field)
                    .cloned()
                    .ok_or_else(|| {
                        RuntimeError::new(
                            format!("Twin has no mirrored shadow field '{field}'"),
                            line,
                        )
                        .into_spanda()
                    })
            }
            "replay" => {
                let twin = self.twin_runtime(line)?;
                // Take the branch when replay is false.
                if !twin.replay {
                    return Err(RuntimeError::new(
                        "Twin replay is disabled — set replay true in twin block",
                        line,
                    )
                    .into_spanda());
                }
                let index =
                    get_number(&self.get_named_arg_value(named_args, "index")?, 0.0) as usize;
                let field = self.twin_field_name(args, named_args, line)?;
                self.twin_runtime(line)?
                    .replay_field(index, &field)
                    .cloned()
                    .ok_or_else(|| {
                        RuntimeError::new(
                            format!("Twin replay frame {index} has no field '{field}'"),
                            line,
                        )
                        .into_spanda()
                    })
            }
            method => {
                let twin = self.twin_runtime(line)?;
                // Take this path when the twin mirrors the requested field.
                if twin.mirrors.iter().any(|m| m == method) {
                    twin.shadow_field(method)
                        .cloned()
                        .ok_or_else(|| {
                            RuntimeError::new(
                                format!("Twin shadow field '{method}' not yet mirrored"),
                                line,
                            )
                            .into_spanda()
                        })
                } else {
                    Ok(RuntimeValue::Void)
                }
            }
        }
    }

    fn twin_field_name(
        &mut self,
        args: &[Expr],
        named_args: &[spanda_ast::nodes::NamedArg],
        line: u32,
    ) -> Result<String, SpandaError> {
        // Twin field name.
        //
        // Parameters:
        // - `self` — method receiver
        // - `args` — input value
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
        // let result = instance.twin_field_name(args, named_args, line);

        // Apply each command-line argument.
        for arg in named_args {
            // Take the branch when name equals "field".
            if arg.name == "field" {
                return self.twin_field_from_expr(&arg.value, line);
            }
        }

        // Emit output when first provides a arg.
        if let Some(arg) = args.first() {
            return self.twin_field_from_expr(arg, line);
        }
        Err(RuntimeError::new("Expected 'field' argument for twin method", line).into_spanda())
    }

    fn twin_field_from_expr(&mut self, expr: &Expr, _line: u32) -> Result<String, SpandaError> {
        // Twin field from expr.
        //
        // Parameters:
        // - `self` — method receiver
        // - `expr` — input value
        // - `_line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.twin_field_from_expr(expr, _line);

        // Match on expr and handle each case.
        match expr {
            Expr::LiteralExpr {
                value: LiteralValue::String(s),
                ..
            } => Ok(s.clone()),
            Expr::IdentExpr { name, .. } => Ok(name.clone()),
            _ => Ok(get_string(&self.eval_expr(expr)?, "")),
        }
    }
}
