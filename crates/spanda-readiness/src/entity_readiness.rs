//! Unified entity readiness — routes readiness engines through [`EntityRegistry`].
//!
use crate::human::evaluate_human_collaboration;
use crate::platform_events::record_readiness_platform_event;
use crate::types::{ReadinessOptions, ReadinessSeverity};
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::Program;
use spanda_config::{
    evaluate_device_readiness, readiness_impact, EntityKind, EntityReadinessStatus, EntityRecord,
    EntityRegistry, EntityRelationshipKind, ResolvedSystemConfig,
};

/// Options for entity-scoped readiness evaluation.
#[derive(Debug, Default)]
pub struct EntityReadinessOptions<'a> {
    pub program: Option<Program>,
    pub now_ms: f64,
    pub include_dependencies: bool,
    pub platform_audit: Option<&'a mut spanda_audit::AuditRuntime>,
}

/// Readiness finding for an entity evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityReadinessFinding {
    pub factor: String,
    pub severity: String,
    pub message: String,
}

/// Unified readiness report for any entity kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityReadinessReport {
    pub entity_id: String,
    pub entity_type: String,
    pub readiness_status: String,
    pub mission_ready: bool,
    pub score: Option<u32>,
    pub issues: Vec<EntityReadinessFinding>,
    pub capabilities: Vec<String>,
    pub children_checked: usize,
    pub sources: Vec<String>,
}

/// Evaluate readiness for any entity in the registry.
pub fn evaluate_entity_readiness(
    entity_id: &str,
    registry: &EntityRegistry,
    config: &ResolvedSystemConfig,
    options: &mut EntityReadinessOptions<'_>,
) -> Option<EntityReadinessReport> {
    // Evaluate operational readiness for one entity using kind-appropriate engines.
    //
    // Parameters:
    // - `entity_id` — target entity identifier
    // - `registry` — unified entity registry projection
    // - `config` — resolved system configuration
    // - `options` — optional program and dependency scope
    //
    // Returns:
    // Readiness report, or `None` when the entity id is unknown.
    //
    // Options:
    // `EntityReadinessOptions::program` enables program-scoped readiness engines.
    //
    // Example:
    // let report = evaluate_entity_readiness("rover-001", &registry, &cfg, &opts)?;

    let entity = registry.get(entity_id)?;
    let mut issues = Vec::new();
    let mut sources = vec!["entity_snapshot".into()];

    snapshot_readiness_issues(entity, &mut issues);

    let children_checked = match &entity.entity_type {
        EntityKind::Robot | EntityKind::Drone | EntityKind::Vehicle => {
            sources.push("device_pool".into());
            evaluate_robot_readiness(entity, registry, config, options, &mut issues)
        }
        EntityKind::Fleet | EntityKind::Swarm => {
            sources.push("fleet".into());
            evaluate_fleet_entity_readiness(entity, registry, config, options, &mut issues)
        }
        EntityKind::Mission => evaluate_mission_readiness(entity, registry, &mut issues),
        EntityKind::Human | EntityKind::Team => {
            sources.push("human_registry".into());
            evaluate_human_readiness(entity, config, options, &mut issues);
            0
        }
        kind if is_device_kind(kind) => {
            sources.push("device_pool".into());
            evaluate_device_readiness_entity(entity, config, options.now_ms, &mut issues);
            0
        }
        EntityKind::Package | EntityKind::Provider => {
            sources.push("provider_registry".into());
            evaluate_supply_chain_readiness(entity, config, &mut issues);
            0
        }
        EntityKind::Facility | EntityKind::Building | EntityKind::Zone => {
            rollup_child_readiness(entity, registry, &mut issues)
        }
        _ => 0,
    };

    if options.include_dependencies {
        for dep_id in registry.dependency_chain(entity_id) {
            if let Some(dep) = registry.get(&dep_id) {
                if dep.readiness_status == EntityReadinessStatus::NotReady {
                    push_issue(
                        &mut issues,
                        "dependency",
                        "high",
                        format!("Dependency '{dep_id}' is not ready"),
                    );
                }
            }
        }
    }

    let mission_ready = !issues
        .iter()
        .any(|i| i.severity == "high" || i.severity == "critical")
        && entity.readiness_status != EntityReadinessStatus::NotReady;
    let score = readiness_score(&issues, entity);
    let report = EntityReadinessReport {
        entity_id: entity.id.clone(),
        entity_type: entity.kind().to_string(),
        readiness_status: entity.readiness_status.as_str().to_string(),
        mission_ready,
        score: Some(score),
        issues,
        capabilities: entity.capabilities.clone(),
        children_checked,
        sources,
    };
    if let Some(audit) = options.platform_audit.as_mut() {
        record_readiness_platform_event(audit, &report);
    }
    Some(report)
}

fn is_device_kind(kind: &EntityKind) -> bool {
    matches!(
        kind,
        EntityKind::Device
            | EntityKind::Sensor
            | EntityKind::Actuator
            | EntityKind::Gateway
            | EntityKind::Controller
            | EntityKind::Wearable
            | EntityKind::MedicalDevice
            | EntityKind::Camera
            | EntityKind::Gps
            | EntityKind::Plc
            | EntityKind::Compute
            | EntityKind::ArDevice
            | EntityKind::VrDevice
            | EntityKind::IotDevice
            | EntityKind::DigitalTwin
    )
}

