//! lint support for Spanda.
//!
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
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.has_errors();

        // Call issues on the current instance.
        self.issues
            .iter()
            .any(|i| i.severity == LintSeverity::Error)
    }
}

pub fn lint(source: &str) -> Result<LintReport, SpandaError> {
    // Lint.
    //
    // Parameters:
    // - `source` — input value
    //
    // Returns:
    // Success value on completion, or an error.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::lint::lint(source);

    // Tokenize the source before parsing.
    let tokens = crate::lexer::tokenize(source)?;
    let program = crate::parser::parse(tokens)?;
    Ok(lint_program(source, &program))
}

fn lint_concurrency(program: &Program, issues: &mut Vec<LintIssue>) {
    // Lint concurrency.
    //
    // Parameters:
    // - `program` — input value
    // - `issues` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::lint::lint_concurrency(program, issues);

    // Extract robot declarations from the parsed program.
    let Program::Program { robots, .. } = program;

    // Handle each robot declared in the program.
    for robot in robots {
        let RobotDecl::RobotDecl {
            behaviors, tasks, ..
        } = robot;

        // Process each behavior.
        for behavior in behaviors {
            let BehaviorDecl::BehaviorDecl { body, .. } = behavior;
            lint_stmt_channel_flow(body, issues);
        }

        // Process each task.
        for task in tasks {
            let TaskDecl::TaskDecl { body, .. } = task;
            lint_stmt_channel_flow(body, issues);
        }
    }
}

