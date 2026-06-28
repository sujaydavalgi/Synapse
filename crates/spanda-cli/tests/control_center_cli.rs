//! CLI integration tests for control-center remote API commands.

use std::process::Command;

fn spanda_bin() -> String {
    std::env::var("CARGO_BIN_EXE_spanda").expect("CARGO_BIN_EXE_spanda not set")
}

#[test]
fn control_center_help_lists_remote_subcommands() {
    let output = Command::new(spanda_bin())
        .args(["control-center", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8_lossy(&output.stderr);
    assert!(help.contains("control-center api"));
    assert!(help.contains("control-center approvals"));
    assert!(help.contains("control-center incidents"));
    assert!(help.contains("control-center evidence"));
    assert!(help.contains("control-center devices"));
    assert!(help.contains("control-center ota"));
    assert!(help.contains("control-center sre summary"));
    assert!(help.contains("control-center api-key generate"));
}

#[test]
fn control_center_api_key_generate_exports_token() {
    let output = Command::new(spanda_bin())
        .args(["control-center", "api-key", "generate", "--export"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let line = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(line.starts_with("export SPANDA_API_KEY="));
    let token = line.trim_start_matches("export SPANDA_API_KEY=");
    assert_eq!(token.len(), 64);
    assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
}
