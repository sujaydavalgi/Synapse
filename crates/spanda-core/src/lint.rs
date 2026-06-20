use crate::ast::*;
use crate::error::SpandaError;
use crate::foundations::TaskDecl;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LintSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LintIssue {
    pub rule: String,
    pub message: String,
    pub line: u32,
    pub column: u32,
    pub severity: LintSeverity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LintReport {
    pub issues: Vec<LintIssue>,
}

impl LintReport {
    pub fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|i| i.severity == LintSeverity::Error)
    }
}

pub fn lint(source: &str) -> Result<LintReport, SpandaError> {
    let tokens = crate::lexer::tokenize(source)?;
    let program = crate::parser::parse(tokens)?;
    Ok(lint_program(source, &program))
}

fn lint_concurrency(program: &Program, issues: &mut Vec<LintIssue>) {
    let Program::Program { robots, .. } = program;
    for robot in robots {
        let RobotDecl::RobotDecl {
            behaviors, tasks, ..
        } = robot;
        for behavior in behaviors {
            let BehaviorDecl::BehaviorDecl { body, .. } = behavior;
            lint_stmt_channel_flow(body, issues);
        }
        for task in tasks {
            let TaskDecl::TaskDecl { body, .. } = task;
            lint_stmt_channel_flow(body, issues);
        }
    }
}

fn lint_stmt_channel_flow(stmts: &[Stmt], issues: &mut Vec<LintIssue>) {
    let mut channels: std::collections::HashMap<String, (bool, bool, u32, u32)> =
        std::collections::HashMap::new();
    collect_channel_flow(stmts, &mut channels);
    for (name, (sent, recv, line, column)) in channels {
        if recv && !sent {
            issues.push(LintIssue {
                rule: "channel-recv-without-send".into(),
                message: format!(
                    "Channel '{name}' may be received from without a matching send in this scope"
                ),
                line,
                column,
                severity: LintSeverity::Warning,
            });
        }
        if sent && !recv {
            issues.push(LintIssue {
                rule: "channel-send-without-recv".into(),
                message: format!(
                    "Channel '{name}' is sent to but never received from in this scope"
                ),
                line,
                column,
                severity: LintSeverity::Warning,
            });
        }
    }
}

