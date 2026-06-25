//! Integration tests for cascading TOML configuration.

use spanda_config::{
    diff_configs, generate_report_bundle, load_config_value, merge_values, ConfigResolver,
    SpandaManifest, ValidationSeverity,
};
use std::collections::HashMap;
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/warehouse")
}

#[test]
fn parses_root_manifest() {
    let root = fixture_root();
    let manifest = SpandaManifest::load_from_dir(&root).expect("manifest");
    let project = manifest.project.expect("project");
    assert_eq!(project.name, "Warehouse Patrol");
    assert_eq!(
        manifest.config.devices.as_deref(),
        Some("spanda.devices.toml")
    );
    assert_eq!(manifest.extends.base.as_deref(), Some("configs/base.toml"));
}

#[test]
fn resolves_cascading_layers() {
    let root = fixture_root();
    let resolved = ConfigResolver::new()
        .resolve_from_dir(&root)
        .expect("resolve");
    assert_eq!(resolved.project_name(), "Warehouse Patrol");
    assert!(resolved.layers_applied.iter().any(|l| l.contains("base")));
    assert!(resolved
        .fragments_loaded
        .iter()
        .any(|l| l.contains("devices")));
    let env = resolved.section("environment").and_then(|v| v.get("name"));
    assert_eq!(env.and_then(|v| v.as_str()), Some("warehouse-a"));
    let interval = resolved
        .section("health")
        .and_then(|h| h.get("default_interval_ms"))
        .and_then(|v| v.as_integer());
    assert_eq!(interval, Some(3000));
}

#[test]
fn parses_device_hierarchy() {
    let root = fixture_root();
    let resolved = ConfigResolver::new()
        .resolve_from_dir(&root)
        .expect("resolve");
    let fleet = resolved.device_tree.fleet.as_ref().expect("fleet");
    assert_eq!(fleet.id, "warehouse-fleet");
    assert_eq!(fleet.robots.len(), 1);
    let robot = &fleet.robots[0];
    assert_eq!(robot.id, "rover-001");
    let compute = robot.compute.as_ref().expect("compute");
    assert_eq!(compute.devices.len(), 3);
}

#[test]
fn validates_providers_and_ports() {
    let root = fixture_root();
    let resolved = ConfigResolver::new()
        .with_validation(true)
        .resolve_from_dir(&root)
        .expect("resolve");
    assert!(
        resolved.validation.passed,
        "{:?}",
        resolved.validation.findings
    );
}

#[test]
fn detects_port_conflict() {
    let base = r#"
[fleet]
id = "f1"
[[fleet.robots]]
id = "r1"
[fleet.robots.compute]
id = "c1"
type = "JetsonOrin"
[[fleet.robots.compute.devices]]
id = "d1"
type = "GPS"
provider = "spanda-gps"
port = "/dev/ttyUSB0"
[[fleet.robots.compute.devices]]
id = "d2"
type = "Lidar"
provider = "spanda-lidar"
port = "/dev/ttyUSB0"
"#;
    let value: toml::Value = toml::from_str(base).unwrap();
    let tree = spanda_config::DeviceTree::from_toml_value(&value);
    let report = spanda_config::validate_device_tree(&tree, &[]);
    assert!(!report.passed);
    assert!(report
        .findings
        .iter()
        .any(|f| f.code == "device.port_conflict"));
}

#[test]
fn merge_strategy_append() {
    let base: toml::Value = toml::from_str("tags = [\"a\"]").unwrap();
    let overlay: toml::Value = toml::from_str("tags = [\"b\"]").unwrap();
    let mut hints = HashMap::new();
    hints.insert("tags".into(), spanda_config::MergeStrategyHint::Append);
    let merged = merge_values(base, overlay, &hints, "root").unwrap();
    let tags = merged.get("tags").unwrap().as_array().unwrap();
    assert_eq!(tags.len(), 2);
}

#[test]
fn merge_strategy_replace_arrays() {
    let base: toml::Value = toml::from_str("tags = [\"a\", \"b\"]").unwrap();
    let overlay: toml::Value = toml::from_str("tags = [\"c\"]").unwrap();
    let merged = merge_values(base, overlay, &HashMap::new(), "root").unwrap();
    let tags = merged.get("tags").unwrap().as_array().unwrap();
    assert_eq!(tags.len(), 1);
}

#[test]
fn merge_by_id_robots() {
    let base: toml::Value = toml::from_str(
        r#"[[robots]]
id = "r1"
model = "A""#,
    )
    .unwrap();
    let overlay: toml::Value = toml::from_str(
        r#"[[robots]]
id = "r1"
hardware_profile = "RoverV1""#,
    )
    .unwrap();
    let mut hints = HashMap::new();
    hints.insert("robots".into(), spanda_config::MergeStrategyHint::MergeById);
    let merged = merge_values(base, overlay, &hints, "root").unwrap();
    let robot = merged
        .get("robots")
        .and_then(|r| r.as_array())
        .and_then(|a| a.first())
        .unwrap();
    assert_eq!(robot.get("model").and_then(|v| v.as_str()), Some("A"));
    assert_eq!(
        robot.get("hardware_profile").and_then(|v| v.as_str()),
        Some("RoverV1")
    );
}