fn push_issue(
    issues: &mut Vec<EntityReadinessFinding>,
    factor: &str,
    severity: &str,
    message: impl Into<String>,
) {
    issues.push(EntityReadinessFinding {
        factor: factor.into(),
        severity: severity.into(),
        message: message.into(),
    });
}

fn snapshot_readiness_issues(entity: &EntityRecord, issues: &mut Vec<EntityReadinessFinding>) {
    if entity.readiness_status == EntityReadinessStatus::NotReady {
        push_issue(
            issues,
            "readiness",
            "high",
            format!("Entity readiness is {}", entity.readiness_status.as_str()),
        );
    } else if entity.readiness_status == EntityReadinessStatus::Partial {
        push_issue(issues, "readiness", "medium", "Entity readiness is partial");
    }
}

fn evaluate_robot_readiness(
    entity: &EntityRecord,
    registry: &EntityRegistry,
    config: &ResolvedSystemConfig,
    options: &EntityReadinessOptions<'_>,
    issues: &mut Vec<EntityReadinessFinding>,
) -> usize {
    let impact = readiness_impact(&config.device_registry, options.now_ms);
    if impact.blocked_count > 0 {
        push_issue(
            issues,
            "device_pool",
            "high",
            format!(
                "{} of {} assigned devices block mission readiness",
                impact.blocked_count, impact.total_devices
            ),
        );
    }

    let mut checked = 0usize;
    for device in devices_for_robot(registry, config, &entity.id) {
        checked += 1;
        let health = evaluate_device_readiness(&device, options.now_ms);
        if health.readiness_blocked {
            push_issue(
                issues,
                "device",
                "high",
                format!(
                    "Device '{}' blocks readiness: {}",
                    device.id,
                    health.blockers.join(", ")
                ),
            );
        }
    }

    if let Some(ref program) = options.program {
        let readiness_opts = ReadinessOptions {
            system_config: Some(std::sync::Arc::new(config.clone())),
            ..Default::default()
        };
        let report = crate::engine::evaluate_readiness_with_runtime(program, &readiness_opts, None);
        for issue in report.issues {
            push_issue(
                issues,
                &issue.factor,
                severity_name(&issue.severity),
                issue.message,
            );
        }
    }

    for mission in registry.linked_missions(&entity.id) {
        if mission.readiness_status == EntityReadinessStatus::NotReady {
            push_issue(
                issues,
                "mission",
                "medium",
                format!("Linked mission '{}' is not ready", mission.id),
            );
        }
    }
    checked
}

fn evaluate_fleet_entity_readiness(
    entity: &EntityRecord,
    registry: &EntityRegistry,
    config: &ResolvedSystemConfig,
    options: &EntityReadinessOptions<'_>,
    issues: &mut Vec<EntityReadinessFinding>,
) -> usize {
    let members: Vec<String> = registry
        .relationships_for(&entity.id)
        .iter()
        .filter(|r| {
            r.from_id == entity.id
                && matches!(
                    r.kind,
                    EntityRelationshipKind::Contains | EntityRelationshipKind::Owns
                )
        })
        .map(|r| r.to_id.clone())
        .collect();
    let mut checked = members.len();
    for member_id in &members {
        if let Some(member) = registry.get(member_id) {
            if member.readiness_status == EntityReadinessStatus::NotReady {
                push_issue(
                    issues,
                    "fleet_member",
                    "high",
                    format!("Fleet member '{member_id}' is not ready"),
                );
            }
            if member.entity_type == EntityKind::Robot {
                checked += evaluate_robot_readiness(member, registry, config, options, issues);
            }
        }
    }
    checked
}

fn evaluate_mission_readiness(
    entity: &EntityRecord,
    registry: &EntityRegistry,
    issues: &mut Vec<EntityReadinessFinding>,
) -> usize {
    if entity.readiness_status == EntityReadinessStatus::NotReady {
        push_issue(issues, "mission", "high", "Mission entity is not ready");
    }
    let participants: Vec<_> = registry
        .relationships_for(&entity.id)
        .iter()
        .filter(|r| r.kind == EntityRelationshipKind::ParticipatesIn)
        .map(|r| {
            if r.to_id == entity.id {
                r.from_id.clone()
            } else {
                r.to_id.clone()
            }
        })
        .collect();
    for participant in &participants {
        if let Some(p) = registry.get(participant) {
            if p.readiness_status == EntityReadinessStatus::NotReady {
                push_issue(
                    issues,
                    "participant",
                    "high",
                    format!("Participant '{participant}' is not ready"),
                );
            }
        }
    }
    participants.len()
}