fn lint_stmt_channel_flow(stmts: &[Stmt], issues: &mut Vec<LintIssue>) {
    // Lint stmt channel flow.
    //
    // Parameters:
    // - `stmts` — input value
    // - `issues` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::lint::lint_stmt_channel_flow(stmts, issues);

    // Create mutable channels for accumulating results.
    let mut channels: std::collections::HashMap<String, (bool, bool, u32, u32)> =
        std::collections::HashMap::new();
    collect_channel_flow(stmts, &mut channels);

    // Iterate over channels with destructured elements.
    for (name, (sent, recv, line, column)) in channels {
        // Take this path when recv && !sent.
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

        // Take this path when sent && !recv.
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
    // Collect channel flow.
    //
    // Parameters:
    // - `stmts` — input value
    // - `channels` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::lint::collect_channel_flow(stmts, channels);

    // Execute each statement in sequence.
    for stmt in stmts {
        // Match on stmt and handle each case.
        match stmt {
            Stmt::VarDecl {
                name, init, span, ..
            } => {
                #[allow(clippy::collapsible_if)]
                // Emit output when init provides a value.
                if let Some(value) = init {
                    // Keep entries that match the expected pattern.
                    if matches!(value, Expr::CallExpr { callee, .. }

                        // Keep entries that match the expected pattern.
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

                // Emit output when else branch provides a else branch.
                if let Some(else_branch) = else_branch {
                    collect_channel_flow(else_branch, channels);
                }
            }
            Stmt::LoopStmt { body, .. } => collect_channel_flow(body, channels),
            Stmt::ParallelStmt { body, .. } => collect_channel_flow(body, channels),
            Stmt::SelectStmt { arms, .. } => {
                // Process each arm.
                for arm in arms {
                    // Match on channel and handle each case.
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
                            // Take this path when let Expr::IdentExpr { name: fn name, .. } = callee.as ref().
                            if let Expr::IdentExpr { name: fn_name, .. } = callee.as_ref() {
                                // Take the branch when fn name equals "recv".
                                if fn_name == "recv" {
                                    // Take this path when let Some(Expr::IdentExpr { name, span }) = args.first().
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
    // Mark channel usage.
    //
    // Parameters:
    // - `expr` — input value
    // - `channels` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::lint::mark_channel_usage(expr, channels);

    // Match on expr and handle each case.
    match expr {
        Expr::CallExpr { callee, args, .. } => {
            // Take this path when let Expr::IdentExpr { name: fn name, .. } = callee.as ref().
            if let Expr::IdentExpr { name: fn_name, .. } = callee.as_ref() {
                // Take the branch when fn name equals "send" || fn name == "recv".
                if fn_name == "send" || fn_name == "recv" {
                    // Take this path when let Some(Expr::IdentExpr { name, span }) = args.first().
                    if let Some(Expr::IdentExpr { name, span }) = args.first() {
                        let entry = channels.entry(name.clone()).or_insert((
                            false,
                            false,
                            span.start.line,
                            span.start.column,
                        ));

                        // Take the branch when fn name equals "send".
                        if fn_name == "send" {
                            entry.0 = true;
                        } else {
                            entry.1 = true;
                        }
                    }
                }
            }

            // Apply each command-line argument.
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

            // Apply each command-line argument.
            for arg in args {
                mark_channel_usage(arg, channels);
            }
        }
        _ => {}
    }
}

fn lint_program(source: &str, program: &Program) -> LintReport {
    // Lint program.
    //
    // Parameters:
    // - `source` — input value
    // - `program` — input value
    //
    // Returns:
    // LintReport.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::lint::lint_program(source, program);

    // Create mutable issues for accumulating results.
    let mut issues = Vec::new();
    lint_source_style(source, &mut issues);
    lint_program_structure(program, &mut issues);
    lint_imports(source, program, &mut issues);
    lint_concurrency(program, &mut issues);
    LintReport { issues }
}

fn lint_source_style(source: &str, issues: &mut Vec<LintIssue>) {
    // Lint source style.
    //
    // Parameters:
    // - `source` — input value
    // - `issues` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::lint::lint_source_style(source, issues);

    // Iterate over enumerate with destructured elements.
    for (idx, line) in source.lines().enumerate() {
        let line_no = idx as u32 + 1;

        // Take this path when line.ends with(' ') || line.ends with('\t').
        if line.ends_with(' ') || line.ends_with('\t') {
            issues.push(LintIssue {
                rule: "trailing-whitespace".into(),
                message: "Line has trailing whitespace".into(),
                line: line_no,
                column: line.len() as u32,
                severity: LintSeverity::Warning,
            });
        }

        // Take this path when line.len() > 120.
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
    // Lint program structure.
    //
    // Parameters:
    // - `program` — input value
    // - `issues` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::lint::lint_program_structure(program, issues);

    // Destructure the program into its top-level sections.
    let Program::Program {
        module_name,
        tests,
        robots,
        ..
    } = program;

    // Take this path when module name.is none().
    if module_name.is_none() {
        issues.push(LintIssue {
            rule: "missing-module".into(),
            message: "Program has no `module` declaration".into(),
            line: 1,
            column: 1,
            severity: LintSeverity::Warning,
        });
    }

    // Run each test block in program order.
    for test in tests {
        // Skip further work when body is empty.
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

    // Handle each robot declared in the program.
    for robot in robots {
        let RobotDecl::RobotDecl { behaviors, .. } = robot;

        // Process each behavior.
        for behavior in behaviors {
            let BehaviorDecl::BehaviorDecl {
                name, body, span, ..
            } = behavior;

            // Skip further work when body is empty.
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
    // Lint imports.
    //
    // Parameters:
    // - `source` — input value
    // - `program` — input value
    // - `issues` — input value
    //
    // Returns:
    // Nothing.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::lint::lint_imports(source, program, issues);

    // Destructure the program into its top-level sections.
    let Program::Program { imports, .. } = program;

    // Emit codegen metadata for each import.
    for import in imports {
        let ImportDecl::ImportDecl { path, span } = import;
        let needle = path.split('.').next_back().unwrap_or(path.as_str());
        let referenced = source.matches(needle).count() > 1
            || source.contains(&format!("{path}::"))
            || source.contains(&format!("from {path}"))
            || is_std_import(path);

        // Take the branch when referenced is false.
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
    //
    // Parameters:
    // - `path` — input value
    //
    // Returns:
    // true or false.
    //
    // Options:
    // None.
    //
    // Example:
    // let result = spanda_core::lint::is_std_import(path);

    // Produce ") as the result.
    path.starts_with("std.") || path.starts_with("sensors.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_missing_module_and_trailing_space() {
        // Detects missing module and trailing space.
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
        // let result = spanda_core::lint::detects_missing_module_and_trailing_space();

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
        // Detects empty test block.
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
        // let result = spanda_core::lint::detects_empty_test_block();

        let source = r#"
module tests;

test "noop" {
}
"#;
        let report = lint(source).expect("lint should parse");
        assert!(report.issues.iter().any(|i| i.rule == "empty-test"));
    }
}
