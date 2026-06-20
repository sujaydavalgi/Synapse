//! Spanda IR (SIR) — typed intermediate representation between AST and backends.
//!
//! Milestone 1: lower module functions, extern declarations, and robot names.
//! Execution still uses the tree-walking interpreter; SIR is for codegen planning.

use crate::ast::{BehaviorDecl, Program, RobotDecl, SpandaType};
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SirFunction {
    pub name: String,
    pub visibility: SirVisibility,
    pub type_params: Vec<String>,
    pub params: Vec<SirParam>,
    pub return_type: String,
    pub is_async: bool,
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
    for robot in robots {
        if let RobotDecl::RobotDecl { behaviors, .. } = robot {
            for behavior in behaviors {
                if let BehaviorDecl::BehaviorDecl { name, .. } = behavior {
                    behavior_names.push(name.clone());
                }
            }
        }
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
    }
}
