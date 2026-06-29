//! Automotive sensor hub and live-read integration tests.
//!
use spanda_providers::radar_env_lock::RadarEnvLock;
use spanda_providers::{
    bootstrap_providers_for_packages, dispatch_official_package_call, read_radar_distance,
    seed_automotive_demos,
};
use spanda_runtime::value::RuntimeValue;

fn clear_live_radar_env() {
    std::env::remove_var("SPANDA_LIVE_RADAR");
    std::env::remove_var("SPANDA_RADAR_CMD");
}

#[test]
fn automotive_hub_seeds_radar_distance() {
    let _lock = RadarEnvLock::acquire().expect("radar env lock");
    clear_live_radar_env();
    seed_automotive_demos();
    let value = read_radar_distance("front-radar");
    assert!((value - 25.0).abs() < f64::EPSILON);
}

#[test]
fn package_dispatch_reads_radar_when_capability_granted() {
    let _lock = RadarEnvLock::acquire().expect("radar env lock");
    clear_live_radar_env();
    let mut registry = bootstrap_providers_for_packages(&["spanda-radar"]);
    let value = dispatch_official_package_call(
        &mut registry,
        "sensors.radar",
        "read",
        &[],
        None,
        None,
        0.0,
    )
    .expect("radar read dispatch");
    match value {
        RuntimeValue::Number { value, .. } => assert!(value > 0.0),
        other => panic!("expected number, got {other:?}"),
    }
}

#[test]
fn live_radar_cmd_overrides_hub_stub() {
    let _lock = RadarEnvLock::acquire().expect("radar env lock");
    clear_live_radar_env();
    std::env::set_var("SPANDA_LIVE_RADAR", "1");
    std::env::set_var("SPANDA_RADAR_CMD", "echo 99.0");
    seed_automotive_demos();
    let value = read_radar_distance("front-radar");
    assert!((value - 99.0).abs() < f64::EPSILON);
    clear_live_radar_env();
}