#[test]
fn config_diff_detects_changes() {
    let left: toml::Value = toml::from_str("[fleet]\nid = \"a\"").unwrap();
    let right: toml::Value = toml::from_str("[fleet]\nid = \"b\"").unwrap();
    let lines = diff_configs(&left, &right);
    assert!(!lines.is_empty());
}

#[test]
fn json_config_loads() {
    let json = r#"{"fleet":{"id":"json-fleet"}}"#;
    let value = spanda_config::parse_config_str(json, std::path::Path::new("test.json")).unwrap();
    assert_eq!(
        value
            .get("fleet")
            .and_then(|f| f.get("id"))
            .and_then(|v| v.as_str()),
        Some("json-fleet")
    );
}

#[test]
fn logical_physical_mapping() {
    let root = fixture_root();
    let resolved = ConfigResolver::new()
        .resolve_from_dir(&root)
        .expect("resolve");
    assert_eq!(resolved.logical_map.sensors.len(), 3);
    assert_eq!(resolved.logical_map.actuators.len(), 2);
    assert!(resolved.logical_map.actuators["drive-controller"].has_emergency_stop);
    assert!(resolved.logical_map.actuators["wheels"].has_emergency_stop);
}

#[test]
fn generates_config_report() {
    let root = fixture_root();
    let resolved = ConfigResolver::new()
        .with_validation(true)
        .resolve_from_dir(&root)
        .expect("resolve");
    let bundle = generate_report_bundle(&resolved);
    assert_eq!(bundle.resolved.project, "Warehouse Patrol");
    assert!(!bundle.device_hierarchy.is_empty());
    assert!(bundle
        .health
        .robots_with_policy
        .contains(&"rover-001".to_string()));
}

#[test]
fn flags_missing_emergency_stop() {
    let toml_str = r#"
[fleet]
id = "f1"
[[fleet.robots]]
id = "r1"
[fleet.robots.compute]
id = "c1"
type = "JetsonOrin"
[[fleet.robots.compute.devices]]
id = "drive"
type = "DifferentialDrive"
provider = "spanda-canbus"
bus = "can0"
capabilities = ["move", "stop"]
"#;
    let value: toml::Value = toml::from_str(toml_str).unwrap();
    let tree = spanda_config::DeviceTree::from_toml_value(&value);
    let report = spanda_config::validate_device_tree(&tree, &["spanda-canbus".into()]);
    assert!(!report.passed);
    assert!(report
        .findings
        .iter()
        .any(|f| f.code == "safety.no_emergency_stop"));
}

#[test]
fn config_graph_has_nodes() {
    let root = fixture_root();
    let resolved = ConfigResolver::new()
        .resolve_from_dir(&root)
        .expect("resolve");
    assert!(!resolved.graph.nodes.is_empty());
    assert!(!resolved.graph.merge_order.is_empty());
}

#[test]
fn duplicate_serial_detected() {
    let toml_str = r#"
[fleet]
id = "f1"
[[fleet.robots]]
id = "r1"
[fleet.robots.compute]
id = "c1"
type = "JetsonOrin"
serial = "SN-1"
[[fleet.robots]]
id = "r2"
[fleet.robots.compute]
id = "c2"
type = "JetsonOrin"
serial = "SN-1"
"#;
    let value: toml::Value = toml::from_str(toml_str).unwrap();
    let tree = spanda_config::DeviceTree::from_toml_value(&value);
    let report = spanda_config::validate_device_tree(&tree, &[]);
    assert!(report
        .findings
        .iter()
        .any(|f| f.code == "compute.duplicate_serial"));
}

#[test]
fn untrusted_actuator_rejected() {
    let toml_str = r#"
[fleet]
id = "f1"
[[fleet.robots]]
id = "r1"
[fleet.robots.compute]
id = "c1"
type = "JetsonOrin"
[[fleet.robots.compute.devices]]
id = "drive"
type = "DifferentialDrive"
provider = "spanda-canbus"
bus = "can0"
trusted = false
capabilities = ["move", "stop", "emergency_stop"]
"#;
    let value: toml::Value = toml::from_str(toml_str).unwrap();
    let tree = spanda_config::DeviceTree::from_toml_value(&value);
    let report = spanda_config::validate_device_tree(&tree, &["spanda-canbus".into()]);
    assert!(report
        .findings
        .iter()
        .any(|f| f.code == "security.untrusted_actuator"));
}

#[test]
fn loads_json_file_from_disk() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cfg.json");
    std::fs::write(&path, r#"{"test": true}"#).unwrap();
    let value = load_config_value(&path).unwrap();
    assert_eq!(value.get("test").and_then(|v| v.as_bool()), Some(true));
}

