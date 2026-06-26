//! Cascading TOML configuration resolution for Spanda autonomous systems.
//!
//! This crate sits between project/package loading and verification/runtime.
//! Downstream consumers should use [`ResolvedSystemConfig`] rather than raw
//! TOML or JSON files.
//!
pub mod config_snapshots;
pub mod device_config_persist;
pub mod device_failover;
pub mod device_health;
pub mod device_identity;
pub mod device_operations;
pub mod device_pool;
pub mod device_quarantine;
pub mod device_reports;
pub mod device_tree;
pub mod discovery_transport;
pub mod drift;
pub mod error;
pub mod integration;
pub mod json;
pub mod layer;
pub mod manifest;
pub mod mapping;
pub mod network_validation;
pub mod operational_drift;
pub mod provisioning;
pub mod reports;
pub mod resolved;
pub mod resolver;
pub mod system_context;
pub mod validation;

pub use config_snapshots::{
    default_snapshots_dir, list_config_snapshots, load_config_snapshot, save_config_snapshot,
    ConfigSnapshot, ConfigSnapshotMeta,
};
pub use device_config_persist::{
    device_fragment_paths, persist_device_record, DevicePersistResult,
};
pub use device_failover::{
    failover_chains, next_failover_device, recovery_failover_actions, FailoverChain, FailoverMember,
};
pub use device_health::{
    evaluate_device_readiness, readiness_impact, CalibrationStatus, DeviceHealthStatus,
    ReadinessImpactReport,
};
pub use device_identity::{
    detect_identity_anomalies, discover_matches, identity_from_device_node, scan_subnet,
    traceability_rows, DeviceIdentityRecord, DeviceRegistry, DiscoveryMatch, IdentityAnomaly,
    Ipv4Subnet, NetworkHostProbe, TraceabilityRow, TrustLevel,
};
pub use device_operations::{
    export_device_mapping_json, AssignDeviceOptions, DeviceOperationResult,
};
pub use device_pool::{DeviceLifecycleState, DevicePoolEntry, DevicePoolSummary};
pub use device_quarantine::{
    can_control_actuators, can_publish_trusted_safety_data, can_satisfy_mission_capabilities,
    evaluate_quarantine_policy, QuarantinePolicyResult,
};
pub use device_reports::{
    generate_device_reports, AssignmentReport, CalibrationReport, CapabilityCoverageReport,
    DeviceInventoryReport, DeviceReportBundle, DeviceTrustReport,
};
pub use device_tree::{ComputeNode, DeviceNode, DeviceTree, FleetNode, RobotNode};
pub use discovery_transport::{
    discovery_transport_by_name, run_discovery_transports, DeviceDiscoveryTransport,
    DiscoveryOptions, DiscoveryTransportResult, MockBleDiscoveryTransport,
    MockCanDiscoveryTransport, MockMdnsDiscoveryTransport, MockMqttDiscoveryTransport,
    MockRos2DiscoveryTransport, MockUsbDiscoveryTransport, SubnetDiscoveryTransport,
};
pub use drift::{
    append_agent_drift, append_program_drift, detect_agent_drift, detect_config_drift,
    expected_agent_states, format_drift_lines, AgentDriftSnapshot, ConfigDriftReport,
    DriftDimension, DriftFinding, DriftSeverity, ExpectedAgentState,
};
pub use error::{ConfigError, ConfigResult};
pub use integration::{
    config_flag_from_args, configured_robot_ids, default_verify_target, ensure_config_valid,
    official_packages_from_resolved, recovery_knowledge_path, resolve_for_source,
    resolve_for_source_or_exit, verify_with_system_config,
};
pub use json::{load_config_value, parse_config_str};
pub use layer::{ConfigGraph, ConfigGraphEdge, ConfigLayer, ConfigMergeStrategy};
pub use manifest::{
    ConfigReferences, ExtendsSection, MergeStrategyHint, ProjectSection, SpandaManifest,
    MANIFEST_FILENAME,
};
pub use mapping::{ActuatorMapping, LogicalPhysicalMap, RobotMapping, SensorMapping};
pub use network_validation::validate_device_registry;
pub use operational_drift::{
    detect_operational_drift, OperationalDriftDimension, OperationalDriftReport,
};
pub use provisioning::{
    run_provision_workflow, ProvisionReport, ProvisionStep, ProvisionStepResult,
};
pub use reports::{
    config_drift_report, format_report_text, format_report_text_with_options,
    generate_report_bundle, ConfigReportBundle,
};
pub use resolved::ResolvedSystemConfig;
pub use resolver::{diff_configs, merge_values, ConfigResolver, ResolverOptions};
pub use system_context::{
    assurance_policy, assurance_score_from_flags, diagnosis_policy, health_inject_faults,
    mission_policy, provider_packages_for_runtime, recovery_failure_catalog, AssurancePolicy,
    DiagnosisPolicy, MissionPolicy,
};
pub use validation::{
    validate_device_tree, validate_logical_map, ConfigValidationReport, ValidationFinding,
    ValidationSeverity,
};
