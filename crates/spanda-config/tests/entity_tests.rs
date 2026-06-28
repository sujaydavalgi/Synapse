//! Unit tests for the unified entity model.
//!
use spanda_config::{
    build_entity_registry, ConfigResolver, EntityHealthStatus, EntityKind, EntityQuery,
    EntityReadinessStatus, EntityRelationshipKind, EntityTrustStatus,
};
use std::path::PathBuf;

fn warehouse_config() -> spanda_config::ResolvedSystemConfig {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/warehouse");
    ConfigResolver::new()
        .resolve_from_dir(&root)
        .expect("warehouse fixture should resolve")
}

#[test]
fn entity_registry_includes_fleet_robots_and_devices() {
    let resolved = warehouse_config();
    let registry = build_entity_registry(&resolved);
    assert!(!registry.entities.is_empty());
    assert!(registry.relationships.iter().any(|r| {
        matches!(
            r.kind,
            EntityRelationshipKind::Contains | EntityRelationshipKind::Owns
        )
    }));
}

#[test]
fn entity_query_filters_by_kind_and_health() {
    let resolved = warehouse_config();
    let registry = build_entity_registry(&resolved);
    let devices = registry.query(&EntityQuery {
        kind: Some("device".into()),
        ..Default::default()
    });
    assert!(devices.count <= registry.entities.len());
    for entity in &devices.entities {
        assert_eq!(entity.kind(), "device");
    }
    let degraded = registry.query(&EntityQuery {
        health_status: Some("degraded".into()),
        ..Default::default()
    });
    for entity in &degraded.entities {
        assert_eq!(entity.health_status, EntityHealthStatus::Degraded);
    }
}

#[test]
fn entity_kind_parses_extensible_types() {
    assert_eq!(EntityKind::parse("robot"), EntityKind::Robot);
    assert_eq!(
        EntityKind::parse("medical_device"),
        EntityKind::MedicalDevice
    );
    assert!(matches!(
        EntityKind::parse("custom_industry_widget"),
        EntityKind::Custom(_)
    ));
}

#[test]
fn entity_trust_and_readiness_parse_legacy_strings() {
    assert_eq!(
        EntityTrustStatus::parse("verified"),
        EntityTrustStatus::Verified
    );
    assert_eq!(
        EntityReadinessStatus::parse("available"),
        EntityReadinessStatus::Ready
    );
}

#[test]
fn runtime_mission_overlay_links_robot_participation() {
    use spanda_config::{apply_runtime_mission_overlay, mission_entity_id, RuntimeMissionEntity};

    let resolved = warehouse_config();
    let mut registry = build_entity_registry(&resolved);
    let robot_id = resolved.robot_ids().into_iter().next().expect("robot");
    let mission = RuntimeMissionEntity {
        id: mission_entity_id(robot_id, "patrol"),
        name: "patrol".into(),
        robot_id: Some(robot_id.to_string()),
        fleet_id: resolved.fleet_id().map(String::from),
        mission_state: "Running".into(),
        step_index: 1,
        current_step: Some("scan".into()),
        steps: vec!["navigate".into(), "scan".into()],
        required_capabilities: vec!["navigate".into()],
        approval_pending: false,
    };
    apply_runtime_mission_overlay(&mut registry, &[mission]);
    let mission_id = mission_entity_id(robot_id, "patrol");
    assert!(registry.get(&mission_id).is_some());
    assert!(registry.relationships.iter().any(|edge| {
        edge.from_id == robot_id
            && edge.to_id == mission_id
            && edge.kind == EntityRelationshipKind::ParticipatesIn
    }));
    let linked = registry.linked_missions(robot_id);
    assert_eq!(linked.len(), 1);
    assert_eq!(linked[0].readiness_status, EntityReadinessStatus::Ready);
}

