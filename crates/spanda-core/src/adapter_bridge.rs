//! Optional subprocess bridges for production Nav2/SLAM adapter backends.

use std::process::Command;

fn bridge_command(env_key: &str) -> Option<String> {
    std::env::var(env_key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Invoke an external Nav2 bridge command when `SPANDA_NAV2_CMD` is configured.
pub fn invoke_nav2_bridge(goal: &str) -> Option<String> {
    // Spawn the configured Nav2 bridge process for a navigation goal.
    //
    // Parameters:
    // - `goal` — navigation goal label passed to the bridge
    //
    // Returns:
    // Bridge stdout on success, or None when no bridge is configured.
    //
    // Options:
    // Environment variable `SPANDA_NAV2_CMD` — executable plus args template using `{goal}`.
    //
    // Example:
    // let output = invoke_nav2_bridge("Dock A");

    let template = bridge_command("SPANDA_NAV2_CMD")?;
    let command_line = template.replace("{goal}", goal);
    let mut parts = command_line.split_whitespace();
    let program = parts.next()?;
    let output = Command::new(program)
        .args(parts)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Invoke an external SLAM bridge command when `SPANDA_SLAM_CMD` is configured.
pub fn invoke_slam_bridge(operation: &str) -> Option<String> {
    // Spawn the configured SLAM bridge process for localize/map operations.
    //
    // Parameters:
    // - `operation` — bridge operation name (e.g. `localize`, `map`)
    //
    // Returns:
    // Bridge stdout on success, or None when no bridge is configured.
    //
    // Options:
    // Environment variable `SPANDA_SLAM_CMD` — executable plus args template using `{op}`.
    //
    // Example:
    // let output = invoke_slam_bridge("localize");

    let template = bridge_command("SPANDA_SLAM_CMD")?;
    let command_line = template.replace("{op}", operation);
    let mut parts = command_line.split_whitespace();
    let program = parts.next()?;
    let output = Command::new(program)
        .args(parts)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
