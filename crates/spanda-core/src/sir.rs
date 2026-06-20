//! Spanda IR (SIR) — typed intermediate representation between AST and backends.
//!
//! Lowers module functions, extern declarations, robot metadata, and a subset of
//! statements for LLVM codegen. Execution still uses the tree-walking interpreter.

use crate::ast::{
    BehaviorDecl, Expr, LiteralValue, NamedArg, Program, RobotDecl, SpandaType, Stmt, UnitKind,
};
use crate::foundations::{BridgeKind, EnumDecl, ExternFnDecl, ModuleFnDecl, Visibility};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SirProgram {
    pub module_name: Option<String>,
    pub imports: Vec<String>,
    pub functions: Vec<SirFunction>,
    pub externs: Vec<SirExtern>,
    pub robot_names: Vec<String>,
    pub behavior_names: Vec<String>,
    pub robots: Vec<SirRobot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SirRobot {
    pub name: String,
    pub behaviors: Vec<SirBehavior>,
    pub task_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SirBehavior {
    pub name: String,
    pub stmt_count: usize,
    pub body: Vec<SirStmt>,
    pub has_requires: bool,
    pub has_ensures: bool,
    pub has_invariant: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SirFunction {
    pub name: String,
    pub visibility: SirVisibility,
    pub type_params: Vec<String>,
    pub params: Vec<SirParam>,
    pub return_type: String,
    pub is_async: bool,
    pub body: Vec<SirStmt>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SirStmt {
    ActuatorDrive {
        actuator: String,
        linear: f64,
        angular: f64,
    },
    ActuatorStop {
        actuator: String,
    },
    EmergencyStop,
    ReturnInt {
        value: i64,
    },
    ReturnFloat {
        value: f64,
    },
    ReturnVoid,
    LetInt {
        name: String,
        value: i64,
    },
    LetBool {
        name: String,
        value: bool,
    },
    LetEnumUnit {
        name: String,
        enum_name: String,
        variant: String,
        tag: u32,
    },
    LetEnumPayload {
        name: String,
        enum_name: String,
        variant: String,
        tag: u32,
        payloads: Vec<f64>,
    },
    LoopEvery {
        interval_ms: f64,
        body: Vec<SirStmt>,
    },
    Publish {
        topic: String,
        payload: Option<String>,
    },
    IfBool {
        condition: bool,
        then_body: Vec<SirStmt>,
        else_body: Option<Vec<SirStmt>>,
    },
    IfVar {
        condition: String,
        then_body: Vec<SirStmt>,
        else_body: Option<Vec<SirStmt>>,
    },
    IfCompareBool {
        variable: String,
        equals: bool,
        then_body: Vec<SirStmt>,
        else_body: Option<Vec<SirStmt>>,
    },
    IfNotVar {
        variable: String,
        then_body: Vec<SirStmt>,
        else_body: Option<Vec<SirStmt>>,
    },
    LetDouble {
        name: String,
        value: f64,
    },
    LetString {
        name: String,
        value: String,
    },
    IfCompareDouble {
        left: String,
        op: SirCompareOp,
        right: f64,
        then_body: Vec<SirStmt>,
        else_body: Option<Vec<SirStmt>>,
    },
    IfScanDistanceCompare {
        scan_var: String,
        op: SirCompareOp,
        threshold: f64,
        then_body: Vec<SirStmt>,
        else_body: Option<Vec<SirStmt>>,
    },
    IfRuntime {
        condition: String,
        then_body: Vec<SirStmt>,
        else_body: Option<Vec<SirStmt>>,
    },
    MatchEnumUnit {
        scrutinee: String,
        enum_name: String,
        arms: Vec<SirMatchArm>,
    },
    Subscribe {
        target: String,
    },
    Unsupported {
        #[serde(rename = "stmt_kind")]
        label: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SirCompareOp {
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    Neq,
}

/// Serializable condition tree for LLVM runtime `if` fallback.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum SirCondition {
    Bool {
        value: bool,
    },
    Ident {
        name: String,
    },
    Not {
        operand: Box<SirCondition>,
    },
    And {
        left: Box<SirCondition>,
        right: Box<SirCondition>,
    },
    Or {
        left: Box<SirCondition>,
        right: Box<SirCondition>,
    },
    EqBool {
        name: String,
        value: bool,
    },
    NeqBool {
        name: String,
        value: bool,
    },
    EqString {
        name: String,
        value: String,
    },
    CompareDouble {
        name: String,
        cmp: SirCompareOp,
        value: f64,
    },
    ScanDistance {
        scan_var: String,
        cmp: SirCompareOp,
        threshold: f64,
    },
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SirMatchArm {
    pub variant: String,
    pub tag: u32,
    #[serde(default)]
    pub bindings: Vec<String>,
    pub body: Vec<SirStmt>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SirVisibility {
    Private,
    Public,
    Export,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SirParam {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SirExtern {
    pub name: String,
    pub library: Option<String>,
    pub bridge: BridgeKind,
    pub params: Vec<SirParam>,
    pub return_type: String,
}

pub fn lower_program(program: &Program) -> SirProgram {
    // Lower program.
    //
    // Parameters:
    // - `program` — input value
    //
    // Returns:
    // SirProgram.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::lower_program(program);

    // Compute ctx for the following logic.
    let ctx = LowerCtx::new(program);
    let Program::Program {
        module_name,
        imports,
        functions,
        extern_functions,
        robots,
        ..
    } = program;
    let mut behavior_names = Vec::new();
    let mut sir_robots = Vec::new();

    // Handle each robot declared in the program.
    for robot in robots {
        let RobotDecl::RobotDecl {
            name,
            behaviors,
            tasks,
            ..
        } = robot;
        let mut sir_behaviors = Vec::new();

        // Process each behavior.
        for behavior in behaviors {
            let BehaviorDecl::BehaviorDecl {
                name: behavior_name,
                requires,
                ensures,
                invariant,
                body,
                ..
            } = behavior;
            behavior_names.push(behavior_name.clone());
            sir_behaviors.push(SirBehavior {
                name: behavior_name.clone(),
                stmt_count: body.len(),
                body: ctx.lower_stmts(body),
                has_requires: requires.is_some(),
                has_ensures: ensures.is_some(),
                has_invariant: invariant.is_some(),
            });
        }
        let task_names: Vec<String> = tasks
            .iter()
            .map(|t| match t {
                crate::foundations::TaskDecl::TaskDecl { name, .. } => name.clone(),
            })
            .collect();
        sir_robots.push(SirRobot {
            name: name.clone(),
            behaviors: sir_behaviors,
            task_names,
        });
    }
    SirProgram {
        module_name: module_name.clone(),
        imports: imports
            .iter()
            .map(|i| match i {
                crate::ast::ImportDecl::ImportDecl { path, .. } => path.clone(),
            })
            .collect(),
        functions: functions
            .iter()
            .map(|func| ctx.lower_function(func))
            .collect(),
        externs: extern_functions.iter().map(lower_extern).collect(),
        robot_names: robots
            .iter()
            .map(|r| match r {
                RobotDecl::RobotDecl { name, .. } => name.clone(),
            })
            .collect(),
        behavior_names,
        robots: sir_robots,
    }
}

fn lower_function(func: &ModuleFnDecl, ctx: &LowerCtx) -> SirFunction {
    // Lower function.
    //
    // Parameters:
    // - `func` — input value
    // - `ctx` — input value
    //
    // Returns:
    // SirFunction.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::lower_function(func, ctx);

    // Compute ModuleFnDecl for the following logic.
    let ModuleFnDecl {
        name,
        visibility,
        type_params,
        params,
        return_type,
        is_async,
        body,
        ..
    } = func;
    SirFunction {
        name: name.clone(),
        visibility: match visibility {
            Visibility::Private => SirVisibility::Private,
            Visibility::Public => SirVisibility::Public,
            Visibility::Export => SirVisibility::Export,
        },
        type_params: type_params.clone(),
        params: params
            .iter()
            .map(|p| SirParam {
                name: p.name.clone(),
                type_name: type_to_string(&p.type_ann),
            })
            .collect(),
        return_type: type_to_string(return_type),
        is_async: *is_async,
        body: ctx.lower_stmts(body),
    }
}

struct LowerCtx<'a> {
    variant_index: HashMap<String, (String, u32)>,
    _program: std::marker::PhantomData<&'a Program>,
}

impl LowerCtx<'_> {
    fn new(program: &Program) -> Self {
        // Create a new instance.
        //
        // Parameters:
        // - `program` — input value
        //
        // Returns:
        // A new instance of this type.
        //
        // Options:
        // None.
        //
        // Example:
        // let value = spanda_core::sir::new(program);

        // Create mutable variant index for accumulating results.
        let mut variant_index = HashMap::new();
        let Program::Program { enums, .. } = program;

        // Process each enum.
        for enum_decl in enums {
            let EnumDecl::EnumDecl { name, variants, .. } = enum_decl;

            // Iterate over enumerate with destructured elements.
            for (tag, variant) in variants.iter().enumerate() {
                variant_index.insert(variant.name.clone(), (name.clone(), tag as u32));
            }
        }
        Self {
            variant_index,
            _program: std::marker::PhantomData,
        }
    }

    fn lower_function(&self, func: &ModuleFnDecl) -> SirFunction {
        // Lower function.
        //
        // Parameters:
        // - `self` — method receiver
        // - `func` — input value
        //
        // Returns:
        // SirFunction.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.lower_function(func);

        // Produce lower function as the result.
        lower_function(func, self)
    }

    fn lower_stmts(&self, stmts: &[Stmt]) -> Vec<SirStmt> {
        // Lower stmts.
        //
        // Parameters:
        // - `self` — method receiver
        // - `stmts` — input value
        //
        // Returns:
        // Vec<SirStmt>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.lower_stmts(stmts);

        // Collect filtered entries into a new list.
        stmts.iter().map(|stmt| self.lower_stmt(stmt)).collect()
    }

    fn lower_stmt(&self, stmt: &Stmt) -> SirStmt {
        // Lower stmt.
        //
        // Parameters:
        // - `self` — method receiver
        // - `stmt` — input value
        //
        // Returns:
        // SirStmt.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.lower_stmt(stmt);

        // Match on stmt and handle each case.
        match stmt {
            Stmt::VarDecl { name, init, .. } => {
                // Emit output when init provides a init.
                if let Some(init) = init {
                    // Emit output when float literal provides a value.
                    if let Some(value) = float_literal(init) {
                        // Take this path when value.fract().abs() > f64::EPSILON.
                        if value.fract().abs() > f64::EPSILON {
                            return SirStmt::LetDouble {
                                name: name.clone(),
                                value,
                            };
                        }
                    }

                    // Emit output when int literal provides a value.
                    if let Some(value) = int_literal(init) {
                        return SirStmt::LetInt {
                            name: name.clone(),
                            value,
                        };
                    }

                    // Emit output when bool literal provides a value.
                    if let Some(value) = bool_literal(init) {
                        return SirStmt::LetBool {
                            name: name.clone(),
                            value,
                        };
                    }

                    // Emit output when string literal provides a value.
                    if let Some(value) = string_literal(init) {
                        return SirStmt::LetString {
                            name: name.clone(),
                            value,
                        };
                    }

                    // Take this path when let Some((enum name, variant, tag)) = self.resolve enum unit(init).
                    if let Some((enum_name, variant, tag)) = self.resolve_enum_unit(init) {
                        return SirStmt::LetEnumUnit {
                            name: name.clone(),
                            enum_name,
                            variant,
                            tag,
                        };
                    }

                    // Take this path when let Some((enum name, variant, tag, payloads)) =.
                    if let Some((enum_name, variant, tag, payloads)) =
                        self.resolve_enum_payload(init)
                    {
                        return SirStmt::LetEnumPayload {
                            name: name.clone(),
                            enum_name,
                            variant,
                            tag,
                            payloads,
                        };
                    }

                    // Emit output when float literal provides a value.
                    if let Some(value) = float_literal(init) {
                        return SirStmt::LetDouble {
                            name: name.clone(),
                            value,
                        };
                    }
                }
                SirStmt::Unsupported {
                    label: "var_decl".into(),
                }
            }
            Stmt::ReturnStmt { value, .. } => lower_return(value.as_ref()),
            Stmt::ExprStmt { expr, .. } => self.lower_expr_stmt(expr),
            Stmt::EmergencyStopStmt { .. } => SirStmt::EmergencyStop,
            Stmt::LoopStmt {
                interval_ms, body, ..
            } => SirStmt::LoopEvery {
                interval_ms: *interval_ms,
                body: self.lower_stmts(body),
            },
            Stmt::PublishStmt {
                topic_name, value, ..
            } => SirStmt::Publish {
                topic: topic_name.clone(),
                payload: string_literal(value),
            },
            Stmt::IfStmt {
                condition,
                then_branch,
                else_branch,
                ..
            } => self.lower_if_stmt(condition, then_branch, else_branch.as_ref()),
            Stmt::SubscribeStmt { target, .. } => SirStmt::Subscribe {
                target: target.clone(),
            },
            other => SirStmt::Unsupported {
                label: stmt_kind(other),
            },
        }
    }

    fn lower_if_stmt(
        &self,
        condition: &Expr,
        then_branch: &[Stmt],
        else_branch: Option<&Vec<Stmt>>,
    ) -> SirStmt {
        // Lower if stmt.
        //
        // Parameters:
        // - `self` — method receiver
        // - `condition` — input value
        // - `then_branch` — input value
        // - `else_branch` — input value
        //
        // Returns:
        // SirStmt.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.lower_if_stmt(condition, then_branch, else_branch);

        // Compute then body for the following logic.
        let then_body = self.lower_stmts(then_branch);
        let else_body = else_branch.map(|branch| self.lower_stmts(branch));

        // Emit output when eval const bool provides a condition.
        if let Some(condition) = eval_const_bool(condition) {
            return SirStmt::IfBool {
                condition,
                then_body,
                else_body,
            };
        }

        // Take this path when let Some((left, op, right)) = extract double compare(condition).
        if let Some((left, op, right)) = extract_double_compare(condition) {
            return SirStmt::IfCompareDouble {
                left,
                op,
                right,
                then_body,
                else_body,
            };
        }

        // Take this path when let Some((scan var, op, threshold)) = extract scan distance compare(co.
        if let Some((scan_var, op, threshold)) = extract_scan_distance_compare(condition) {
            return SirStmt::IfScanDistanceCompare {
                scan_var,
                op,
                threshold,
                then_body,
                else_body,
            };
        }

        // Emit output when bool literal provides a condition.
        if let Some(condition) = bool_literal(condition) {
            return SirStmt::IfBool {
                condition,
                then_body,
                else_body,
            };
        }

        // Take this path when let Expr::IdentExpr { name, .. } = condition.
        if let Expr::IdentExpr { name, .. } = condition {
            return SirStmt::IfVar {
                condition: name.clone(),
                then_body,
                else_body,
            };
        }

        // Take this path when let Some((variable, equals)) = compare bool literal(condition).
        if let Some((variable, equals)) = compare_bool_literal(condition) {
            return SirStmt::IfCompareBool {
                variable,
                equals,
                then_body,
                else_body,
            };
        }

        // Take this path when let Some((variable, equals)) = compare bool ne literal(condition).
        if let Some((variable, equals)) = compare_bool_ne_literal(condition) {
            return SirStmt::IfCompareBool {
                variable,
                equals,
                then_body,
                else_body,
            };
        }

        // Take this path when let Expr::UnaryExpr.
        if let Expr::UnaryExpr {
            op: crate::ast::UnaryOp::Not,
            operand,
            ..
        } = condition
        {
            // Take this path when let Expr::IdentExpr { name, .. } = operand.as ref().
            if let Expr::IdentExpr { name, .. } = operand.as_ref() {
                return SirStmt::IfNotVar {
                    variable: name.clone(),
                    then_body,
                    else_body,
                };
            }
        }

        // Take this path when let Expr::BinaryExpr.
        if let Expr::BinaryExpr {
            op: crate::ast::BinaryOp::And,
            left,
            right,
            ..
        } = condition
        {
            // Take this path when let Some((var, cmp op, rhs)) = extract double compare(left).
            if let Some((var, cmp_op, rhs)) = extract_double_compare(left) {
                return SirStmt::IfCompareDouble {
                    left: var,
                    op: cmp_op,
                    right: rhs,
                    then_body: vec![self.lower_if_stmt(right, then_branch, else_branch)],
                    else_body,
                };
            }

            // Take this path when let Some((scan var, cmp op, threshold)) = extract scan distance compar.
            if let Some((scan_var, cmp_op, threshold)) = extract_scan_distance_compare(left) {
                return SirStmt::IfScanDistanceCompare {
                    scan_var,
                    op: cmp_op,
                    threshold,
                    then_body: vec![self.lower_if_stmt(right, then_branch, else_branch)],
                    else_body,
                };
            }

            // Take this path when let (Expr::IdentExpr { name: l, .. }, Expr::IdentExpr { name: r, .. }).
            if let (Expr::IdentExpr { name: l, .. }, Expr::IdentExpr { name: r, .. }) =
                (left.as_ref(), right.as_ref())
            {
                return SirStmt::IfVar {
                    condition: l.clone(),
                    then_body: vec![SirStmt::IfVar {
                        condition: r.clone(),
                        then_body: then_body.clone(),
                        else_body: else_body.clone(),
                    }],
                    else_body,
                };
            }
        }

        // Take this path when let Expr::BinaryExpr.
        if let Expr::BinaryExpr {
            op: crate::ast::BinaryOp::Or,
            left,
            right,
            ..
        } = condition
        {
            // Take this path when let Some((var, cmp op, rhs)) = extract double compare(left).
            if let Some((var, cmp_op, rhs)) = extract_double_compare(left) {
                return SirStmt::IfCompareDouble {
                    left: var,
                    op: cmp_op,
                    right: rhs,
                    then_body,
                    else_body: Some(vec![self.lower_if_stmt(right, then_branch, else_branch)]),
                };
            }

            // Take this path when let Some((scan var, cmp op, threshold)) = extract scan distance compar.
            if let Some((scan_var, cmp_op, threshold)) = extract_scan_distance_compare(left) {
                return SirStmt::IfScanDistanceCompare {
                    scan_var,
                    op: cmp_op,
                    threshold,
                    then_body,
                    else_body: Some(vec![self.lower_if_stmt(right, then_branch, else_branch)]),
                };
            }

            // Take this path when let (Expr::IdentExpr { name: l, .. }, Expr::IdentExpr { name: r, .. }).
            if let (Expr::IdentExpr { name: l, .. }, Expr::IdentExpr { name: r, .. }) =
                (left.as_ref(), right.as_ref())
            {
                return SirStmt::IfVar {
                    condition: l.clone(),
                    then_body: then_body.clone(),
                    else_body: Some(vec![SirStmt::IfVar {
                        condition: r.clone(),
                        then_body,
                        else_body,
                    }]),
                };
            }
        }
        SirStmt::IfRuntime {
            condition: serialize_expr_condition(condition),
            then_body,
            else_body,
        }
    }

    fn lower_expr_stmt(&self, expr: &Expr) -> SirStmt {
        // Lower expr stmt.
        //
        // Parameters:
        // - `self` — method receiver
        // - `expr` — input value
        //
        // Returns:
        // SirStmt.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.lower_expr_stmt(expr);

        // Match on expr and handle each case.
        match expr {
            Expr::CallExpr {
                callee, named_args, ..
            } => lower_actuator_call(callee, named_args),
            Expr::MatchExpr {
                scrutinee, arms, ..
            } => {
                // Take this path when let Expr::IdentExpr { name, .. } = scrutinee.as ref().
                if let Expr::IdentExpr { name, .. } = scrutinee.as_ref() {
                    let enum_name = arms.first().and_then(|arm| {
                        self.variant_index
                            .get(&arm.variant)
                            .map(|(enum_name, _)| enum_name.clone())
                    });

                    // Emit output when enum name provides a enum name.
                    if let Some(enum_name) = enum_name {
                        let sir_arms: Vec<SirMatchArm> = arms
                            .iter()
                            .filter_map(|arm| {
                                let (owner, tag) = self.variant_index.get(&arm.variant)?;

                                // Take the branch when owner differs from enum name.
                                if owner != &enum_name {
                                    return None;
                                }
                                Some(SirMatchArm {
                                    variant: arm.variant.clone(),
                                    tag: *tag,
                                    bindings: arm.bindings.clone(),
                                    body: self.lower_stmts(&arm.body),
                                })
                            })
                            .collect();

                        // Take the branch when len equals len.
                        if sir_arms.len() == arms.len() {
                            return SirStmt::MatchEnumUnit {
                                scrutinee: name.clone(),
                                enum_name,
                                arms: sir_arms,
                            };
                        }
                    }
                }
                SirStmt::Unsupported {
                    label: "match".into(),
                }
            }
            _ => SirStmt::Unsupported {
                label: "expr_stmt".into(),
            },
        }
    }

    fn resolve_enum_unit(&self, expr: &Expr) -> Option<(String, String, u32)> {
        // Resolve enum unit.
        //
        // Parameters:
        // - `self` — method receiver
        // - `expr` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.resolve_enum_unit(expr);

        // Match on expr and handle each case.
        match expr {
            Expr::IdentExpr { name, .. } => self
                .variant_index
                .get(name)
                .map(|(enum_name, tag)| (enum_name.clone(), name.clone(), *tag)),
            Expr::MemberExpr {
                object, property, ..
            } => {
                // Take this path when let Expr::IdentExpr.
                if let Expr::IdentExpr {
                    name: enum_name, ..
                } = object.as_ref()
                {
                    self.variant_index.get(property).and_then(|(owner, tag)| {
                        // Take the branch when owner equals enum name.
                        if owner == enum_name {
                            Some((enum_name.clone(), property.clone(), *tag))
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn resolve_enum_payload(&self, expr: &Expr) -> Option<(String, String, u32, Vec<f64>)> {
        // Resolve enum payload.
        //
        // Parameters:
        // - `self` — method receiver
        // - `expr` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.resolve_enum_payload(expr);

        // Compute Expr for the following logic.
        let Expr::CallExpr { callee, args, .. } = expr else {
            return None;
        };
        let Expr::IdentExpr { name, .. } = callee.as_ref() else {
            return None;
        };
        let (enum_name, tag) = self.variant_index.get(name)?;

        // Skip further work when args is empty.
        if args.is_empty() {
            return None;
        }
        let payloads: Vec<f64> = args.iter().filter_map(float_literal).collect();

        // Take the branch when len differs from len.
        if payloads.len() != args.len() {
            return None;
        }
        Some((enum_name.clone(), name.clone(), *tag, payloads))
    }
}

fn compare_bool_literal(expr: &Expr) -> Option<(String, bool)> {
    // Compare bool literal.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::compare_bool_literal(expr);

    // Compute Expr for the following logic.
    let Expr::BinaryExpr {
        op: crate::ast::BinaryOp::Eq,
        left,
        right,
        ..
    } = expr
    // Handle any remaining cases.
    else {
        return None;
    };

    // Take this path when let Expr::IdentExpr { name, .. } = left.as ref().
    if let Expr::IdentExpr { name, .. } = left.as_ref() {
        // Emit output when bool literal provides a value.
        if let Some(value) = bool_literal(right) {
            return Some((name.clone(), value));
        }
    }

    // Take this path when let Expr::IdentExpr { name, .. } = right.as ref().
    if let Expr::IdentExpr { name, .. } = right.as_ref() {
        // Emit output when bool literal provides a value.
        if let Some(value) = bool_literal(left) {
            return Some((name.clone(), value));
        }
    }
    None
}

fn compare_bool_ne_literal(expr: &Expr) -> Option<(String, bool)> {
    // Compare bool ne literal.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::compare_bool_ne_literal(expr);

    // Compute Expr for the following logic.
    let Expr::BinaryExpr {
        op: crate::ast::BinaryOp::Neq,
        left,
        right,
        ..
    } = expr
    // Handle any remaining cases.
    else {
        return None;
    };

    // Take this path when let Expr::IdentExpr { name, .. } = left.as ref().
    if let Expr::IdentExpr { name, .. } = left.as_ref() {
        // Emit output when bool literal provides a value.
        if let Some(value) = bool_literal(right) {
            return Some((name.clone(), !value));
        }
    }

    // Take this path when let Expr::IdentExpr { name, .. } = right.as ref().
    if let Expr::IdentExpr { name, .. } = right.as_ref() {
        // Emit output when bool literal provides a value.
        if let Some(value) = bool_literal(left) {
            return Some((name.clone(), !value));
        }
    }
    None
}

fn lower_return(value: Option<&Expr>) -> SirStmt {
    // Lower return.
    //
    // Parameters:
    // - `value` — input value
    //
    // Returns:
    // SirStmt.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::lower_return(value);

    // Match on value and handle each case.
    match value {
        None => SirStmt::ReturnVoid,
        Some(expr) => {
            // Emit output when int literal provides a value.
            if let Some(value) = int_literal(expr) {
                SirStmt::ReturnInt { value }
            } else if let Some(value) = float_literal(expr) {
                SirStmt::ReturnFloat { value }
            } else {
                SirStmt::Unsupported {
                    label: "return".into(),
                }
            }
        }
    }
}

fn lower_actuator_call(callee: &Expr, named_args: &[NamedArg]) -> SirStmt {
    // Lower actuator call.
    //
    // Parameters:
    // - `callee` — input value
    // - `named_args` — input value
    //
    // Returns:
    // SirStmt.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::lower_actuator_call(callee, named_args);

    // Compute Expr for the following logic.
    let Expr::MemberExpr {
        object, property, ..
    } = callee
    // Handle any remaining cases.
    else {
        return SirStmt::Unsupported {
            label: "call".into(),
        };
    };
    let Expr::IdentExpr { name: actuator, .. } = object.as_ref() else {
        return SirStmt::Unsupported {
            label: "actuator_call".into(),
        };
    };

    // Match on as str and handle each case.
    match property.as_str() {
        "stop" => SirStmt::ActuatorStop {
            actuator: actuator.clone(),
        },
        "drive" => SirStmt::ActuatorDrive {
            actuator: actuator.clone(),
            linear: named_arg_f64(named_args, "linear").unwrap_or(0.0),
            angular: named_arg_f64(named_args, "angular").unwrap_or(0.0),
        },
        _ => SirStmt::Unsupported {
            label: format!("actuator.{property}"),
        },
    }
}

fn named_arg_f64(args: &[NamedArg], name: &str) -> Option<f64> {
    // Named arg f64.
    //
    // Parameters:
    // - `args` — input value
    // - `name` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::named_arg_f64(args, name);

    // Iterate over args.
    args.iter()
        .find(|arg| arg.name == name)
        .and_then(|arg| numeric_value(&arg.value))
}

fn numeric_value(expr: &Expr) -> Option<f64> {
    // Numeric value.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::numeric_value(expr);

    // Match on expr and handle each case.
    match expr {
        Expr::LiteralExpr {
            value: LiteralValue::Number(n),
            ..
        } => Some(*n),
        Expr::UnitLiteralExpr { value, unit, .. } => Some(unit_scalar(*value, unit)),
        _ => None,
    }
}

fn unit_scalar(value: f64, _unit: &UnitKind) -> f64 {
    // Unit scalar.
    //
    // Parameters:
    // - `value` — input value
    // - `_unit` — input value
    //
    // Returns:
    // Numeric result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::unit_scalar(value, _unit);

    // Produce value as the result.
    value
}

fn int_literal(expr: &Expr) -> Option<i64> {
    // Int literal.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::int_literal(expr);

    // Match on expr and handle each case.
    match expr {
        Expr::LiteralExpr {
            value: LiteralValue::Number(n),
            ..
        } => Some(*n as i64),
        _ => None,
    }
}

fn float_literal(expr: &Expr) -> Option<f64> {
    // Float literal.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::float_literal(expr);

    // Produce numeric value as the result.
    numeric_value(expr)
}

fn bool_literal(expr: &Expr) -> Option<bool> {
    // Bool literal.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::bool_literal(expr);

    // Match on expr and handle each case.
    match expr {
        Expr::LiteralExpr {
            value: LiteralValue::Bool(b),
            ..
        } => Some(*b),
        _ => None,
    }
}

fn string_literal(expr: &Expr) -> Option<String> {
    // String literal.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::string_literal(expr);

    // Match on expr and handle each case.
    match expr {
        Expr::LiteralExpr {
            value: LiteralValue::String(s),
            ..
        } => Some(s.clone()),
        Expr::LiteralExpr {
            value: LiteralValue::Number(n),
            ..
        } => Some((*n as i64).to_string()),
        _ => None,
    }
}

fn stmt_kind(stmt: &Stmt) -> String {
    // Stmt kind.
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
    // let result = spanda_core::sir::stmt_kind(stmt);

    // Match on stmt and handle each case.
    match stmt {
        Stmt::IfStmt { .. } => "if".into(),
        Stmt::LoopStmt { .. } => "loop".into(),
        Stmt::PublishStmt { .. } => "publish".into(),
        Stmt::ServiceCallStmt { .. } => "service_call".into(),
        Stmt::ActionSendStmt { .. } => "action_send".into(),
        Stmt::ResetEmergencyStopStmt { .. } => "reset_emergency_stop".into(),
        Stmt::EmitStmt { .. } => "emit".into(),
        Stmt::EnterStmt { .. } => "enter".into(),
        Stmt::RememberStmt { .. } => "remember".into(),
        Stmt::SubscribeStmt { .. } => "subscribe".into(),
        Stmt::ExecuteStmt { .. } => "execute".into(),
        Stmt::DiscoverStmt { .. } => "discover".into(),
        Stmt::ReceiveStmt { .. } => "receive".into(),
        Stmt::SpawnStmt { .. } => "spawn".into(),
        Stmt::SelectStmt { .. } => "select".into(),
        _ => "stmt".into(),
    }
}

fn binary_op_to_compare(op: crate::ast::BinaryOp) -> Option<SirCompareOp> {
    // Binary op to compare.
    //
    // Parameters:
    // - `op` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::binary_op_to_compare(op);

    // Import the items needed by the logic below.
    use crate::ast::BinaryOp;

    // Match on op and handle each case.
    match op {
        BinaryOp::Lt => Some(SirCompareOp::Lt),
        BinaryOp::Lte => Some(SirCompareOp::Lte),
        BinaryOp::Gt => Some(SirCompareOp::Gt),
        BinaryOp::Gte => Some(SirCompareOp::Gte),
        BinaryOp::Eq => Some(SirCompareOp::Eq),
        BinaryOp::Neq => Some(SirCompareOp::Neq),
        _ => None,
    }
}

fn reverse_compare(op: SirCompareOp) -> SirCompareOp {
    // Reverse compare.
    //
    // Parameters:
    // - `op` — input value
    //
    // Returns:
    // SirCompareOp.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::reverse_compare(op);

    // Match on op and handle each case.
    match op {
        SirCompareOp::Lt => SirCompareOp::Gt,
        SirCompareOp::Lte => SirCompareOp::Gte,
        SirCompareOp::Gt => SirCompareOp::Lt,
        SirCompareOp::Gte => SirCompareOp::Lte,
        other => other,
    }
}

fn eval_double_compare(op: SirCompareOp, left: f64, right: f64) -> bool {
    // Eval double compare.
    //
    // Parameters:
    // - `op` — input value
    // - `left` — input value
    // - `right` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::eval_double_compare(op, left, right);

    // Match on op and handle each case.
    match op {
        SirCompareOp::Lt => left < right,
        SirCompareOp::Lte => left <= right,
        SirCompareOp::Gt => left > right,
        SirCompareOp::Gte => left >= right,
        SirCompareOp::Eq => (left - right).abs() < f64::EPSILON,
        SirCompareOp::Neq => (left - right).abs() >= f64::EPSILON,
    }
}

fn extract_double_compare(expr: &Expr) -> Option<(String, SirCompareOp, f64)> {
    // Extract double compare.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::extract_double_compare(expr);

    // Compute Expr for the following logic.
    let Expr::BinaryExpr {
        op, left, right, ..
    } = expr
    // Handle any remaining cases.
    else {
        return None;
    };
    let sir_op = binary_op_to_compare(*op)?;

    // Take this path when let Expr::IdentExpr { name, .. } = left.as ref().
    if let Expr::IdentExpr { name, .. } = left.as_ref() {
        // Emit output when float literal provides a value.
        if let Some(value) = float_literal(right) {
            return Some((name.clone(), sir_op, value));
        }
    }

    // Take this path when let Expr::IdentExpr { name, .. } = right.as ref().
    if let Expr::IdentExpr { name, .. } = right.as_ref() {
        // Emit output when float literal provides a value.
        if let Some(value) = float_literal(left) {
            return Some((name.clone(), reverse_compare(sir_op), value));
        }
    }
    None
}

fn extract_scan_distance_compare(expr: &Expr) -> Option<(String, SirCompareOp, f64)> {
    // Extract scan distance compare.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::extract_scan_distance_compare(expr);

    // Compute Expr for the following logic.
    let Expr::BinaryExpr {
        op, left, right, ..
    } = expr
    // Handle any remaining cases.
    else {
        return None;
    };
    let sir_op = binary_op_to_compare(*op)?;
    let threshold = float_literal(right)?;
    let Expr::MemberExpr {
        object, property, ..
    } = left.as_ref()
    // Handle any remaining cases.
    else {
        return None;
    };

    // Take the branch when property differs from "nearest distance".
    if property != "nearest_distance" {
        return None;
    }
    let Expr::IdentExpr { name, .. } = object.as_ref() else {
        return None;
    };
    Some((name.clone(), sir_op, threshold))
}

fn eval_const_bool(expr: &Expr) -> Option<bool> {
    // Eval const bool.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::eval_const_bool(expr);

    // use value when bool literal is present.

    // Emit output when bool literal provides a value.
    if let Some(value) = bool_literal(expr) {
        return Some(value);
    }

    // Take this path when let Expr::UnaryExpr.
    if let Expr::UnaryExpr {
        op: crate::ast::UnaryOp::Not,
        operand,
        ..
    } = expr
    {
        return eval_const_bool(operand).map(|value| !value);
    }

    // Take this path when let Expr::BinaryExpr.
    if let Expr::BinaryExpr {
        op: crate::ast::BinaryOp::And,
        left,
        right,
        ..
    } = expr
    {
        return Some(eval_const_bool(left)? && eval_const_bool(right)?);
    }

    // Take this path when let Expr::BinaryExpr.
    if let Expr::BinaryExpr {
        op: crate::ast::BinaryOp::Or,
        left,
        right,
        ..
    } = expr
    {
        return Some(eval_const_bool(left)? || eval_const_bool(right)?);
    }

    // Take this path when let Expr::BinaryExpr.
    if let Expr::BinaryExpr {
        op, left, right, ..
    } = expr
    {
        // Take this path when let (Some(l), Some(r)) = (float literal(left), float literal(right)).
        if let (Some(l), Some(r)) = (float_literal(left), float_literal(right)) {
            // Emit output when binary op to compare provides a cmp.
            if let Some(cmp) = binary_op_to_compare(*op) {
                return Some(eval_double_compare(cmp, l, r));
            }
        }
    }
    None
}

pub fn serialize_expr_condition(expr: &Expr) -> String {
    // Serialize expr condition.
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
    // let result = spanda_core::sir::serialize_expr_condition(expr);

    // Produce unwrap or else as the result.
    serde_json::to_string(&expr_to_condition(expr)).unwrap_or_else(|_| {
        serde_json::to_string(&SirCondition::Unsupported).unwrap_or_else(|_| "{}".into())
    })
}

fn expr_to_condition(expr: &Expr) -> SirCondition {
    // Expr to condition.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // SirCondition.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::expr_to_condition(expr);

    // use value when bool literal is present.

    // Emit output when bool literal provides a value.
    if let Some(value) = bool_literal(expr) {
        return SirCondition::Bool { value };
    }

    // Take this path when let Expr::IdentExpr { name, .. } = expr.
    if let Expr::IdentExpr { name, .. } = expr {
        return SirCondition::Ident { name: name.clone() };
    }

    // Take this path when let Expr::UnaryExpr.
    if let Expr::UnaryExpr {
        op: crate::ast::UnaryOp::Not,
        operand,
        ..
    } = expr
    {
        return SirCondition::Not {
            operand: Box::new(expr_to_condition(operand)),
        };
    }

    // Take this path when let Expr::BinaryExpr.
    if let Expr::BinaryExpr {
        op: crate::ast::BinaryOp::And,
        left,
        right,
        ..
    } = expr
    {
        return SirCondition::And {
            left: Box::new(expr_to_condition(left)),
            right: Box::new(expr_to_condition(right)),
        };
    }

    // Take this path when let Expr::BinaryExpr.
    if let Expr::BinaryExpr {
        op: crate::ast::BinaryOp::Or,
        left,
        right,
        ..
    } = expr
    {
        return SirCondition::Or {
            left: Box::new(expr_to_condition(left)),
            right: Box::new(expr_to_condition(right)),
        };
    }

    // Take this path when let Some((variable, equals)) = compare bool literal(expr).
    if let Some((variable, equals)) = compare_bool_literal(expr) {
        return SirCondition::EqBool {
            name: variable,
            value: equals,
        };
    }

    // Take this path when let Some((variable, equals)) = compare bool ne literal(expr).
    if let Some((variable, equals)) = compare_bool_ne_literal(expr) {
        return SirCondition::NeqBool {
            name: variable,
            value: equals,
        };
    }

    // Take this path when let Some((name, value)) = compare string literal(expr).
    if let Some((name, value)) = compare_string_literal(expr) {
        return SirCondition::EqString { name, value };
    }

    // Take this path when let Some((name, cmp, value)) = extract double compare(expr).
    if let Some((name, cmp, value)) = extract_double_compare(expr) {
        return SirCondition::CompareDouble { name, cmp, value };
    }

    // Take this path when let Some((scan var, cmp, threshold)) = extract scan distance compare(e.
    if let Some((scan_var, cmp, threshold)) = extract_scan_distance_compare(expr) {
        return SirCondition::ScanDistance {
            scan_var,
            cmp,
            threshold,
        };
    }
    SirCondition::Unsupported
}

fn compare_string_literal(expr: &Expr) -> Option<(String, String)> {
    // Compare string literal.
    //
    // Parameters:
    // - `expr` — input value
    //
    // Returns:
    // Some value on success, otherwise none.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::compare_string_literal(expr);

    // Compute Expr for the following logic.
    let Expr::BinaryExpr {
        op: crate::ast::BinaryOp::Eq,
        left,
        right,
        ..
    } = expr
    // Handle any remaining cases.
    else {
        return None;
    };

    // Take this path when let Expr::IdentExpr { name, .. } = left.as ref().
    if let Expr::IdentExpr { name, .. } = left.as_ref() {
        // Emit output when string literal provides a value.
        if let Some(value) = string_literal(right) {
            return Some((name.clone(), value));
        }
    }

    // Take this path when let Expr::IdentExpr { name, .. } = right.as ref().
    if let Expr::IdentExpr { name, .. } = right.as_ref() {
        // Emit output when string literal provides a value.
        if let Some(value) = string_literal(left) {
            return Some((name.clone(), value));
        }
    }
    None
}

fn lower_extern(ext: &ExternFnDecl) -> SirExtern {
    // Lower extern.
    //
    // Parameters:
    // - `ext` — input value
    //
    // Returns:
    // SirExtern.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::lower_extern(ext);

    // Produce SirExtern as the result.
    SirExtern {
        name: ext.name.clone(),
        library: ext.library.clone(),
        bridge: ext.bridge,
        params: ext
            .params
            .iter()
            .map(|p| SirParam {
                name: p.name.clone(),
                type_name: type_to_string(&p.type_ann),
            })
            .collect(),
        return_type: type_to_string(&ext.return_type),
    }
}

fn type_to_string(ty: &SpandaType) -> String {
    // Type to string.
    //
    // Parameters:
    // - `ty` — input value
    //
    // Returns:
    // Text result.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::sir::type_to_string(ty);

    // Match on ty and handle each case.
    match ty {
        SpandaType::Void => "void".into(),
        SpandaType::Int => "Int".into(),
        SpandaType::Float => "Float".into(),
        SpandaType::Bool => "Bool".into(),
        SpandaType::String => "String".into(),
        SpandaType::Char => "Char".into(),
        SpandaType::Bytes => "Bytes".into(),
        SpandaType::Null => "Null".into(),
        SpandaType::Number { unit } => format!("number({unit:?})"),
        SpandaType::Named { name } => name.clone(),
        SpandaType::Generic { name, type_args } => {
            let args = type_args
                .iter()
                .map(type_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}<{args}>")
        }
        SpandaType::Scan => "Scan".into(),
        SpandaType::Pose => "Pose".into(),
        SpandaType::Velocity => "Velocity".into(),
        SpandaType::Trajectory => "Trajectory".into(),
        SpandaType::Transform => "Transform".into(),
        SpandaType::EnumVariant { enum_name, variant } => format!("{enum_name}::{variant}"),
        SpandaType::TypeParam { name } => name.clone(),
        SpandaType::TraitObject { trait_name } => format!("dyn {trait_name}"),
        SpandaType::Regex => "Regex".into(),
        SpandaType::Match => "Match".into(),
        SpandaType::Capture => "Capture".into(),
        SpandaType::CaptureGroup => "CaptureGroup".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer, parser, types};

    #[test]
    fn lowers_module_and_externs() {
        // Lowers module and externs.
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
        // let result = spanda_core::sir::lowers_module_and_externs();

        let source = r#"
module demo;

extern "libc" fn stub_add(a: Int, b: Int) -> Int;
extern python fn py_echo(x: Int) -> Int;

export fn main() -> Int { return 1; }

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.stop(); }
}
"#;
        let tokens = lexer::tokenize(source).expect("tokenize");
        let program = parser::parse(tokens).expect("parse");
        types::check(&program).expect("check");
        let sir = lower_program(&program);
        assert_eq!(sir.module_name.as_deref(), Some("demo"));
        assert_eq!(sir.functions.len(), 1);
        assert_eq!(sir.functions[0].name, "main");
        assert_eq!(sir.externs.len(), 2);
        assert_eq!(sir.externs[0].bridge, BridgeKind::Native);
        assert_eq!(sir.externs[1].bridge, BridgeKind::Python);
        assert_eq!(sir.robot_names, vec!["R"]);
        assert_eq!(sir.behavior_names, vec!["run"]);
        assert_eq!(sir.robots.len(), 1);
        assert_eq!(sir.robots[0].name, "R");
        assert_eq!(sir.robots[0].behaviors[0].name, "run");
        assert!(sir.robots[0].behaviors[0].stmt_count >= 1);
        assert!(matches!(
            sir.robots[0].behaviors[0].body[0],
            SirStmt::ActuatorStop { .. }
        ));
        assert!(matches!(
            sir.functions[0].body[0],
            SirStmt::ReturnInt { value: 1 }
        ));
    }

    #[test]
    fn lowers_drive_and_return_stmts() {
        // Lowers drive and return stmts.
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
        // let result = spanda_core::sir::lowers_drive_and_return_stmts();

        let source = r#"
module demo;
export fn add() -> Int { return 42; }
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() { wheels.drive(linear: 0.5 m/s, angular: 0.1 rad/s); }
}
"#;
        let tokens = lexer::tokenize(source).expect("tokenize");
        let program = parser::parse(tokens).expect("parse");
        types::check(&program).expect("check");
        let sir = lower_program(&program);
        assert!(matches!(
            sir.robots[0].behaviors[0].body[0],
            SirStmt::ActuatorDrive {
                linear,
                angular,
                ..
            } if (linear - 0.5).abs() < f64::EPSILON && (angular - 0.1).abs() < f64::EPSILON
        ));
        assert!(matches!(
            sir.functions[0].body[0],
            SirStmt::ReturnInt { value: 42 }
        ));
    }

    #[test]
    fn lowers_loop_and_publish_stmts() {
        // Lowers loop and publish stmts.
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
        // let result = spanda_core::sir::lowers_loop_and_publish_stmts();

        let source = r#"
robot R {
  topic status: String publish on "/status";
  actuator wheels: DifferentialDrive;
  behavior run() {
    publish status with "ok";
    loop every 100ms {
      wheels.stop();
    }
  }
}
"#;
        let program = parser::parse(lexer::tokenize(source).expect("tokenize")).expect("parse");
        types::check(&program).expect("check");
        let sir = lower_program(&program);
        let body = &sir.robots[0].behaviors[0].body;
        assert!(matches!(body[0], SirStmt::Publish { ref topic, .. } if topic == "status"));
        assert!(
            matches!(body[1], SirStmt::LoopEvery { interval_ms, .. } if (interval_ms - 100.0).abs() < f64::EPSILON)
        );
        assert!(
            matches!(body[1], SirStmt::LoopEvery { ref body, .. } if matches!(body[0], SirStmt::ActuatorStop { .. }))
        );
    }

    #[test]
    fn lowers_if_bool_and_subscribe_stmts() {
        // Lowers if bool and subscribe stmts.
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
        // let result = spanda_core::sir::lowers_if_bool_and_subscribe_stmts();

        let source = r#"
robot R {
  topic cmd: String subscribe on "/cmd";
  actuator wheels: DifferentialDrive;
  behavior run() {
    subscribe cmd;
    if true { wheels.stop(); } else { wheels.drive(linear: 0.1 m/s, angular: 0.0 rad/s); }
  }
}
"#;
        let program = parser::parse(lexer::tokenize(source).expect("tokenize")).expect("parse");
        types::check(&program).expect("check");
        let sir = lower_program(&program);
        let body = &sir.robots[0].behaviors[0].body;
        assert!(matches!(body[0], SirStmt::Subscribe { ref target } if target == "cmd"));
        assert!(matches!(
            body[1],
            SirStmt::IfBool {
                condition: true,
                ref then_body,
                ref else_body,
                ..
            } if then_body.len() == 1 && else_body.as_ref().is_some_and(|b| b.len() == 1)
        ));
    }

    #[test]
    fn lowers_if_var_and_match_enum_unit() {
        // Lowers if var and match enum unit.
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
        // let result = spanda_core::sir::lowers_if_var_and_match_enum_unit();

        let source = r#"
enum RobotState { Idle, Navigating }

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let enabled = true;
    let state = Idle;
    if enabled { wheels.stop(); }
    match state {
      Idle => wheels.stop();
      Navigating => wheels.drive(linear: 0.2 m/s, angular: 0.0 rad/s);
    };
  }
}
"#;
        let program = parser::parse(lexer::tokenize(source).expect("tokenize")).expect("parse");
        types::check(&program).expect("check");
        let sir = lower_program(&program);
        let body = &sir.robots[0].behaviors[0].body;
        assert!(matches!(body[0], SirStmt::LetBool { .. }));
        assert!(matches!(body[1], SirStmt::LetEnumUnit { .. }));
        assert!(matches!(body[2], SirStmt::IfVar { .. }));
        assert!(matches!(body[3], SirStmt::MatchEnumUnit { .. }));
    }

    #[test]
    fn lowers_enum_payload_if_compare_and_if_not() {
        // Lowers enum payload if compare and if not.
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
        // let result = spanda_core::sir::lowers_enum_payload_if_compare_and_if_not();

        let source = r#"
enum DriveCmd { Stop, Drive(Float, Float) }

robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let cmd = Drive(0.3, 0.0);
    let flag = true;
    if flag == true { wheels.drive(linear: 0.1 m/s, angular: 0.0 rad/s); }
    if not flag { wheels.stop(); }
    match cmd {
      Stop => wheels.stop();
      Drive(l, a) => wheels.drive(linear: 0.3 m/s, angular: 0.0 rad/s);
    };
  }
}
"#;
        let program = parser::parse(lexer::tokenize(source).expect("tokenize")).expect("parse");
        types::check(&program).expect("check");
        let sir = lower_program(&program);
        let body = &sir.robots[0].behaviors[0].body;
        assert!(
            matches!(body[0], SirStmt::LetEnumPayload { ref payloads, .. } if payloads.len() == 2)
        );
        assert!(matches!(
            body[2],
            SirStmt::IfCompareBool { equals: true, .. }
        ));
        assert!(matches!(body[3], SirStmt::IfNotVar { .. }));
        assert!(
            matches!(body[4], SirStmt::MatchEnumUnit { ref arms, .. } if arms.iter().any(|arm| !arm.bindings.is_empty()))
        );
    }

    #[test]
    fn lowers_if_double_and_scan_distance_compare() {
        // Lowers if double and scan distance compare.
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
        // let result = spanda_core::sir::lowers_if_double_and_scan_distance_compare();

        let source = r#"
robot R {
  sensor scan: Lidar;
  actuator wheels: DifferentialDrive;
  behavior run() {
    let limit = 1.5;
    if limit < 2.0 { wheels.stop(); }
    if scan.nearest_distance < 1.0 m { wheels.drive(linear: 0.1 m/s, angular: 0.0 rad/s); }
  }
}
"#;
        let program = parser::parse(lexer::tokenize(source).expect("tokenize")).expect("parse");
        types::check(&program).expect("check");
        let sir = lower_program(&program);
        let body = &sir.robots[0].behaviors[0].body;
        assert!(
            matches!(body[0], SirStmt::LetDouble { value, .. } if (value - 1.5).abs() < f64::EPSILON)
        );
        assert!(matches!(
            body[1],
            SirStmt::IfCompareDouble {
                op: SirCompareOp::Lt,
                right,
                ..
            } if (right - 2.0).abs() < f64::EPSILON
        ));
        assert!(matches!(
            body[2],
            SirStmt::IfScanDistanceCompare {
                ref scan_var,
                op: SirCompareOp::Lt,
                threshold,
                ..
            } if scan_var == "scan" && (threshold - 1.0).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn lowers_unsupported_if_to_runtime_condition() {
        // Lowers unsupported if to runtime condition.
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
        // let result = spanda_core::sir::lowers_unsupported_if_to_runtime_condition();

        let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let mode = "auto";
    if mode == "auto" { wheels.stop(); }
  }
}
"#;
        let program = parser::parse(lexer::tokenize(source).expect("tokenize")).expect("parse");
        types::check(&program).expect("check");
        let sir = lower_program(&program);
        let body = &sir.robots[0].behaviors[0].body;
        assert!(
            matches!(body[1], SirStmt::IfRuntime { ref condition, .. } if condition.contains("eq_string"))
        );
    }

    #[test]
    fn lowers_string_let_for_runtime_if() {
        // Lowers string let for runtime if.
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
        // let result = spanda_core::sir::lowers_string_let_for_runtime_if();

        let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let mode = "auto";
    if mode == "auto" { wheels.stop(); }
  }
}
"#;
        let program = parser::parse(lexer::tokenize(source).expect("tokenize")).expect("parse");
        types::check(&program).expect("check");
        let sir = lower_program(&program);
        let body = &sir.robots[0].behaviors[0].body;
        assert!(matches!(
            body[0],
            SirStmt::LetString {
                ref name,
                ref value,
                ..
            } if name == "mode" && value == "auto"
        ));
    }
}
