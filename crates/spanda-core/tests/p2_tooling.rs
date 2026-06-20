use spanda_core::{format_ast, format_source, generate_markdown, lint};

#[test]
fn ast_formatter_normalizes_spacing() {
    let source = "module m;\nexport fn f(x:Int)->Int{return x;}\n";
    let formatted = format_ast(source).expect("should parse");
    assert!(formatted.contains("export fn f(x: Int) -> Int"));
    assert!(formatted.contains("return x;"));
}

#[test]
fn format_source_falls_back_on_parse_error() {
    let source = "not valid spanda {{{\n";
    let formatted = format_source(source);
    assert!(formatted.ends_with('\n'));
}

#[test]
fn lint_reports_missing_module() {
    let source = "robot R {\n  actuator wheels: DifferentialDrive;\n}\n";
    let report = lint(source).expect("lint parses robot");
    assert!(report.issues.iter().any(|i| i.rule == "missing-module"));
}

#[test]
fn doc_generator_emits_markdown() {
    let source = r#"
module math;

export fn double(x: Int) -> Int {
  return x;
}
"#;
    let md = generate_markdown(source).expect("docs");
    assert!(md.contains("# Module `math`"));
    assert!(md.contains("double"));
}
