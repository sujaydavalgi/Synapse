//! Continuity CLI golden output tests.

use std::path::PathBuf;
use std::process::Command;

fn spanda_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_spanda")
        .map(PathBuf::from)
        .expect("CARGO_BIN_EXE_spanda not set (run via cargo test -p spanda-cli)")
}

fn warehouse_example() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/showcase/continuity/warehouse.sd")
}

#[test]
fn continuity_cli_json_reports_successor() {
    let output = Command::new(spanda_bin())
        .args([
            "continuity",
            warehouse_example().to_str().unwrap(),
            "--failed",
            "ScannerAlpha",
            "--progress",
            "72",
            "--trigger",
            "robot_failed",
            "--json",
        ])
        .output()
        .expect("run continuity");
    assert!(
        output.status.success(),
        "continuity failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let body = String::from_utf8_lossy(&output.stdout);
    assert!(body.contains("ScannerBeta") || body.contains("successor"));
    assert!(body.contains("WarehouseInventoryScan") || body.contains("mission"));
}

#[test]
fn succession_cli_json_ranks_candidates() {
    let output = Command::new(spanda_bin())
        .args([
            "succession",
            warehouse_example().to_str().unwrap(),
            "--failed",
            "ScannerAlpha",
            "--scope",
            "fleet",
            "--json",
        ])
        .output()
        .expect("run succession");
    assert!(
        output.status.success(),
        "succession failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let body = String::from_utf8_lossy(&output.stdout);
    assert!(body.contains("rankings") || body.contains("candidates"));
}
