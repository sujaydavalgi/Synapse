//! Cascading TOML configuration resolution for Spanda autonomous systems.
//!
//! This crate sits between project/package loading and verification/runtime.
//! Downstream consumers should use [`ResolvedSystemConfig`] rather than raw
//! TOML or JSON files.
//!
pub mod device_identity;
pub mod device_tree;
pub mod drift;
pub mod error;
pub mod integration;
pub mod json;
pub mod layer;
pub mod manifest;
pub mod mapping;
pub mod network_validation;
pub mod reports;
pub mod resolved;
pub mod resolver;
pub mod system_context;
pub mod validation;

pub use device_identity::{
    discover_matches, identity_from_device_node, scan_subnet, traceability_rows,
    DeviceIdentityRecord, DeviceRegistry, DiscoveryMatch, Ipv4Subnet, NetworkHostProbe,
    TraceabilityRow, TrustLevel,
};
pub use device_tree::{ComputeNode, DeviceNode, DeviceTree, FleetNode, RobotNode};
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
pub use drift::{
    append_program_drift, detect_config_drift, format_drift_lines, ConfigDriftReport,
    DriftDimension, DriftFinding, DriftSeverity,
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
