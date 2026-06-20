use crate::ast::*;
use crate::error::SpandaError;
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

fn lint_program(source: &str, program: &Program) -> LintReport {
    let mut issues = Vec::new();
    lint_source_style(source, &mut issues);
    lint_program_structure(program, &mut issues);
    lint_imports(source, program, &mut issues);
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
