//! Spawn targets, async futures, and task-handle resolution.
//!

use super::{IntoSpandaError, Interpreter, RobotBackend, RuntimeError, RuntimeValue};
use spanda_ast::nodes::Expr;
use spanda_error::SpandaError;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn call_module_function(
        &mut self,
        func: &spanda_ast::foundations::ModuleFnDecl,
        args: &[Expr],
        _line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Call module function.
        //
        // Parameters:
        // - `self` — method receiver
        // - `func` — input value
        // - `args` — input value
        // - `_line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.call_module_function(func, args, _line);

        // Save current variable bindings before the call.
        let saved = self.env.clone_bindings();

        // Bind each formal parameter to its call argument.
        for (i, param) in func.params.iter().enumerate() {
            // Emit output when get provides a arg.
            if let Some(arg) = args.get(i) {
                let val = self.eval_expr(arg)?;
                self.env.define(param.name.clone(), val);
            }
        }
        let result = self
            .execute_block_with_return(&func.body)?
            .unwrap_or(RuntimeValue::Void);
        self.env = saved;
        Ok(result)
    }

    pub(super) fn resolve_future(
        &mut self,
        future: RuntimeValue,
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Resolve future.
        //
        // Parameters:
        // - `self` — method receiver
        // - `future` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.resolve_future(future, line);

        // Match on future and handle each case.
        match future {
            RuntimeValue::Future {
                resolved: Some(value),
                ..
            } => Ok(*value),
            RuntimeValue::Future {
                func_name,
                args,
                resolved: None,
                ..
            } => {
                let func = self
                    .module_functions
                    .get(&func_name)
                    .or_else(|| self.imported_functions.get(&func_name))
                    .cloned()
                    .ok_or_else(|| {
                        RuntimeError::new(format!("Unknown async function '{func_name}'"), line)
                            .into_spanda()
                    })?;
                let saved = self.env.clone_bindings();

                // Bind each formal parameter to its call argument.
                for (i, param) in func.params.iter().enumerate() {
                    // Emit output when get provides a val.
                    if let Some(val) = args.get(i) {
                        self.env.define(param.name.clone(), val.clone());
                    }
                }
                let result = self
                    .execute_block_with_return(&func.body)?
                    .unwrap_or(RuntimeValue::Void);
                self.env = saved;
                Ok(result)
            }
            other => Ok(other),
        }
    }

    pub(super) fn eval_spawn_target(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        line: u32,
    ) -> Result<(String, Vec<RuntimeValue>), SpandaError> {
        // Eval spawn target.
        //
        // Parameters:
        // - `self` — method receiver
        // - `callee` — input value
        // - `args` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.eval_spawn_target(callee, args, line);

        // Create mutable arg values for accumulating results.
        let mut arg_values = Vec::new();

        // Apply each command-line argument.
        for arg in args {
            arg_values.push(self.eval_expr(arg)?);
        }
        let name = match callee {
            Expr::IdentExpr { name, .. } => name.clone(),
            _ => return Err(RuntimeError::new("spawn requires function name", line).into_spanda()),
        };
        Ok((name, arg_values))
    }

    fn execute_spawn_job(
        &mut self,
        func_name: &str,
        args: &[RuntimeValue],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Execute spawn job.
        //
        // Parameters:
        // - `self` — method receiver
        // - `func_name` — input value
        // - `args` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_spawn_job(func_name, args, line);

        // Compute func for the following logic.
        let func = self
            .module_functions
            .get(func_name)
            .or_else(|| self.imported_functions.get(func_name))
            .cloned()
            .ok_or_else(|| {
                RuntimeError::new(format!("Unknown spawn target '{func_name}'"), line).into_spanda()
            })?;
        let saved = self.env.clone_bindings();

        // Bind each formal parameter to its call argument.
        for (i, param) in func.params.iter().enumerate() {
            // Emit output when get provides a val.
            if let Some(val) = args.get(i) {
                self.env.define(param.name.clone(), val.clone());
            }
        }
        let result = self
            .execute_block_with_return(&func.body)?
            .unwrap_or(RuntimeValue::Void);
        self.env = saved;
        Ok(result)
    }

    pub(super) fn resolve_task_handle(&mut self, id: u64, line: u32) -> Result<RuntimeValue, SpandaError> {
        // Resolve task handle.
        //
        // Parameters:
        // - `self` — method receiver
        // - `id` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.resolve_task_handle(id, line);

        // use result when clone is present.

        // Emit output when clone provides a result.
        if let Some(result) = self.concurrency.handle(id).and_then(|h| h.result.clone()) {
            return Ok(result);
        }
        let result = self.execute_spawn_handle(id, line)?;
        self.telemetry.record_join();
        self.trace_task_log(format!("join handle {id} -> completed"));
        Ok(result)
    }

    fn execute_spawn_handle(&mut self, id: u64, line: u32) -> Result<RuntimeValue, SpandaError> {
        // Execute spawn handle.
        //
        // Parameters:
        // - `self` — method receiver
        // - `id` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.execute_spawn_handle(id, line);

        // use result when clone is present.

        // Emit output when clone provides a result.
        if let Some(result) = self.concurrency.handle(id).and_then(|h| h.result.clone()) {
            return Ok(result);
        }
        let (func_name, args) = {
            let handle = self.concurrency.handle(id).ok_or_else(|| {
                RuntimeError::new(format!("Unknown task handle id {id}"), line).into_spanda()
            })?;
            (handle.func_name.clone(), handle.args.clone())
        };
        let result = self.execute_spawn_job(&func_name, &args, line)?;
        self.concurrency.set_handle_result(id, result.clone());
        Ok(result)
    }

    pub(super) fn process_spawn_queue(&mut self) -> Result<(), SpandaError> {
        // Process spawn queue.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.process_spawn_queue();

        // Compute ids for the following logic.
        let ids = self.concurrency.drain_fire_and_forget_queue();

        // Iterate over ids.
        for id in ids {
            self.execute_spawn_handle(id, 0)?;
        }
        Ok(())
    }

}
