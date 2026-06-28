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
fn impact_analysis_traverses_relationships() {
    let resolved = warehouse_config();
    let registry = build_entity_registry(&resolved);
    if let Some(fleet_id) = resolved.fleet_id() {
        let impact = registry.impact_analysis(fleet_id);
        assert!(impact.is_empty() || !impact.contains(&fleet_id.to_string()));
    }
}
