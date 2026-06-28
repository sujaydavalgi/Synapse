//! Cascading TOML configuration resolution for Spanda autonomous systems.
//!
//! This crate sits between project/package loading and verification/runtime.
//! Downstream consumers should use [`ResolvedSystemConfig`] rather than raw
//! TOML or JSON files.
//!
pub mod config_approval;
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
pub mod discovery_live;
pub mod discovery_registry;
pub mod discovery_tls;
pub mod discovery_transport;
pub mod drift;
pub mod entity;
pub mod error;
pub mod facility;
pub mod human_entities;
pub mod integration;
pub mod json;
pub mod layer;
pub mod manifest;
pub mod mapping;
pub mod mission_approval;
pub mod network_validation;
pub mod operational_drift;
pub mod provisioning;
pub mod reports;
pub mod resolved;
pub mod resolver;
pub mod snapshot_encryption;
pub mod system_context;
pub mod validation;

pub use config_approval::{
    append_evidence_record, approval_policy_required_count, approval_quorum_met,
    approve_config_request, default_approvals_path, default_evidence_log_path,
    list_evidence_records, load_approval_queue, reject_config_request, save_approval_queue,
    submit_config_approval, ConfigApprovalQueue, ConfigApprovalRequest, ConfigApprovalStatus,
    ConfigApprovalVote,
};
pub use config_snapshots::{
    default_snapshots_dir, list_config_snapshots, load_config_snapshot, publish_config_snapshot,
    save_config_snapshot, ConfigPublishResult, ConfigSnapshot, ConfigSnapshotMeta,
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
pub use discovery_live::default_discovery_subnet;
pub use discovery_registry::{
    discovery_package_for_transport, is_registry_discovery_package_installed,
    list_installed_discovery_packages,
};
pub use discovery_tls::{discovery_tls_policy, discovery_tls_summary, validate_discovery_endpoint};
pub use discovery_transport::{
    discovery_match_to_record, discovery_transport_by_name, ingest_discovery_matches,
    run_discovery_transports, DeviceDiscoveryTransport, DiscoveryOptions, DiscoveryTransportResult,
    MockBleDiscoveryTransport, MockCanDiscoveryTransport, MockMdnsDiscoveryTransport,
    MockMqttDiscoveryTransport, MockRos2DiscoveryTransport, MockUsbDiscoveryTransport,
    SubnetDiscoveryTransport,
};
pub use drift::{
    append_agent_drift, append_program_drift, detect_agent_drift, detect_config_drift,
    expected_agent_states, format_drift_lines, AgentDriftSnapshot, ConfigDriftReport,
    DriftDimension, DriftFinding, DriftSeverity, ExpectedAgentState,
};
pub use entity::{
    apply_runtime_mission_overlay, apply_traceability_overlay, build_entity_registry,
    mission_entity_id, runtime_missions_from_approval_seeds, DigitalThreadTraceabilityLink,
    EntityAuditInfo, EntityGraph, EntityHealthStatus, EntityKind, EntityLifecycleState,
    EntityLocation, EntityQuery, EntityQueryResult, EntityReadinessStatus, EntityRecord,
    EntityRegistry, EntityRelationship, EntityRelationshipKind, EntitySecurityIdentity,
    EntityTrustStatus, ProgramGraphTraceabilityEdge, RuntimeMissionEntity,
};
pub use error::{ConfigError, ConfigResult};
pub use facility::{
    BuildingEntity, DeclaredEntityKind, FacilityEntity, FacilityRegistry, ZoneEntity,
};
pub use human_entities::{
    is_operator_capability, operator_capability_names, records_from_human_registry,
    CertificationRecord, ControlCenterEntity, HumanEntity, HumanRegistry, RemoteExpertSession,
    SpatialDeviceEntity, WearableEntity,
};
pub use integration::{
    config_flag_from_args, configured_human_ids, configured_robot_ids, default_verify_target,
    ensure_config_valid, official_packages_from_resolved, recovery_knowledge_path,
    resolve_for_source, resolve_for_source_or_exit, verify_with_system_config,
};
pub use json::{load_config_value, parse_config_str};
pub use layer::{ConfigGraph, ConfigGraphEdge, ConfigLayer, ConfigMergeStrategy};
pub use manifest::{
    ConfigReferences, ExtendsSection, MergeStrategyHint, ProjectSection, SpandaManifest,
    MANIFEST_FILENAME,
};
pub use mapping::{ActuatorMapping, LogicalPhysicalMap, RobotMapping, SensorMapping};
pub use mission_approval::{
    default_mission_approvals_path, load_mission_approval_queue, merge_mission_approval_seeds,
    resolve_mission_approval, save_mission_approval_queue, MissionApprovalQueue,
    MissionApprovalRecord, MissionApprovalSeed, MissionApprovalStatus,
};
pub use network_validation::validate_device_registry;
pub use operational_drift::{
    detect_operational_drift, detect_operational_drift_full, OperationalDriftDimension,
    OperationalDriftReport,
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