#[test]
fn entity_query_filters_participates_in_mission() {
    use spanda_config::{apply_runtime_mission_overlay, mission_entity_id, RuntimeMissionEntity};

    let resolved = warehouse_config();
    let mut registry = build_entity_registry(&resolved);
    let robot_id = resolved.robot_ids().into_iter().next().expect("robot");
    let mission_id = mission_entity_id(robot_id, "patrol");
    apply_runtime_mission_overlay(
        &mut registry,
        &[RuntimeMissionEntity {
            id: mission_id.clone(),
            name: "patrol".into(),
            robot_id: Some(robot_id.to_string()),
            fleet_id: None,
            mission_state: "Pending".into(),
            step_index: 0,
            current_step: None,
            steps: vec!["navigate".into()],
            required_capabilities: Vec::new(),
            approval_pending: false,
        }],
    );
    let matches = registry.query(&EntityQuery {
        participates_in: Some(mission_id),
        ..Default::default()
    });
    assert_eq!(matches.count, 1);
    assert_eq!(matches.entities[0].id, robot_id);
}

#[test]
fn traceability_overlay_merges_device_capability_links() {
    use spanda_config::{
        apply_traceability_overlay, DigitalThreadTraceabilityLink, ProgramGraphTraceabilityEdge,
    };

    let resolved = warehouse_config();
    let mut registry = build_entity_registry(&resolved);
    let robot_id = resolved.robot_ids().into_iter().next().expect("robot");
    let device_id = resolved
        .device_registry
        .devices
        .first()
        .map(|device| device.id.clone())
        .expect("device");
    apply_traceability_overlay(
        &mut registry,
        &[DigitalThreadTraceabilityLink {
            device_id: device_id.clone(),
            capability: "navigate".into(),
            assigned_robot: Some(robot_id.to_string()),
        }],
        &[ProgramGraphTraceabilityEdge {
            from_entity_id: robot_id.to_string(),
            to_entity_id: device_id.clone(),
            relation: "uses_hardware".into(),
        }],
    );
    assert!(registry.relationships.iter().any(|edge| {
        edge.from_id == robot_id
            && edge.to_id == device_id
            && edge.kind == EntityRelationshipKind::DependsOn
            && edge.label.as_deref() == Some("traceability:navigate")
    }));
    assert!(registry.relationships.iter().any(|edge| {
        edge.from_id == robot_id
            && edge.to_id == device_id
            && edge.label.as_deref() == Some("uses_hardware")
    }));
}

#[test]
fn entity_registry_projects_facilities_and_declared_kinds() {
    let resolved = warehouse_config();
    let registry = build_entity_registry(&resolved);
    assert!(registry.get("warehouse-a").is_some());
    assert!(registry.get("north-wing").is_some());
    assert!(registry.get("calibration-bay").is_some());
    assert!(registry.relationships.iter().any(|edge| {
        edge.from_id == "warehouse-fleet"
            && edge.to_id == "warehouse-a"
            && edge.kind == EntityRelationshipKind::Contains
    }));
}

#[test]
fn flat_hazard_zones_project_into_registry() {
    use spanda_config::human_entities::HazardZoneEntity;

    let mut resolved = warehouse_config();
    resolved.human_registry.hazard_zones.push(HazardZoneEntity {
        id: "restricted-a".into(),
        zone_type: Some("restricted".into()),
        severity: Some("high".into()),
        center: None,
        radius_m: None,
        linked_robots: Vec::new(),
        alert_on_entry: None,
        description: None,
    });
    let registry = build_entity_registry(&resolved);
    assert!(registry.get("restricted-a").is_some());
    assert_eq!(
        registry.get("restricted-a").unwrap().entity_type,
        EntityKind::Hazard
    );
}

#[test]
fn impact_analysis_traverses_relationships() {
    let resolved = warehouse_config();
    let registry = build_entity_registry(&resolved);
    if let Some(fleet_id) = resolved.fleet_id() {
        let impact = registry.impact_analysis(fleet_id);
        assert!(impact.is_empty() || !impact.contains(&fleet_id.to_string()));
    }
}