#[test]
fn validation_severity_counts() {
    let root = fixture_root();
    let resolved = ConfigResolver::new()
        .with_validation(true)
        .resolve_from_dir(&root)
        .expect("resolve");
    let errors = resolved.validation.error_count();
    let warnings = resolved.validation.warning_count();
    assert_eq!(errors + warnings, resolved.validation.findings.len());
    assert!(resolved.validation.findings.iter().all(|f| {
        matches!(
            f.severity,
            ValidationSeverity::Error | ValidationSeverity::Warning | ValidationSeverity::Info
        )
    }));
}

#[test]
fn resolve_for_source_from_project_dir() {
    let root = fixture_root();
    let cfg = spanda_config::resolve_for_source(&root, None, true)
        .expect("resolve")
        .expect("config");
    assert_eq!(cfg.project_name(), "Warehouse Patrol");
}

#[test]
fn health_inject_faults_uses_robot_policy() {
    let root = fixture_root();
    let resolved = ConfigResolver::new()
        .resolve_from_dir(&root)
        .expect("resolve");
    let faults = spanda_config::health_inject_faults(&resolved, "rover-001");
    assert!(!faults.is_empty());
}

#[test]
fn provider_packages_include_device_tree_providers() {
    let root = fixture_root();
    let resolved = ConfigResolver::new()
        .resolve_from_dir(&root)
        .expect("resolve");
    let packages = spanda_config::provider_packages_for_runtime(&resolved);
    assert!(!packages.is_empty());
}

#[test]
fn assurance_policy_reads_minimum_score() {
    let root = fixture_root();
    let resolved = ConfigResolver::new()
        .resolve_from_dir(&root)
        .expect("resolve");
    let policy = spanda_config::assurance_policy(&resolved);
    assert_eq!(policy.minimum_score, 70);
}

#[test]
fn parses_flat_device_identity_registry() {
    let root = fixture_root();
    let resolved = ConfigResolver::new()
        .resolve_from_dir(&root)
        .expect("resolve");
    let camera = resolved
        .device_registry
        .get("camera-front-001")
        .expect("camera");
    assert_eq!(camera.logical_name.as_deref(), Some("front_camera"));
    assert_eq!(camera.ip_address.as_deref(), Some("192.168.1.42"));
    assert_eq!(camera.protocol.as_deref(), Some("rtsp"));
}

#[test]
fn logical_map_uses_configured_logical_names() {
    let root = fixture_root();
    let resolved = ConfigResolver::new()
        .resolve_from_dir(&root)
        .expect("resolve");
    let sensor = resolved
        .logical_map
        .sensors
        .get("front_camera")
        .expect("front_camera mapping");
    assert_eq!(sensor.physical_device_id, "camera-front-001");
}

#[test]
fn detects_duplicate_ip_in_registry() {
    use spanda_config::{validate_device_registry, DeviceIdentityRecord, DeviceRegistry};
    let registry = DeviceRegistry {
        devices: vec![
            DeviceIdentityRecord {
                id: "a".into(),
                device_type: "Camera".into(),
                ip_address: Some("10.0.0.1".into()),
                ..Default::default()
            },
            DeviceIdentityRecord {
                id: "b".into(),
                device_type: "Camera".into(),
                ip_address: Some("10.0.0.1".into()),
                ..Default::default()
            },
        ],
    };
    let report = validate_device_registry(&registry, &[]);
    assert!(!report.passed);
    assert!(report
        .findings
        .iter()
        .any(|f| f.code == "device.duplicate_ip"));
}

#[test]
fn subnet_parser_emits_hosts() {
    let subnet = spanda_config::Ipv4Subnet::parse("192.168.1.0/24").expect("cidr");
    let hosts = subnet.hosts();
    assert!(!hosts.is_empty());
    assert!(hosts.len() <= 254);
}

#[test]
fn network_report_includes_traceability() {
    let root = fixture_root();
    let resolved = ConfigResolver::new()
        .resolve_from_dir(&root)
        .expect("resolve");
    let bundle = spanda_config::generate_report_bundle(&resolved);
    assert!(bundle.network.networked_devices >= 1);
    assert!(!bundle.network.traceability.is_empty());
}

#[test]
fn config_drift_passes_for_identical_resolved_configs() {
    let root = fixture_root();
    let resolved = ConfigResolver::new()
        .resolve_from_dir(&root)
        .expect("resolve");
    let report = spanda_config::detect_config_drift(&resolved, &resolved);
    assert!(report.passed);
    assert!(report.findings.is_empty());
}

#[test]
fn config_drift_detects_device_ip_change() {
    let root = fixture_root();
    let baseline = ConfigResolver::new()
        .resolve_from_dir(&root)
        .expect("resolve");
    let mut current = baseline.clone();
    let device = current
        .device_registry
        .devices
        .iter_mut()
        .find(|d| d.id == "camera-front-001")
        .expect("camera");
    device.ip_address = Some("10.0.0.99".into());
    let report = spanda_config::detect_config_drift(&baseline, &current);
    assert!(!report.passed);
    assert!(report.findings.iter().any(|f| f.message.contains("ip")));
}