fn evaluate_human_readiness(
    entity: &EntityRecord,
    config: &ResolvedSystemConfig,
    options: &EntityReadinessOptions<'_>,
    issues: &mut Vec<EntityReadinessFinding>,
) {
    if let Some(human) = config
        .human_registry
        .humans
        .iter()
        .find(|h| h.id == entity.id)
    {
        if !human.is_available() {
            push_issue(
                issues,
                "human",
                "high",
                format!("Operator '{}' is not available", human.id),
            );
        }
        if let Some(ref program) = options.program {
            let report = evaluate_human_collaboration(config, program);
            if !report.operator_ready {
                push_issue(
                    issues,
                    "human",
                    "medium",
                    "Human collaboration readiness failed",
                );
            }
            for issue in report.issues {
                push_issue(
                    issues,
                    &issue.factor,
                    severity_name(&issue.severity),
                    issue.message,
                );
            }
        }
    }
}

fn evaluate_device_readiness_entity(
    entity: &EntityRecord,
    config: &ResolvedSystemConfig,
    now_ms: f64,
    issues: &mut Vec<EntityReadinessFinding>,
) {
    if let Some(device) = config
        .device_registry
        .devices
        .iter()
        .find(|d| d.id == entity.id)
    {
        let health = evaluate_device_readiness(device, now_ms);
        if health.readiness_blocked {
            push_issue(
                issues,
                "device",
                "high",
                format!("Device blocks readiness: {}", health.blockers.join(", ")),
            );
        }
    }
}

fn evaluate_supply_chain_readiness(
    entity: &EntityRecord,
    config: &ResolvedSystemConfig,
    issues: &mut Vec<EntityReadinessFinding>,
) {
    let name = entity
        .provider
        .as_deref()
        .or(entity.package.as_deref())
        .unwrap_or(entity.id.as_str());
    if !config.providers.iter().any(|p| p == name) {
        push_issue(
            issues,
            "provider",
            "medium",
            format!("'{name}' not listed in resolved providers"),
        );
    }
}

fn rollup_child_readiness(
    entity: &EntityRecord,
    registry: &EntityRegistry,
    issues: &mut Vec<EntityReadinessFinding>,
) -> usize {
    let checked = entity.children_ids.len();
    for child_id in &entity.children_ids {
        if let Some(child) = registry.get(child_id) {
            if child.readiness_status == EntityReadinessStatus::NotReady {
                push_issue(
                    issues,
                    "child",
                    "medium",
                    format!("Child entity '{child_id}' is not ready"),
                );
            }
        }
    }
    checked
}

fn readiness_score(issues: &[EntityReadinessFinding], entity: &EntityRecord) -> u32 {
    let mut score: u32 = match entity.readiness_status {
        EntityReadinessStatus::Ready => 100,
        EntityReadinessStatus::Partial => 70,
        EntityReadinessStatus::NotReady => 30,
        EntityReadinessStatus::Unknown => 50,
    };
    for issue in issues {
        score = score.saturating_sub(match issue.severity.as_str() {
            "critical" => 40,
            "high" => 25,
            "medium" => 10,
            _ => 5,
        });
    }
    score
}

fn severity_name(severity: &ReadinessSeverity) -> &'static str {
    match severity {
        ReadinessSeverity::Critical => "critical",
        ReadinessSeverity::High => "high",
        ReadinessSeverity::Medium => "medium",
        ReadinessSeverity::Low => "low",
        ReadinessSeverity::Info => "info",
    }
}

fn devices_for_robot(
    registry: &EntityRegistry,
    config: &ResolvedSystemConfig,
    robot_id: &str,
) -> Vec<spanda_config::DeviceIdentityRecord> {
    let mut device_ids: Vec<String> = registry
        .relationships_for(robot_id)
        .iter()
        .filter(|r| {
            r.from_id == robot_id
                && matches!(
                    r.kind,
                    EntityRelationshipKind::Contains
                        | EntityRelationshipKind::DependsOn
                        | EntityRelationshipKind::ConnectedTo
                )
        })
        .map(|r| r.to_id.clone())
        .collect();
    if device_ids.is_empty() {
        if let Some(robot) = config
            .device_tree
            .fleet
            .as_ref()
            .and_then(|f| f.robots.iter().find(|r| r.id == robot_id))
        {
            if let Some(compute) = robot.compute.as_ref() {
                device_ids.extend(compute.devices.iter().map(|d| d.id.clone()));
            }
        }
    }
    device_ids
        .into_iter()
        .filter_map(|id| {
            config
                .device_registry
                .devices
                .iter()
                .find(|d| d.id == id)
                .cloned()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use spanda_config::{build_entity_registry, ConfigResolver};
    use std::path::PathBuf;

    fn warehouse_config() -> ResolvedSystemConfig {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../spanda-config/tests/fixtures/warehouse");
        ConfigResolver::new()
            .resolve_from_dir(&root)
            .expect("warehouse fixture")
    }

    #[test]
    fn evaluate_robot_readiness_returns_report() {
        let config = warehouse_config();
        let registry = build_entity_registry(&config);
        let mut readiness_options = EntityReadinessOptions {
                now_ms: 0.0,
                ..Default::default()
            };
        let report = evaluate_entity_readiness(
            "rover-001",
            &registry,
            &config,
            &mut readiness_options,
        )
        .expect("rover-001");
        assert_eq!(report.entity_id, "rover-001");
        assert!(report.score.is_some());
    }
}
