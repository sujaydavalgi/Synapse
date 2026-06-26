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
}