#[allow(clippy::collapsible_match)]
fn collect_channel_flow(
    stmts: &[Stmt],
    channels: &mut std::collections::HashMap<String, (bool, bool, u32, u32)>,
) {
    for stmt in stmts {
        match stmt {
            Stmt::VarDecl {
                name, init, span, ..
            } =>
            {
                #[allow(clippy::collapsible_if)]
                if let Some(value) = init {
                    if matches!(value, Expr::CallExpr { callee, .. }
                        if matches!(callee.as_ref(), Expr::IdentExpr { name: n, .. } if n == "channel"))
                    {
                        channels.entry(name.clone()).or_insert((
                            false,
                            false,
                            span.start.line,
                            span.start.column,
                        ));
                    }
                }
            }
            Stmt::ExprStmt { expr, .. }
            | Stmt::ReturnStmt {
                value: Some(expr), ..
            } => {
                mark_channel_usage(expr, channels);
            }
            Stmt::IfStmt {
                then_branch,
                else_branch,
                ..
            } => {
                collect_channel_flow(then_branch, channels);
                if let Some(else_branch) = else_branch {
                    collect_channel_flow(else_branch, channels);
                }
            }
            Stmt::LoopStmt { body, .. } => collect_channel_flow(body, channels),
            Stmt::ParallelStmt { body, .. } => collect_channel_flow(body, channels),
            Stmt::SelectStmt { arms, .. } => {
                for arm in arms {
                    match &arm.channel {
                        Expr::IdentExpr { name, span } => {
                            let entry = channels.entry(name.clone()).or_insert((
                                false,
                                false,
                                span.start.line,
                                span.start.column,
                            ));
                            entry.1 = true;
                        }
                        Expr::CallExpr { callee, args, .. } => {
                            if let Expr::IdentExpr { name: fn_name, .. } = callee.as_ref() {
                                if fn_name == "recv" {
                                    if let Some(Expr::IdentExpr { name, span }) = args.first() {
                                        let entry = channels.entry(name.clone()).or_insert((
                                            false,
                                            false,
                                            span.start.line,
                                            span.start.column,
                                        ));
                                        entry.1 = true;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    collect_channel_flow(&arm.body, channels);
                }
            }
            _ => {}
        }
    }
}

fn mark_channel_usage(
    expr: &Expr,
    channels: &mut std::collections::HashMap<String, (bool, bool, u32, u32)>,
) {
    match expr {
        Expr::CallExpr { callee, args, .. } => {
            if let Expr::IdentExpr { name: fn_name, .. } = callee.as_ref() {
                if fn_name == "send" || fn_name == "recv" {
                    if let Some(Expr::IdentExpr { name, span }) = args.first() {
                        let entry = channels.entry(name.clone()).or_insert((
                            false,
                            false,
                            span.start.line,
                            span.start.column,
                        ));
                        if fn_name == "send" {
                            entry.0 = true;
                        } else {
                            entry.1 = true;
                        }
                    }
                }
            }
            for arg in args {
                mark_channel_usage(arg, channels);
            }
        }
        Expr::BinaryExpr { left, right, .. } => {
            mark_channel_usage(left, channels);
            mark_channel_usage(right, channels);
        }
        Expr::UnaryExpr { operand, .. } => mark_channel_usage(operand, channels),
        Expr::MemberExpr { object, .. } => mark_channel_usage(object, channels),
        Expr::SpawnExpr { callee, args, .. } => {
            mark_channel_usage(callee, channels);
            for arg in args {
                mark_channel_usage(arg, channels);
            }
        }
        _ => {}
    }
}

fn lint_program(source: &str, program: &Program) -> LintReport {
    let mut issues = Vec::new();
    lint_source_style(source, &mut issues);
    lint_program_structure(program, &mut issues);
    lint_imports(source, program, &mut issues);
    lint_concurrency(program, &mut issues);
    LintReport { issues }
}

fn lint_source_style(source: &str, issues: &mut Vec<LintIssue>) {
    for (idx, line) in source.lines().enumerate() {
        let line_no = idx as u32 + 1;
        if line.ends_with(' ') || line.ends_with('\t') {
            issues.push(LintIssue {
                rule: "trailing-whitespace".into(),
                message: "Line has trailing whitespace".into(),
                line: line_no,
                column: line.len() as u32,
                severity: LintSeverity::Warning,
            });
        }
        if line.len() > 120 {
            issues.push(LintIssue {
                rule: "line-length".into(),
                message: format!("Line exceeds 120 columns ({} chars)", line.len()),
                line: line_no,
                column: 121,
                severity: LintSeverity::Warning,
            });
        }
    }
}

fn lint_program_structure(program: &Program, issues: &mut Vec<LintIssue>) {
    let Program::Program {
        module_name,
        tests,
        robots,
        ..
    } = program;

    if module_name.is_none() {
        issues.push(LintIssue {
            rule: "missing-module".into(),
            message: "Program has no `module` declaration".into(),
            line: 1,
            column: 1,
            severity: LintSeverity::Warning,
        });
    }

    for test in tests {
        if test.body.is_empty() {
            issues.push(LintIssue {
                rule: "empty-test".into(),
                message: format!("Test \"{}\" has an empty body", test.name),
                line: test.span.start.line,
                column: test.span.start.column,
                severity: LintSeverity::Warning,
            });
        }
    }

    for robot in robots {
        let RobotDecl::RobotDecl { behaviors, .. } = robot;
        for behavior in behaviors {
            let BehaviorDecl::BehaviorDecl {
                name, body, span, ..
            } = behavior;
            if body.is_empty() {
                issues.push(LintIssue {
                    rule: "empty-behavior".into(),
                    message: format!("Behavior `{name}` has an empty body"),
                    line: span.start.line,
                    column: span.start.column,
                    severity: LintSeverity::Warning,
                });
            }
        }
    }
}

fn lint_imports(source: &str, program: &Program, issues: &mut Vec<LintIssue>) {
    let Program::Program { imports, .. } = program;
    for import in imports {
        let ImportDecl::ImportDecl { path, span } = import;
        let needle = path.split('.').next_back().unwrap_or(path.as_str());
        let referenced = source.matches(needle).count() > 1
            || source.contains(&format!("{path}::"))
            || source.contains(&format!("from {path}"))
            || is_std_import(path);
        if !referenced {
            issues.push(LintIssue {
                rule: "unused-import".into(),
                message: format!("Import `{path}` appears unused"),
                line: span.start.line,
                column: span.start.column,
                severity: LintSeverity::Warning,
            });
        }
    }
}

fn is_std_import(path: &str) -> bool {
    path.starts_with("std.") || path.starts_with("sensors.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_missing_module_and_trailing_space() {
        let source = "robot R {  \n  actuator wheels: DifferentialDrive;\n}\n";
        let report = lint(source).expect("lint should parse");
        assert!(report.issues.iter().any(|i| i.rule == "missing-module"));
        assert!(report
            .issues
            .iter()
            .any(|i| i.rule == "trailing-whitespace"));
    }

    #[test]
    fn detects_empty_test_block() {
        let source = r#"
module tests;

test "noop" {
}
"#;
        let report = lint(source).expect("lint should parse");
        assert!(report.issues.iter().any(|i| i.rule == "empty-test"));
    }
}
