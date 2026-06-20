//! Spanda IR (SIR) — typed intermediate representation between AST and backends.
//!
//! Lowers module functions, extern declarations, robot metadata, and a subset of
//! statements for LLVM codegen. Execution still uses the tree-walking interpreter.

use crate::ast::{
    BehaviorDecl, Expr, LiteralValue, NamedArg, Program, RobotDecl, SpandaType, Stmt, UnitKind,
};
use crate::foundations::{BridgeKind, ExternFnDecl, ModuleFnDecl, Visibility};
use serde::{Deserialize, Serialize};

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
    Unsupported {
        #[serde(rename = "stmt_kind")]
        label: String,
    },
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
    for robot in robots {
        let RobotDecl::RobotDecl {
            name,
            behaviors,
            tasks,
            ..
        } = robot;
        let mut sir_behaviors = Vec::new();
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
                body: lower_stmts(body),
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
        functions: functions.iter().map(lower_function).collect(),
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

fn lower_function(func: &ModuleFnDecl) -> SirFunction {
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
        body: lower_stmts(body),
    }
}

fn lower_stmts(stmts: &[Stmt]) -> Vec<SirStmt> {
    stmts.iter().map(lower_stmt).collect()
}

fn lower_stmt(stmt: &Stmt) -> SirStmt {
    match stmt {
        Stmt::VarDecl { name, init, .. } => {
            if let Some(init) = init {
                if let Some(value) = int_literal(init) {
                    return SirStmt::LetInt {
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
        Stmt::ExprStmt { expr, .. } => lower_expr_stmt(expr),
        Stmt::EmergencyStopStmt { .. } => SirStmt::EmergencyStop,
        other => SirStmt::Unsupported {
            label: stmt_kind(other),
        },
    }
}

fn lower_return(value: Option<&Expr>) -> SirStmt {
    match value {
        None => SirStmt::ReturnVoid,
        Some(expr) => {
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

fn lower_expr_stmt(expr: &Expr) -> SirStmt {
    match expr {
        Expr::CallExpr {
            callee,
            named_args,
            ..
        } => lower_actuator_call(callee, named_args),
        _ => SirStmt::Unsupported {
            label: "expr_stmt".into(),
        },
    }
}

fn lower_actuator_call(callee: &Expr, named_args: &[NamedArg]) -> SirStmt {
    let Expr::MemberExpr { object, property, .. } = callee else {
        return SirStmt::Unsupported {
            label: "call".into(),
        };
    };
    let Expr::IdentExpr { name: actuator, .. } = object.as_ref() else {
        return SirStmt::Unsupported {
            label: "actuator_call".into(),
        };
    };
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
    args.iter()
        .find(|arg| arg.name == name)
        .and_then(|arg| numeric_value(&arg.value))
}

fn numeric_value(expr: &Expr) -> Option<f64> {
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
    value
}

fn int_literal(expr: &Expr) -> Option<i64> {
    match expr {
        Expr::LiteralExpr {
            value: LiteralValue::Number(n),
            ..
        } => Some(*n as i64),
        _ => None,
    }
}

fn float_literal(expr: &Expr) -> Option<f64> {
    numeric_value(expr)
}

fn stmt_kind(stmt: &Stmt) -> String {
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

fn lower_extern(ext: &ExternFnDecl) -> SirExtern {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer, parser, types};

    #[test]
    fn lowers_module_and_externs() {
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
        assert!(matches!(sir.functions[0].body[0], SirStmt::ReturnInt { value: 42 }));
    }
}
