use glob::glob;
use spanda_core::{compile, run, RunOptions};

const NEGATIVE_FIXTURES: &[&str] = &["ai_safety_violation.sd"];

#[test]
fn examples_compile_and_run() {
    let patterns = [
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/*.sd"),
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/communication/*.sd"
        ),
    ];
    let mut paths = Vec::new();
    for pattern in patterns {
        paths.extend(
            glob(pattern)
                .expect("glob pattern")
                .filter_map(|entry| entry.ok()),
        );
    }

    assert!(!paths.is_empty(), "expected example .sd files");

    for path in paths {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if NEGATIVE_FIXTURES.contains(&file_name) {
            continue;
        }

        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));

        compile(&source).unwrap_or_else(|e| panic!("compile {} failed: {e}", path.display()));

        run(&source, RunOptions::default())
            .unwrap_or_else(|e| panic!("run {} failed: {e}", path.display()));
    }
}

#[test]
fn negative_fixture_fails_type_check() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/ai_safety_violation.sd"
    );
    let source = std::fs::read_to_string(path).expect("read ai_safety_violation.sd");
    let err = compile(&source).expect_err("expected compile failure");
    assert!(
        err.diagnostics()
            .iter()
            .any(|d| d.message.contains("SafeAction") || d.message.contains("ActionProposal")),
        "expected SafeAction/ActionProposal error, got: {:?}",
        err.diagnostics()
    );
}
