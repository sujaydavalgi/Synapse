//! Audit and ledger runtime method dispatch.
//!

use super::{get_string, IntoSpandaError, Interpreter, RobotBackend, RuntimeError, RuntimeValue};
use spanda_ast::nodes::{Expr, UnitKind};
use spanda_error::SpandaError;
use std::collections::HashMap;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn eval_audit_method(
        &mut self,
        method: &str,
        args: &[Expr],
        _named_args: &[spanda_ast::nodes::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Eval audit method.
        //
        // Parameters:
        // - `self` — method receiver
        // - `method` — input value
        // - `args` — input value
        // - `_named_args` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.eval_audit_method(method, args, _named_args, line);

        // Match on method and handle each case.
        match method {
            "record" => {
                self.security
                    .require_operation("audit.record")
                    .map_err(|e| self.security_error(e, line))?;
                let event_type = if let Some(arg) = args.first() {
                    get_string(&self.eval_expr(arg)?, "event")
                } else {
                    "event".into()
                };
                let payload = if args.len() > 1 {
                    Self::runtime_value_payload(&self.eval_expr(&args[1])?)
                } else {
                    String::new()
                };
                let rt = self.audit_runtime.as_mut().ok_or_else(|| {
                    RuntimeError::new(
                        "Audit not configured — declare an audit block on the robot",
                        line,
                    )
                    .into_spanda()
                })?;
                let id = rt.record_event(&event_type, &payload).map_err(|e| {
                    RuntimeError::new(format!("audit.record failed: {e}"), line).into_spanda()
                })?;
                let _ = self.security.audit_event(rt, "audit.record", &event_type);
                self.log(format!("audit.record({event_type}) -> {}", id.0));
                Ok(RuntimeValue::Object {
                    type_name: "RecordId".into(),
                    fields: HashMap::from([("id".into(), RuntimeValue::String { value: id.0 })]),
                })
            }
            "export" => {
                self.security
                    .require_operation("audit.read")
                    .map_err(|e| self.security_error(e, line))?;
                let rt = self.audit_runtime.as_mut().ok_or_else(|| {
                    RuntimeError::new(
                        "Audit not configured — declare an audit block on the robot",
                        line,
                    )
                    .into_spanda()
                })?;
                let json = rt.export_json().map_err(|e| {
                    RuntimeError::new(format!("audit.export failed: {e}"), line).into_spanda()
                })?;
                Ok(RuntimeValue::String { value: json })
            }
            "count" => {
                let count = self
                    .audit_runtime
                    .as_ref()
                    .map(|rt| rt.record_count())
                    .unwrap_or(0);
                Ok(RuntimeValue::Number {
                    value: count as f64,
                    unit: UnitKind::None,
                })
            }
            "root_hash" => {
                let hash = self
                    .audit_runtime
                    .as_ref()
                    .and_then(|rt| rt.root_hash())
                    .map(|h| h.0)
                    .unwrap_or_default();
                Ok(RuntimeValue::Object {
                    type_name: "Hash".into(),
                    fields: HashMap::from([("hex".into(), RuntimeValue::String { value: hash })]),
                })
            }
            "create_provenance" => {
                self.security
                    .require_operation("identity.sign")
                    .map_err(|e| self.security_error(e, line))?;
                let name = if let Some(arg) = args.first() {
                    get_string(&self.eval_expr(arg)?, "provenance")
                } else {
                    "provenance".into()
                };
                let record_id = if args.len() > 1 {
                    // Match on eval expr and handle each case.
                    match self.eval_expr(&args[1])? {
                        RuntimeValue::Object { fields, .. } => fields
                            .get("id")
                            .and_then(|v| match v {
                                RuntimeValue::String { value } => {
                                    Some(spanda_audit::RecordId(value.clone()))
                                }
                                _ => None,
                            })
                            .unwrap_or_else(|| spanda_audit::RecordId("audit-1".into())),
                        _ => spanda_audit::RecordId("audit-1".into()),
                    }
                } else {
                    spanda_audit::RecordId("audit-1".into())
                };
                let rt = self.audit_runtime.as_ref().ok_or_else(|| {
                    RuntimeError::new(
                        "Audit not configured — declare an audit block on the robot",
                        line,
                    )
                    .into_spanda()
                })?;
                let prov = rt.create_provenance(&name, &record_id).map_err(|e| {
                    RuntimeError::new(format!("audit.create_provenance failed: {e}"), line)
                        .into_spanda()
                })?;
                Ok(RuntimeValue::Object {
                    type_name: "ProvenanceRecord".into(),
                    fields: HashMap::from([
                        (
                            "name".into(),
                            RuntimeValue::String {
                                value: prov.name.clone(),
                            },
                        ),
                        (
                            "record_id".into(),
                            RuntimeValue::Object {
                                type_name: "RecordId".into(),
                                fields: HashMap::from([(
                                    "id".into(),
                                    RuntimeValue::String {
                                        value: prov.record_id.0.clone(),
                                    },
                                )]),
                            },
                        ),
                        (
                            "signed_by".into(),
                            RuntimeValue::String {
                                value: prov.signed_by.clone(),
                            },
                        ),
                    ]),
                })
            }
            _ => Ok(RuntimeValue::Void),
        }
    }

    pub(super) fn eval_ledger_method(
        &mut self,
        method: &str,
        args: &[Expr],
        _named_args: &[spanda_ast::nodes::NamedArg],
        line: u32,
    ) -> Result<RuntimeValue, SpandaError> {
        // Eval ledger method.
        //
        // Parameters:
        // - `self` — method receiver
        // - `method` — input value
        // - `args` — input value
        // - `_named_args` — input value
        // - `line` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.eval_ledger_method(method, args, _named_args, line);

        // Import the items needed by the logic below.
        use spanda_audit::LedgerBackend;

        // Match on method and handle each case.
        match method {
            "anchor" => {
                self.security
                    .require_operation("ledger.anchor")
                    .map_err(|e| self.security_error(e, line))?;
                let hash_hex = if let Some(arg) = args.first() {
                    // Match on eval expr and handle each case.
                    match self.eval_expr(arg)? {
                        RuntimeValue::Object { fields, .. } => fields
                            .get("hex")
                            .and_then(|v| match v {
                                RuntimeValue::String { value } => Some(value.clone()),
                                _ => None,
                            })
                            .unwrap_or_default(),
                        RuntimeValue::String { value } => value,
                        _ => String::new(),
                    }
                } else {
                    String::new()
                };
                let hash = spanda_audit::Hash(hash_hex);
                let tx = self.mock_ledger.anchor_hash(&hash).map_err(|e| {
                    RuntimeError::new(format!("mock_ledger.anchor failed: {e}"), line).into_spanda()
                })?;
                self.log(format!("mock_ledger.anchor -> {}", tx.0));
                Ok(RuntimeValue::Object {
                    type_name: "TransactionId".into(),
                    fields: HashMap::from([("id".into(), RuntimeValue::String { value: tx.0 })]),
                })
            }
            "verify" => {
                let hash_hex = if let Some(arg) = args.first() {
                    // Match on eval expr and handle each case.
                    match self.eval_expr(arg)? {
                        RuntimeValue::Object { fields, .. } => fields
                            .get("hex")
                            .and_then(|v| match v {
                                RuntimeValue::String { value } => Some(value.clone()),
                                _ => None,
                            })
                            .unwrap_or_default(),
                        RuntimeValue::String { value } => value,
                        _ => String::new(),
                    }
                } else {
                    String::new()
                };
                let hash = spanda_audit::Hash(hash_hex);
                let ok = self.mock_ledger.verify_anchor(&hash).map_err(|e| {
                    RuntimeError::new(format!("mock_ledger.verify failed: {e}"), line).into_spanda()
                })?;
                Ok(RuntimeValue::Bool { value: ok })
            }
            _ => Ok(RuntimeValue::Void),
        }
    }

}
