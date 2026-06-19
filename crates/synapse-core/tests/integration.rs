use glob::glob;
use synapse_core::{compile, run, RunOptions};

const NEGATIVE_FIXTURES: &[&str] = &["ai_safety_violation.syn"];

#[test]
fn examples_compile_and_run() {
    let pattern = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/*.syn");
    let paths: Vec<_> = glob(pattern)
        .expect("glob pattern")
        .filter_map(|entry| entry.ok())
        .collect();

    assert!(!paths.is_empty(), "expected example .syn files");

    for path in paths {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if NEGATIVE_FIXTURES.contains(&file_name) {
            continue;
        }

        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));

        compile(&source)
            .unwrap_or_else(|e| panic!("compile {} failed: {e}", path.display()));

        run(&source, RunOptions::default())
            .unwrap_or_else(|e| panic!("run {} failed: {e}", path.display()));
    }
}

#[test]
fn negative_fixture_fails_type_check() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/ai_safety_violation.syn"
    );
    let source = std::fs::read_to_string(path).expect("read ai_safety_violation.syn");
    let err = compile(&source).expect_err("expected compile failure");
    assert!(
        err.diagnostics()
            .iter()
            .any(|d| d.message.contains("SafeAction")),
        "expected SafeAction error, got: {:?}",
        err.diagnostics()
    );
}
