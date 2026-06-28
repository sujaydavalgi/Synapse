//! Unified Entity Model — canonical representation for all Spanda platform objects.
//!
//! Every robot, fleet, device, human, mission, provider, package, and facility
//! is modeled as an [`EntityRecord`] in the [`EntityRegistry`]. Domain-specific
//! types (`DeviceIdentityRecord`, `HumanEntity`, `RobotNode`, …) remain the
//! source of truth in TOML; this module projects them into a consistent graph.
//!
use crate::device_identity::{DeviceIdentityRecord, DeviceRegistry};
use crate::device_pool::DeviceLifecycleState;
use crate::device_tree::{ComputeNode, DeviceNode, DeviceTree, FleetNode, RobotNode};
use crate::facility::{
    BuildingEntity, DeclaredEntityKind, FacilityEntity, FacilityRegistry, ZoneEntity,
};
use crate::human_entities::{
    HazardZoneEntity, HumanEntity, HumanRegistry, RemoteExpertSession, TwinEntity, WearableEntity,
};
use crate::mapping::LogicalPhysicalMap;
use crate::resolved::ResolvedSystemConfig;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Extensible entity type taxonomy for the Spanda platform.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Human,
    Robot,
    Drone,
    Vehicle,
    Fleet,
    Swarm,
    Team,
    Device,
    Sensor,
    Actuator,
    Gateway,
    Controller,
    Wearable,
    MedicalDevice,
    Camera,
    Gps,
    Plc,
    AiAgent,
    CloudService,
    EdgeService,
    DigitalTwin,
    Provider,
    Package,
    Mission,
    Facility,
    Building,
    Zone,
    Hazard,
    Incident,
    CommandCenter,
    ControlCenter,
    Organization,
    Compute,
    ArDevice,
    VrDevice,
    IotDevice,
    SpatialSession,
    /// User-defined or future industry-specific entity type.
    Custom(String),
}

impl EntityKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Human => "human",
            Self::Robot => "robot",
            Self::Drone => "drone",
            Self::Vehicle => "vehicle",
            Self::Fleet => "fleet",
            Self::Swarm => "swarm",
            Self::Team => "team",
            Self::Device => "device",
            Self::Sensor => "sensor",
            Self::Actuator => "actuator",
            Self::Gateway => "gateway",
            Self::Controller => "controller",
            Self::Wearable => "wearable",
            Self::MedicalDevice => "medical_device",
            Self::Camera => "camera",
            Self::Gps => "gps",
            Self::Plc => "plc",
            Self::AiAgent => "ai_agent",
            Self::CloudService => "cloud_service",
            Self::EdgeService => "edge_service",
            Self::DigitalTwin => "digital_twin",
            Self::Provider => "provider",
            Self::Package => "package",
            Self::Mission => "mission",
            Self::Facility => "facility",
            Self::Building => "building",
            Self::Zone => "zone",
            Self::Hazard => "hazard",
            Self::Incident => "incident",
            Self::CommandCenter => "command_center",
            Self::ControlCenter => "control_center",
            Self::Organization => "organization",
            Self::Compute => "compute",
            Self::ArDevice => "ar_device",
            Self::VrDevice => "vr_device",
            Self::IotDevice => "iot_device",
            Self::SpatialSession => "spatial_session",
            Self::Custom(name) => name.as_str(),
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "human" | "operator" => Self::Human,
            "robot" => Self::Robot,
            "drone" => Self::Drone,
            "vehicle" => Self::Vehicle,
            "fleet" => Self::Fleet,
            "swarm" => Self::Swarm,
            "team" => Self::Team,
            "device" => Self::Device,
            "sensor" => Self::Sensor,
            "actuator" => Self::Actuator,
            "gateway" => Self::Gateway,
            "controller" => Self::Controller,
            "wearable" => Self::Wearable,
            "medical_device" => Self::MedicalDevice,
            "camera" => Self::Camera,
            "gps" => Self::Gps,
            "plc" => Self::Plc,
            "ai_agent" => Self::AiAgent,
            "cloud_service" => Self::CloudService,
            "edge_service" => Self::EdgeService,
            "digital_twin" | "twin" => Self::DigitalTwin,
            "provider" => Self::Provider,
            "package" => Self::Package,
            "mission" => Self::Mission,
            "facility" => Self::Facility,
            "building" => Self::Building,
            "zone" | "hazard_zone" => Self::Zone,
            "hazard" => Self::Hazard,
            "incident" => Self::Incident,
            "command_center" => Self::CommandCenter,
            "control_center" => Self::ControlCenter,
            "organization" | "org" => Self::Organization,
            "compute" => Self::Compute,
            "ar_device" => Self::ArDevice,
            "vr_device" => Self::VrDevice,
            "iot_device" => Self::IotDevice,
            "spatial_session" => Self::SpatialSession,
            other => Self::Custom(other.to_string()),
        }
    }

    pub fn from_device_type(device_type: &str) -> Self {
        match device_type.to_ascii_lowercase().as_str() {
            "human" => Self::Human,
            "sensor" | "lidar" | "imu" | "radar" => Self::Sensor,
            "actuator" | "motor" | "gripper" => Self::Actuator,
            "camera" | "vision" => Self::Camera,
            "gps" | "gnss" => Self::Gps,
            "plc" => Self::Plc,
            "gateway" => Self::Gateway,
            "controller" => Self::Controller,
            "wearable" => Self::Wearable,
            "ardevice" | "ar_device" => Self::ArDevice,
            "vrdevice" | "vr_device" => Self::VrDevice,
            "drone" => Self::Drone,
            "iotdevice" | "iot_device" => Self::IotDevice,
            "controlcenter" | "control_center" => Self::ControlCenter,
            "medical_device" => Self::MedicalDevice,
            _ => Self::Device,
        }
    }
}

/// Platform-wide health posture for any entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EntityHealthStatus {
    Healthy,
    Warning,
    Degraded,
    Offline,
    Critical,
    #[default]
    Unknown,
}

impl EntityHealthStatus {
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "healthy" | "ok" | "active" => Self::Healthy,
            "warning" | "warn" => Self::Warning,
            "degraded" => Self::Degraded,
            "offline" => Self::Offline,
            "critical" | "failed" | "fail" => Self::Critical,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Warning => "warning",
            Self::Degraded => "degraded",
            Self::Offline => "offline",
            Self::Critical => "critical",
            Self::Unknown => "unknown",
        }
    }
}

/// Operational readiness for missions, fleets, operators, and facilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EntityReadinessStatus {
    Ready,
    NotReady,
    Partial,
    #[default]
    Unknown,
}

impl EntityReadinessStatus {
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "ready" | "available" | "active" => Self::Ready,
            "not_ready" | "unavailable" | "blocked" => Self::NotReady,
            "partial" | "degraded" => Self::Partial,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::NotReady => "not_ready",
            Self::Partial => "partial",
            Self::Unknown => "unknown",
        }
    }
}

/// Trust evaluation posture for security and assurance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EntityTrustStatus {
    Verified,
    Trusted,
    Untrusted,
    Compromised,
    #[default]
    Unknown,
}

impl EntityTrustStatus {
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "verified" => Self::Verified,
            "trusted" => Self::Trusted,
            "untrusted" | "unverified" | "restricted" => Self::Untrusted,
            "compromised" | "quarantined" => Self::Compromised,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Trusted => "trusted",
            Self::Untrusted => "untrusted",
            Self::Compromised => "compromised",
            Self::Unknown => "unknown",
        }
    }
}

/// Lifecycle phase from discovery through retirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EntityLifecycleState {
    Discovered,
    Provisioned,
    Verified,
    Assigned,
    Active,
    Suspended,
    Degraded,
    Offline,
    Retired,
    Archived,
    #[default]
    Unknown,
}

impl EntityLifecycleState {
    pub fn from_device_lifecycle(state: DeviceLifecycleState) -> Self {
        match state {
            DeviceLifecycleState::Discovered => Self::Discovered,
            DeviceLifecycleState::Quarantined => Self::Suspended,
            DeviceLifecycleState::Verified => Self::Verified,
            DeviceLifecycleState::Assigned => Self::Assigned,
            DeviceLifecycleState::Active | DeviceLifecycleState::Healthy => Self::Active,
            DeviceLifecycleState::Degraded => Self::Degraded,
            DeviceLifecycleState::Offline => Self::Offline,
            DeviceLifecycleState::Failed => Self::Suspended,
            DeviceLifecycleState::Retired => Self::Retired,
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "discovered" => Self::Discovered,
            "provisioned" => Self::Provisioned,
            "verified" => Self::Verified,
            "assigned" => Self::Assigned,
            "active" | "healthy" => Self::Active,
            "suspended" | "quarantined" => Self::Suspended,
            "degraded" => Self::Degraded,
            "offline" => Self::Offline,
            "retired" => Self::Retired,
            "archived" => Self::Archived,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Discovered => "discovered",
            Self::Provisioned => "provisioned",
            Self::Verified => "verified",
            Self::Assigned => "assigned",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Degraded => "degraded",
            Self::Offline => "offline",
            Self::Retired => "retired",
            Self::Archived => "archived",
            Self::Unknown => "unknown",
        }
    }
}

/// Directed relationship between two entities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityRelationshipKind {
    Owns,
    Contains,
    ConnectedTo,
    Controls,
    Monitors,
    DependsOn,
    AssignedTo,
    CommunicatesWith,
    BacksUp,
    Replaces,
    ReportsTo,
    BelongsTo,
    LocatedIn,
    SecuredBy,
    ManagedBy,
    Provides,
    Consumes,
    ParticipatesIn,
}

impl EntityRelationshipKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Owns => "owns",
            Self::Contains => "contains",
            Self::ConnectedTo => "connected_to",
            Self::Controls => "controls",
            Self::Monitors => "monitors",
            Self::DependsOn => "depends_on",
            Self::AssignedTo => "assigned_to",
            Self::CommunicatesWith => "communicates_with",
            Self::BacksUp => "backs_up",
            Self::Replaces => "replaces",
            Self::ReportsTo => "reports_to",
            Self::BelongsTo => "belongs_to",
            Self::LocatedIn => "located_in",
            Self::SecuredBy => "secured_by",
            Self::ManagedBy => "managed_by",
            Self::Provides => "provides",
            Self::Consumes => "consumes",
            Self::ParticipatesIn => "participates_in",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "owns" => Some(Self::Owns),
            "contains" => Some(Self::Contains),
            "connected_to" => Some(Self::ConnectedTo),
            "controls" => Some(Self::Controls),
            "monitors" => Some(Self::Monitors),
            "depends_on" => Some(Self::DependsOn),
            "assigned_to" => Some(Self::AssignedTo),
            "communicates_with" => Some(Self::CommunicatesWith),
            "backs_up" => Some(Self::BacksUp),
            "replaces" => Some(Self::Replaces),
            "reports_to" => Some(Self::ReportsTo),
            "belongs_to" => Some(Self::BelongsTo),
            "located_in" => Some(Self::LocatedIn),
            "secured_by" => Some(Self::SecuredBy),
            "managed_by" => Some(Self::ManagedBy),
            "provides" => Some(Self::Provides),
            "consumes" => Some(Self::Consumes),
            "participates_in" => Some(Self::ParticipatesIn),
            _ => None,
        }
    }
}

/// Edge in the entity relationship graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityRelationship {
    pub from_id: String,
    pub to_id: String,
    pub kind: EntityRelationshipKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Geographic or logical location attached to an entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EntityLocation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordinates: Option<toml::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone_id: Option<String>,
}

/// Security identity metadata for authentication and authorization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EntitySecurityIdentity {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub certificates: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encryption: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<String>,
}

/// Audit trail pointer for governance and compliance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EntityAuditInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Canonical entity record with shared platform properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityRecord {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub entity_type: EntityKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serial_number: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hardware_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firmware_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub software_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<EntityLocation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
    pub health_status: EntityHealthStatus,
    pub readiness_status: EntityReadinessStatus,
    pub trust_status: EntityTrustStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security: Option<EntitySecurityIdentity>,
    pub lifecycle_state: EntityLifecycleState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit: Option<EntityAuditInfo>,
}

impl EntityRecord {
    /// Backward-compatible kind string for legacy `/v1/entities` consumers.
    pub fn kind(&self) -> &str {
        self.entity_type.as_str()
    }

    pub fn summary_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "kind": self.kind(),
            "entity_type": self.entity_type,
            "display_name": self.display_name,
            "name": self.name,
            "health_status": self.health_status,
            "readiness_status": self.readiness_status,
            "trust_status": self.trust_status,
            "lifecycle_state": self.lifecycle_state,
            "parent_id": self.parent_id,
            "capabilities": self.capabilities,
            "tags": self.tags,
        })
    }
}

/// Entity graph for traversal, dependency analysis, and visualization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EntityGraph {
    pub nodes: Vec<EntityRecord>,
    pub edges: Vec<EntityRelationship>,
}

/// Filter criteria for entity registry queries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EntityQuery {
    #[serde(default)]
    pub entity_type: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub health_status: Option<String>,
    #[serde(default)]
    pub readiness_status: Option<String>,
    #[serde(default)]
    pub trust_status: Option<String>,
    #[serde(default)]
    pub lifecycle_state: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub package: Option<String>,
    #[serde(default)]
    pub firmware_version: Option<String>,
    #[serde(default)]
    pub assigned_to: Option<String>,
    #[serde(default)]
    pub depends_on: Option<String>,
    #[serde(default)]
    pub participates_in: Option<String>,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub search: Option<String>,
}

/// Query result envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityQueryResult {
    pub entities: Vec<EntityRecord>,
    pub count: usize,
    pub query: EntityQuery,
}

/// Unified entity registry backed by resolved system configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EntityRegistry {
    pub entities: HashMap<String, EntityRecord>,
    pub relationships: Vec<EntityRelationship>,
}

impl EntityRegistry {
    pub fn get(&self, id: &str) -> Option<&EntityRecord> {
        self.entities.get(id)
    }

    pub fn list(&self) -> Vec<&EntityRecord> {
        let mut out: Vec<_> = self.entities.values().collect();
        out.sort_by(|a, b| a.id.cmp(&b.id));
        out
    }

    pub fn graph(&self) -> EntityGraph {
        EntityGraph {
            nodes: self.list().into_iter().cloned().collect(),
            edges: self.relationships.clone(),
        }
    }

    pub fn relationships_for(&self, entity_id: &str) -> Vec<&EntityRelationship> {
        self.relationships
            .iter()
            .filter(|r| r.from_id == entity_id || r.to_id == entity_id)
            .collect()
    }

    pub fn query(&self, query: &EntityQuery) -> EntityQueryResult {
        let entities: Vec<EntityRecord> = self
            .list()
            .into_iter()
            .filter(|e| matches_query(e, query, self))
            .cloned()
            .collect();
        EntityQueryResult {
            count: entities.len(),
            entities,
            query: query.clone(),
        }
    }

    pub fn impact_analysis(&self, entity_id: &str) -> Vec<String> {
        let mut affected = HashSet::new();
        let mut queue = VecDeque::from([entity_id.to_string()]);
        while let Some(current) = queue.pop_front() {
            for edge in &self.relationships {
                if edge.from_id == current && affected.insert(edge.to_id.clone()) {
                    queue.push_back(edge.to_id.clone());
                }
                if edge.to_id == current
                    && matches!(
                        edge.kind,
                        EntityRelationshipKind::DependsOn
                            | EntityRelationshipKind::Consumes
                            | EntityRelationshipKind::AssignedTo
                            | EntityRelationshipKind::BelongsTo
                    )
                    && affected.insert(edge.from_id.clone())
                {
                    queue.push_back(edge.from_id.clone());
                }
            }
        }
        affected.remove(entity_id);
        let mut out: Vec<_> = affected.into_iter().collect();
        out.sort();
        out
    }

    pub fn dependency_chain(&self, entity_id: &str) -> Vec<String> {
        let mut chain = Vec::new();
        let mut current = entity_id.to_string();
        let mut visited = HashSet::new();
        loop {
            let Some(dep) = self.relationships.iter().find(|r| {
                r.from_id == current
                    && matches!(
                        r.kind,
                        EntityRelationshipKind::DependsOn | EntityRelationshipKind::Consumes
                    )
            }) else {
                break;
            };
            if !visited.insert(dep.to_id.clone()) {
                break;
            }
            chain.push(dep.to_id.clone());
            current = dep.to_id.clone();
        }
        chain
    }

    /// Mission entities linked via `participates_in` from a robot, operator, or fleet member.
    pub fn linked_missions(&self, participant_id: &str) -> Vec<&EntityRecord> {
        let mut ids = Vec::new();
        for edge in &self.relationships {
            if edge.from_id == participant_id && edge.kind == EntityRelationshipKind::ParticipatesIn
            {
                ids.push(edge.to_id.as_str());
            }
        }
        ids.sort_unstable();
        ids.dedup();
        ids.iter()
            .filter_map(|id| self.get(id))
            .filter(|entity| entity.entity_type == EntityKind::Mission)
            .collect()
    }
}

/// Build a unified entity registry from resolved configuration.
pub fn build_entity_registry(resolved: &ResolvedSystemConfig) -> EntityRegistry {
    let mut registry = EntityRegistry::default();
    let project_name = resolved.project_name().to_string();

    if let Some(org) = resolved.manifest.project.as_ref() {
        let org_id = format!("org:{}", org.name);
        registry.entities.insert(
            org_id.clone(),
            EntityRecord {
                id: org_id.clone(),
                name: Some(org.name.clone()),
                display_name: Some(org.name.clone()),
                entity_type: EntityKind::Organization,
                version: Some(org.version.clone()),
                lifecycle_state: EntityLifecycleState::Active,
                health_status: EntityHealthStatus::Healthy,
                readiness_status: EntityReadinessStatus::Ready,
                trust_status: EntityTrustStatus::Trusted,
                metadata: HashMap::from([("project".into(), project_name)]),
                ..Default::default()
            },
        );
    }

    ingest_device_tree(
        &mut registry,
        &resolved.device_tree,
        &resolved.human_registry,
    );
    ingest_device_registry(&mut registry, &resolved.device_registry);
    ingest_human_registry(&mut registry, &resolved.human_registry);
    ingest_logical_map(&mut registry, &resolved.logical_map);
    ingest_packages_and_providers(&mut registry, &resolved.packages, &resolved.providers);
    ingest_facilities(
        &mut registry,
        &FacilityRegistry::from_raw(&resolved.raw),
        resolved.fleet_id(),
    );
    link_zone_references(&mut registry);
    apply_compliance_metadata(&mut registry, resolved);

    if let Some(fleet_id) = resolved.fleet_id() {
        if let Some(org_id) = registry
            .entities
            .keys()
            .find(|k| k.starts_with("org:"))
            .cloned()
        {
            link(
                &mut registry,
                &org_id,
                fleet_id,
                EntityRelationshipKind::Owns,
                None,
            );
        }
    }

    registry
}

fn matches_query(entity: &EntityRecord, query: &EntityQuery, registry: &EntityRegistry) -> bool {
    let type_filter = query.entity_type.as_deref().or(query.kind.as_deref());
    if let Some(kind) = type_filter {
        if entity.kind() != kind && entity.entity_type.as_str() != kind {
            return false;
        }
    }
    if let Some(health) = &query.health_status {
        if entity.health_status.as_str() != health.to_ascii_lowercase().as_str() {
            return false;
        }
    }
    if let Some(readiness) = &query.readiness_status {
        if entity.readiness_status.as_str() != readiness.to_ascii_lowercase().as_str() {
            return false;
        }
    }
    if let Some(trust) = &query.trust_status {
        if entity.trust_status.as_str() != trust.to_ascii_lowercase().as_str() {
            return false;
        }
    }
    if let Some(lifecycle) = &query.lifecycle_state {
        if entity.lifecycle_state.as_str() != lifecycle.to_ascii_lowercase().as_str() {
            return false;
        }
    }
    if let Some(tag) = &query.tag {
        if !entity.tags.iter().any(|t| t == tag) {
            return false;
        }
    }
    if let Some(label) = &query.label {
        if !entity.labels.iter().any(|l| l == label) {
            return false;
        }
    }
    if let Some(provider) = &query.provider {
        if entity.provider.as_deref() != Some(provider.as_str()) {
            return false;
        }
    }
    if let Some(package) = &query.package {
        if entity.package.as_deref() != Some(package.as_str()) {
            return false;
        }
    }
    if let Some(firmware) = &query.firmware_version {
        if entity.firmware_version.as_deref() != Some(firmware.as_str()) {
            return false;
        }
    }
    if let Some(parent) = &query.parent_id {
        if entity.parent_id.as_deref() != Some(parent.as_str()) {
            return false;
        }
    }
    if let Some(assigned) = &query.assigned_to {
        let assigned_ok = registry.relationships.iter().any(|r| {
            r.to_id == *assigned
                && r.from_id == entity.id
                && matches!(
                    r.kind,
                    EntityRelationshipKind::AssignedTo | EntityRelationshipKind::BelongsTo
                )
        });
        if !assigned_ok {
            return false;
        }
    }
    if let Some(dep) = &query.depends_on {
        let depends_ok = registry.relationships.iter().any(|r| {
            r.to_id == *dep
                && r.from_id == entity.id
                && matches!(
                    r.kind,
                    EntityRelationshipKind::DependsOn | EntityRelationshipKind::Consumes
                )
        });
        if !depends_ok {
            return false;
        }
    }
    if let Some(mission_id) = &query.participates_in {
        let participates_ok = registry.relationships.iter().any(|r| {
            r.from_id == entity.id
                && r.to_id == *mission_id
                && r.kind == EntityRelationshipKind::ParticipatesIn
        });
        if !participates_ok {
            return false;
        }
    }
    if let Some(search) = &query.search {
        let needle = search.to_ascii_lowercase();
        let haystack = format!(
            "{} {} {} {:?}",
            entity.id,
            entity.display_name.as_deref().unwrap_or(""),
            entity.name.as_deref().unwrap_or(""),
            entity.entity_type
        )
        .to_ascii_lowercase();
        if !haystack.contains(&needle) {
            return false;
        }
    }
    true
}

fn link(
    registry: &mut EntityRegistry,
    from: &str,
    to: &str,
    kind: EntityRelationshipKind,
    label: Option<&str>,
) {
    registry.relationships.push(EntityRelationship {
        from_id: from.to_string(),
        to_id: to.to_string(),
        kind,
        label: label.map(String::from),
    });
    if let Some(parent) = registry.entities.get_mut(from) {
        if !parent.children_ids.contains(&to.to_string()) {
            parent.children_ids.push(to.to_string());
        }
    }
    if let Some(child) = registry.entities.get_mut(to) {
        child.parent_id = Some(from.to_string());
    }
}

fn ingest_device_tree(
    registry: &mut EntityRegistry,
    tree: &DeviceTree,
    human_registry: &HumanRegistry,
) {
    let Some(fleet) = tree.fleet.as_ref() else {
        return;
    };
    upsert_fleet(registry, fleet);
    for robot in &fleet.robots {
        upsert_robot(registry, robot, &fleet.id);
        if let Some(compute) = robot.compute.as_ref() {
            upsert_compute(registry, compute, &robot.id);
            for device in &compute.devices {
                upsert_device_node(registry, device, &compute.id, Some(&robot.id));
            }
        }
    }
    for human in &fleet.humans {
        upsert_human(registry, human, Some(&fleet.id));
    }
    for wearable in &fleet.wearables {
        upsert_wearable(registry, wearable, Some(&fleet.id));
    }
    for ar in &fleet.ar_devices {
        upsert_spatial(
            registry,
            ar.id.as_str(),
            EntityKind::ArDevice,
            ar,
            Some(&fleet.id),
        );
    }
    for vr in &fleet.vr_devices {
        upsert_spatial(
            registry,
            vr.id.as_str(),
            EntityKind::VrDevice,
            vr,
            Some(&fleet.id),
        );
    }
    for drone in &fleet.drones {
        upsert_spatial(
            registry,
            drone.id.as_str(),
            EntityKind::Drone,
            drone,
            Some(&fleet.id),
        );
    }
    for iot in &fleet.iot_devices {
        upsert_spatial(
            registry,
            iot.id.as_str(),
            EntityKind::IotDevice,
            iot,
            Some(&fleet.id),
        );
    }
    for cc in &fleet.control_center {
        upsert_control_center(registry, cc, &fleet.id);
    }
    for zone in &fleet.hazard_zones {
        upsert_hazard_zone(registry, zone, Some(&fleet.id));
    }
    for twin in &human_registry.twins {
        upsert_twin(registry, twin);
    }
}

fn upsert_fleet(registry: &mut EntityRegistry, fleet: &FleetNode) {
    registry.entities.insert(
        fleet.id.clone(),
        EntityRecord {
            id: fleet.id.clone(),
            name: Some(fleet.id.clone()),
            display_name: Some(fleet.id.clone()),
            entity_type: EntityKind::Fleet,
            lifecycle_state: EntityLifecycleState::Active,
            health_status: EntityHealthStatus::Healthy,
            readiness_status: EntityReadinessStatus::Ready,
            trust_status: EntityTrustStatus::Trusted,
            tags: vec!["fleet".into()],
            ..Default::default()
        },
    );
}

fn robot_entity_kind(robot: &RobotNode) -> EntityKind {
    if let Some(kind) = robot.entity_kind.as_deref() {
        return EntityKind::parse(kind);
    }
    if robot
        .model
        .as_deref()
        .is_some_and(|model| is_automotive_model(model))
    {
        return EntityKind::Vehicle;
    }
    EntityKind::Robot
}

fn is_automotive_model(model: &str) -> bool {
    matches!(
        model.to_ascii_lowercase().as_str(),
        "passengersedan" | "passenger_sedan" | "vehicle" | "automotive" | "adas_platform"
    )
}

fn upsert_robot(registry: &mut EntityRegistry, robot: &RobotNode, fleet_id: &str) {
    let entity_type = robot_entity_kind(robot);
    let kind_tag = entity_type.as_str().to_string();
    let mut metadata = HashMap::new();
    if let Some(profile) = robot.compliance_profile.as_ref() {
        metadata.insert("compliance.profile".into(), profile.clone());
    }
    registry.entities.insert(
        robot.id.clone(),
        EntityRecord {
            id: robot.id.clone(),
            name: Some(robot.id.clone()),
            display_name: Some(robot.id.clone()),
            entity_type,
            model: robot.model.clone(),
            parent_id: Some(fleet_id.to_string()),
            lifecycle_state: EntityLifecycleState::Active,
            health_status: EntityHealthStatus::Healthy,
            readiness_status: EntityReadinessStatus::Ready,
            trust_status: EntityTrustStatus::Trusted,
            tags: vec![kind_tag],
            metadata,
            ..Default::default()
        },
    );
    link(
        registry,
        fleet_id,
        &robot.id,
        EntityRelationshipKind::Contains,
        None,
    );
}

fn upsert_compute(registry: &mut EntityRegistry, compute: &ComputeNode, robot_id: &str) {
    registry.entities.insert(
        compute.id.clone(),
        EntityRecord {
            id: compute.id.clone(),
            name: Some(compute.id.clone()),
            display_name: Some(compute.id.clone()),
            entity_type: EntityKind::Compute,
            serial_number: compute.serial.clone(),
            parent_id: Some(robot_id.to_string()),
            lifecycle_state: EntityLifecycleState::Active,
            health_status: EntityHealthStatus::Healthy,
            readiness_status: EntityReadinessStatus::Ready,
            trust_status: EntityTrustStatus::Trusted,
            metadata: HashMap::from([("compute_type".into(), compute.compute_type.clone())]),
            ..Default::default()
        },
    );
    link(
        registry,
        robot_id,
        &compute.id,
        EntityRelationshipKind::Contains,
        None,
    );
}

fn upsert_device_node(
    registry: &mut EntityRegistry,
    device: &DeviceNode,
    parent_id: &str,
    robot_id: Option<&str>,
) {
    let entity_type = device
        .entity_kind
        .as_deref()
        .map(EntityKind::parse)
        .unwrap_or_else(|| EntityKind::from_device_type(&device.device_type));
    let trust = device
        .trust_level
        .as_deref()
        .map(EntityTrustStatus::parse)
        .unwrap_or_default();
    registry.entities.insert(
        device.id.clone(),
        EntityRecord {
            id: device.id.clone(),
            name: device
                .logical_name
                .clone()
                .or_else(|| Some(device.id.clone())),
            display_name: device.logical_name.clone(),
            entity_type,
            parent_id: Some(parent_id.to_string()),
            provider: device.provider.clone(),
            firmware_version: device
                .firmware_version
                .clone()
                .or_else(|| device.firmware.clone()),
            hardware_revision: device.hardware_revision.clone(),
            software_version: device.version.clone(),
            serial_number: device.serial.clone(),
            capabilities: device.capabilities.clone(),
            lifecycle_state: EntityLifecycleState::Active,
            health_status: EntityHealthStatus::Healthy,
            readiness_status: EntityReadinessStatus::Ready,
            trust_status: trust,
            security: Some(EntitySecurityIdentity {
                identity: device.security_identity.clone().or(device.identity.clone()),
                certificates: device
                    .certificate_fingerprint
                    .clone()
                    .map(|c| vec![c])
                    .unwrap_or_default(),
                ..Default::default()
            }),
            tags: vec![device.device_type.clone()],
            metadata: device
                .compliance_profile
                .as_ref()
                .map(|profile| HashMap::from([("compliance.profile".into(), profile.clone())]))
                .unwrap_or_default(),
            ..Default::default()
        },
    );
    link(
        registry,
        parent_id,
        &device.id,
        EntityRelationshipKind::Contains,
        None,
    );
    if let Some(provider) = device.provider.as_ref() {
        link(
            registry,
            &device.id,
            provider,
            EntityRelationshipKind::DependsOn,
            Some("provider"),
        );
    }
    if let Some(robot) = robot_id {
        link(
            registry,
            robot,
            &device.id,
            EntityRelationshipKind::Monitors,
            None,
        );
    }
}

fn ingest_device_registry(registry: &mut EntityRegistry, device_registry: &DeviceRegistry) {
    for record in &device_registry.devices {
        upsert_device_record(registry, record);
    }
}

fn upsert_device_record(registry: &mut EntityRegistry, record: &DeviceIdentityRecord) {
    let lifecycle = record
        .lifecycle_state
        .as_deref()
        .map(DeviceLifecycleState::parse)
        .map(EntityLifecycleState::from_device_lifecycle)
        .unwrap_or_default();
    let health = record
        .health_status
        .as_deref()
        .map(EntityHealthStatus::parse)
        .unwrap_or_default();
    let trust = record
        .trust_level
        .as_deref()
        .map(EntityTrustStatus::parse)
        .unwrap_or_default();
    let entity_type = EntityKind::from_device_type(&record.device_type);
    let entry = EntityRecord {
        id: record.id.clone(),
        name: record
            .logical_name
            .clone()
            .or_else(|| Some(record.id.clone())),
        display_name: record.logical_name.clone(),
        entity_type,
        provider: record.provider.clone(),
        firmware_version: record.firmware_version.clone(),
        hardware_revision: record.hardware_revision.clone(),
        serial_number: record.serial.clone(),
        capabilities: record.capabilities.clone(),
        lifecycle_state: lifecycle,
        health_status: health,
        readiness_status: readiness_from_lifecycle(lifecycle),
        trust_status: trust,
        tags: vec![record.device_type.clone()],
        ..Default::default()
    };
    registry.entities.insert(record.id.clone(), entry);
    if let Some(robot) = record.assigned_robot.as_ref().or(record.robot_id.as_ref()) {
        if registry.entities.contains_key(robot) {
            link(
                registry,
                robot,
                &record.id,
                EntityRelationshipKind::Contains,
                Some("assigned"),
            );
            link(
                registry,
                &record.id,
                robot,
                EntityRelationshipKind::AssignedTo,
                None,
            );
        }
    }
    if let Some(provider) = record.provider.as_ref() {
        link(
            registry,
            &record.id,
            provider,
            EntityRelationshipKind::DependsOn,
            Some("provider"),
        );
    }
}

fn ingest_human_registry(registry: &mut EntityRegistry, human_registry: &HumanRegistry) {
    for human in &human_registry.humans {
        upsert_human(registry, human, None);
    }
    for wearable in &human_registry.wearables {
        upsert_wearable(registry, wearable, None);
    }
    for ar in &human_registry.ar_devices {
        upsert_spatial(registry, ar.id.as_str(), EntityKind::ArDevice, ar, None);
    }
    for vr in &human_registry.vr_devices {
        upsert_spatial(registry, vr.id.as_str(), EntityKind::VrDevice, vr, None);
    }
    for drone in &human_registry.drones {
        upsert_spatial(registry, drone.id.as_str(), EntityKind::Drone, drone, None);
    }
    for iot in &human_registry.iot_devices {
        upsert_spatial(registry, iot.id.as_str(), EntityKind::IotDevice, iot, None);
    }
    for twin in &human_registry.twins {
        upsert_twin(registry, twin);
    }
    for zone in &human_registry.hazard_zones {
        upsert_hazard_zone(registry, zone, None);
    }
    for session in &human_registry.spatial_sessions {
        upsert_spatial_session(registry, session, None);
    }
}

fn upsert_human(registry: &mut EntityRegistry, human: &HumanEntity, fleet_id: Option<&str>) {
    let readiness = human
        .availability
        .as_deref()
        .map(EntityReadinessStatus::parse)
        .unwrap_or_default();
    let health = human
        .health_status
        .as_deref()
        .map(EntityHealthStatus::parse)
        .unwrap_or_default();
    let trust = human
        .trust_level
        .as_deref()
        .map(EntityTrustStatus::parse)
        .unwrap_or_default();
    registry.entities.insert(
        human.id.clone(),
        EntityRecord {
            id: human.id.clone(),
            name: Some(human.id.clone()),
            display_name: human
                .display_name
                .clone()
                .or_else(|| Some(human.id.clone())),
            entity_type: EntityKind::Human,
            parent_id: fleet_id.map(String::from),
            capabilities: human.capabilities.clone(),
            lifecycle_state: EntityLifecycleState::Active,
            health_status: health,
            readiness_status: readiness,
            trust_status: trust,
            location: human.location.as_ref().map(|loc| EntityLocation {
                coordinates: Some(loc.clone()),
                zone_id: loc
                    .get("zone")
                    .or_else(|| loc.get("zone_id"))
                    .and_then(|value| value.as_str())
                    .map(String::from),
                ..Default::default()
            }),
            security: Some(EntitySecurityIdentity {
                permissions: human.permissions.clone(),
                ..Default::default()
            }),
            metadata: HashMap::from([("role".into(), human.role.clone())]),
            tags: vec!["human".into(), human.role.clone()],
            ..Default::default()
        },
    );
    if let Some(fleet) = fleet_id {
        link(
            registry,
            fleet,
            &human.id,
            EntityRelationshipKind::Contains,
            None,
        );
    }
}

fn upsert_wearable(
    registry: &mut EntityRegistry,
    wearable: &WearableEntity,
    fleet_id: Option<&str>,
) {
    let trust = wearable
        .trust_level
        .as_deref()
        .map(EntityTrustStatus::parse)
        .unwrap_or_default();
    registry.entities.insert(
        wearable.id.clone(),
        EntityRecord {
            id: wearable.id.clone(),
            name: Some(wearable.id.clone()),
            display_name: Some(wearable.id.clone()),
            entity_type: EntityKind::Wearable,
            parent_id: fleet_id.map(String::from),
            provider: wearable.provider.clone(),
            capabilities: wearable.capabilities.clone(),
            lifecycle_state: EntityLifecycleState::Active,
            health_status: EntityHealthStatus::Healthy,
            readiness_status: EntityReadinessStatus::Ready,
            trust_status: trust,
            tags: vec![wearable.device_type.clone()],
            ..Default::default()
        },
    );
    if let Some(fleet) = fleet_id {
        link(
            registry,
            fleet,
            &wearable.id,
            EntityRelationshipKind::Contains,
            None,
        );
    }
    if let Some(human_id) = wearable.human_id.as_ref() {
        link(
            registry,
            &wearable.id,
            human_id,
            EntityRelationshipKind::AssignedTo,
            None,
        );
    }
    if let Some(provider) = wearable.provider.as_ref() {
        link(
            registry,
            &wearable.id,
            provider,
            EntityRelationshipKind::DependsOn,
            Some("provider"),
        );
    }
}

fn upsert_spatial(
    registry: &mut EntityRegistry,
    id: &str,
    kind: EntityKind,
    spatial: &crate::human_entities::SpatialDeviceEntity,
    fleet_id: Option<&str>,
) {
    let trust = spatial
        .trust_level
        .as_deref()
        .map(EntityTrustStatus::parse)
        .unwrap_or_default();
    registry.entities.insert(
        id.to_string(),
        EntityRecord {
            id: id.to_string(),
            name: Some(id.to_string()),
            display_name: Some(id.to_string()),
            entity_type: kind,
            parent_id: fleet_id.map(String::from),
            provider: spatial.provider.clone(),
            capabilities: spatial.capabilities.clone(),
            lifecycle_state: EntityLifecycleState::Active,
            health_status: EntityHealthStatus::Healthy,
            readiness_status: EntityReadinessStatus::Ready,
            trust_status: trust,
            tags: vec![spatial.device_type.clone()],
            ..Default::default()
        },
    );
    if let Some(fleet) = fleet_id {
        link(registry, fleet, id, EntityRelationshipKind::Contains, None);
    }
    if let Some(human_id) = spatial.human_id.as_ref() {
        link(
            registry,
            id,
            human_id,
            EntityRelationshipKind::AssignedTo,
            None,
        );
    }
}

fn upsert_control_center(
    registry: &mut EntityRegistry,
    cc: &crate::human_entities::ControlCenterEntity,
    fleet_id: &str,
) {
    registry.entities.insert(
        cc.id.clone(),
        EntityRecord {
            id: cc.id.clone(),
            name: Some(cc.id.clone()),
            display_name: Some(cc.id.clone()),
            entity_type: EntityKind::ControlCenter,
            parent_id: Some(fleet_id.to_string()),
            capabilities: cc.capabilities.clone(),
            lifecycle_state: EntityLifecycleState::Active,
            health_status: EntityHealthStatus::Healthy,
            readiness_status: EntityReadinessStatus::Ready,
            trust_status: EntityTrustStatus::Trusted,
            ..Default::default()
        },
    );
    link(
        registry,
        fleet_id,
        &cc.id,
        EntityRelationshipKind::Contains,
        None,
    );
}

fn upsert_hazard_zone(
    registry: &mut EntityRegistry,
    zone: &HazardZoneEntity,
    fleet_id: Option<&str>,
) {
    registry.entities.insert(
        zone.id.clone(),
        EntityRecord {
            id: zone.id.clone(),
            name: Some(zone.id.clone()),
            display_name: Some(zone.id.clone()),
            entity_type: EntityKind::Hazard,
            parent_id: fleet_id.map(String::from),
            location: zone.center.as_ref().map(|center| EntityLocation {
                coordinates: Some(center.clone()),
                zone_id: Some(zone.id.clone()),
                ..Default::default()
            }),
            lifecycle_state: EntityLifecycleState::Active,
            health_status: EntityHealthStatus::Warning,
            readiness_status: EntityReadinessStatus::Partial,
            trust_status: EntityTrustStatus::Trusted,
            metadata: HashMap::from([
                (
                    "severity".into(),
                    zone.severity.clone().unwrap_or_else(|| "unknown".into()),
                ),
                (
                    "zone_type".into(),
                    zone.zone_type.clone().unwrap_or_else(|| "hazard".into()),
                ),
            ]),
            ..Default::default()
        },
    );
    if let Some(fleet) = fleet_id {
        link(
            registry,
            fleet,
            &zone.id,
            EntityRelationshipKind::Contains,
            None,
        );
    }
    for robot_id in &zone.linked_robots {
        link(
            registry,
            &zone.id,
            robot_id,
            EntityRelationshipKind::Monitors,
            Some("linked_robot"),
        );
    }
}

fn upsert_twin(registry: &mut EntityRegistry, twin: &TwinEntity) {
    registry.entities.insert(
        twin.id.clone(),
        EntityRecord {
            id: twin.id.clone(),
            name: Some(twin.id.clone()),
            display_name: Some(twin.id.clone()),
            entity_type: EntityKind::DigitalTwin,
            lifecycle_state: EntityLifecycleState::Active,
            health_status: EntityHealthStatus::Healthy,
            readiness_status: EntityReadinessStatus::Ready,
            trust_status: EntityTrustStatus::Trusted,
            ..Default::default()
        },
    );
    link(
        registry,
        &twin.id,
        &twin.entity_id,
        EntityRelationshipKind::Monitors,
        Some("mirrors"),
    );
}

fn ingest_logical_map(registry: &mut EntityRegistry, map: &LogicalPhysicalMap) {
    for robot in map.robots.values() {
        if registry.entities.contains_key(&robot.physical_robot_id) {
            link(
                registry,
                &robot.logical_id,
                &robot.physical_robot_id,
                EntityRelationshipKind::ConnectedTo,
                Some("logical_map"),
            );
        }
    }
    for sensor in map.sensors.values() {
        if registry.entities.contains_key(&sensor.physical_device_id) {
            link(
                registry,
                &sensor.robot_id,
                &sensor.physical_device_id,
                EntityRelationshipKind::ConnectedTo,
                Some("sensor_map"),
            );
        }
    }
    for actuator in map.actuators.values() {
        if registry.entities.contains_key(&actuator.physical_device_id) {
            link(
                registry,
                &actuator.robot_id,
                &actuator.physical_device_id,
                EntityRelationshipKind::Controls,
                Some("actuator_map"),
            );
        }
    }
}

fn ingest_packages_and_providers(
    registry: &mut EntityRegistry,
    packages: &[String],
    providers: &[String],
) {
    for package in packages {
        registry.entities.insert(
            package.clone(),
            EntityRecord {
                id: package.clone(),
                name: Some(package.clone()),
                display_name: Some(package.clone()),
                entity_type: EntityKind::Package,
                lifecycle_state: EntityLifecycleState::Active,
                health_status: EntityHealthStatus::Healthy,
                readiness_status: EntityReadinessStatus::Ready,
                trust_status: EntityTrustStatus::Verified,
                capabilities: vec!["install".into(), "update".into(), "validate".into()],
                tags: vec!["package".into()],
                ..Default::default()
            },
        );
    }
    for provider in providers {
        registry.entities.insert(
            provider.clone(),
            EntityRecord {
                id: provider.clone(),
                name: Some(provider.clone()),
                display_name: Some(provider.clone()),
                entity_type: EntityKind::Provider,
                lifecycle_state: EntityLifecycleState::Active,
                health_status: EntityHealthStatus::Healthy,
                readiness_status: EntityReadinessStatus::Ready,
                trust_status: EntityTrustStatus::Verified,
                tags: vec!["provider".into()],
                ..Default::default()
            },
        );
    }
}

fn readiness_from_lifecycle(lifecycle: EntityLifecycleState) -> EntityReadinessStatus {
    match lifecycle {
        EntityLifecycleState::Active | EntityLifecycleState::Assigned => {
            EntityReadinessStatus::Ready
        }
        EntityLifecycleState::Degraded | EntityLifecycleState::Suspended => {
            EntityReadinessStatus::Partial
        }
        EntityLifecycleState::Offline
        | EntityLifecycleState::Retired
        | EntityLifecycleState::Archived => EntityReadinessStatus::NotReady,
        _ => EntityReadinessStatus::Unknown,
    }
}

/// Runtime mission snapshot merged into the entity registry during active programs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeMissionEntity {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub robot_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fleet_id: Option<String>,
    pub mission_state: String,
    #[serde(default)]
    pub step_index: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_step: Option<String>,
    #[serde(default)]
    pub steps: Vec<String>,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    #[serde(default)]
    pub approval_pending: bool,
}

/// Stable mission entity id for a robot-bound mission declaration.
pub fn mission_entity_id(robot_id: &str, mission_name: &str) -> String {
    format!("mission:{robot_id}:{mission_name}")
}

/// Build pending mission entities from declarative approval seeds in TOML.
pub fn runtime_missions_from_approval_seeds(
    seeds: &[crate::mission_approval::MissionApprovalSeed],
) -> Vec<RuntimeMissionEntity> {
    seeds
        .iter()
        .map(|seed| {
            let pending = seed
                .status
                .as_deref()
                .map(|s| s.eq_ignore_ascii_case("approved"))
                .unwrap_or(false)
                == false;
            RuntimeMissionEntity {
                id: format!("mission:{}", seed.mission_id),
                name: seed.mission_id.clone(),
                robot_id: None,
                fleet_id: None,
                mission_state: if pending {
                    "PendingApproval".into()
                } else {
                    "Pending".into()
                },
                step_index: 0,
                current_step: None,
                steps: Vec::new(),
                required_capabilities: seed
                    .requires_capability
                    .clone()
                    .map(|cap| vec![cap])
                    .unwrap_or_default(),
                approval_pending: pending,
            }
        })
        .collect()
}

/// Merge runtime mission entities into a configuration-backed registry overlay.
pub fn apply_runtime_mission_overlay(
    registry: &mut EntityRegistry,
    missions: &[RuntimeMissionEntity],
) {
    for mission in missions {
        let (health, readiness, lifecycle) = mission_status_to_entity_state(&mission.mission_state);
        let mut capabilities = vec!["pause".into(), "resume".into(), "cancel".into()];
        capabilities.extend(mission.required_capabilities.clone());
        if mission.approval_pending {
            capabilities.push("awaiting_approval".into());
        }
        let readiness = if mission.approval_pending {
            EntityReadinessStatus::NotReady
        } else {
            readiness
        };
        registry.entities.insert(
            mission.id.clone(),
            EntityRecord {
                id: mission.id.clone(),
                name: Some(mission.name.clone()),
                display_name: Some(mission.name.clone()),
                entity_type: EntityKind::Mission,
                parent_id: mission.fleet_id.clone(),
                capabilities,
                health_status: health,
                readiness_status: readiness,
                trust_status: EntityTrustStatus::Trusted,
                lifecycle_state: lifecycle,
                metadata: HashMap::from([
                    ("mission_state".into(), mission.mission_state.clone()),
                    (
                        "current_step".into(),
                        mission.current_step.clone().unwrap_or_default(),
                    ),
                    ("step_index".into(), mission.step_index.to_string()),
                    (
                        "approval_pending".into(),
                        mission.approval_pending.to_string(),
                    ),
                ]),
                tags: vec!["mission".into()],
                ..Default::default()
            },
        );
        if let Some(robot_id) = mission.robot_id.as_ref() {
            if registry.entities.contains_key(robot_id) {
                link(
                    registry,
                    robot_id,
                    &mission.id,
                    EntityRelationshipKind::ParticipatesIn,
                    None,
                );
            }
        }
        if let Some(fleet_id) = mission.fleet_id.as_ref() {
            if registry.entities.contains_key(fleet_id) {
                link(
                    registry,
                    fleet_id,
                    &mission.id,
                    EntityRelationshipKind::Contains,
                    Some("mission"),
                );
            }
        }
    }
}

fn mission_status_to_entity_state(
    mission_state: &str,
) -> (
    EntityHealthStatus,
    EntityReadinessStatus,
    EntityLifecycleState,
) {
    match mission_state.to_ascii_lowercase().as_str() {
        "running" => (
            EntityHealthStatus::Healthy,
            EntityReadinessStatus::Ready,
            EntityLifecycleState::Active,
        ),
        "paused" => (
            EntityHealthStatus::Warning,
            EntityReadinessStatus::Partial,
            EntityLifecycleState::Suspended,
        ),
        "completed" => (
            EntityHealthStatus::Healthy,
            EntityReadinessStatus::Ready,
            EntityLifecycleState::Archived,
        ),
        "failed" => (
            EntityHealthStatus::Critical,
            EntityReadinessStatus::NotReady,
            EntityLifecycleState::Degraded,
        ),
        "pendingapproval" | "pending_approval" => (
            EntityHealthStatus::Warning,
            EntityReadinessStatus::NotReady,
            EntityLifecycleState::Assigned,
        ),
        "pending" => (
            EntityHealthStatus::Healthy,
            EntityReadinessStatus::Partial,
            EntityLifecycleState::Assigned,
        ),
        _ => (
            EntityHealthStatus::Unknown,
            EntityReadinessStatus::Unknown,
            EntityLifecycleState::Unknown,
        ),
    }
}

/// Digital-thread device link projected into the entity relationship store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DigitalThreadTraceabilityLink {
    pub device_id: String,
    pub capability: String,
    pub assigned_robot: Option<String>,
}

/// Program dependency graph edge projected to entity ids.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgramGraphTraceabilityEdge {
    pub from_entity_id: String,
    pub to_entity_id: String,
    pub relation: String,
}

/// Merge digital-thread and program-graph traceability into the entity registry.
pub fn apply_traceability_overlay(
    registry: &mut EntityRegistry,
    device_links: &[DigitalThreadTraceabilityLink],
    program_edges: &[ProgramGraphTraceabilityEdge],
) {
    for link in device_links {
        if !registry.entities.contains_key(&link.device_id) {
            continue;
        }
        let Some(robot_id) = link.assigned_robot.as_ref() else {
            continue;
        };
        if !registry.entities.contains_key(robot_id) {
            continue;
        }
        link_if_absent(
            registry,
            robot_id,
            &link.device_id,
            EntityRelationshipKind::DependsOn,
            Some(&format!("traceability:{}", link.capability)),
        );
    }
    for edge in program_edges {
        if !registry.entities.contains_key(&edge.from_entity_id)
            || !registry.entities.contains_key(&edge.to_entity_id)
        {
            continue;
        }
        let kind = program_relation_to_entity_kind(&edge.relation);
        link_if_absent(
            registry,
            &edge.from_entity_id,
            &edge.to_entity_id,
            kind,
            Some(&edge.relation),
        );
    }
}

fn program_relation_to_entity_kind(relation: &str) -> EntityRelationshipKind {
    match relation {
        "runs" => EntityRelationshipKind::ParticipatesIn,
        "requires" | "requires_hardware" | "requires_package" | "requires_provider"
        | "requires_component" | "uses_hardware" | "uses_provider" | "deploy_to" => {
            EntityRelationshipKind::DependsOn
        }
        "exposes" | "provided_by" => EntityRelationshipKind::Provides,
        "has_sensor" | "has_actuator" | "protected_by" | "kill_switch" | "device_type" => {
            EntityRelationshipKind::Contains
        }
        _ => EntityRelationshipKind::DependsOn,
    }
}

fn link_if_absent(
    registry: &mut EntityRegistry,
    from: &str,
    to: &str,
    kind: EntityRelationshipKind,
    label: Option<&str>,
) {
    let label_string = label.map(String::from);
    if registry.relationships.iter().any(|edge| {
        edge.from_id == from && edge.to_id == to && edge.kind == kind && edge.label == label_string
    }) {
        return;
    }
    link(registry, from, to, kind, label);
}

fn ingest_facilities(
    registry: &mut EntityRegistry,
    facilities: &FacilityRegistry,
    fleet_id: Option<&str>,
) {
    for facility in &facilities.facilities {
        upsert_facility(registry, facility, fleet_id);
    }
    for building in &facilities.buildings {
        upsert_building(registry, building);
    }
    for zone in &facilities.zones {
        upsert_operational_zone(registry, zone);
    }
    for declared in &facilities.entity_kinds {
        upsert_declared_entity_kind(registry, declared, fleet_id);
    }
}

fn upsert_facility(
    registry: &mut EntityRegistry,
    facility: &FacilityEntity,
    fleet_id: Option<&str>,
) {
    let mut metadata = HashMap::new();
    if let Some(profile) = facility.compliance_profile.as_ref() {
        metadata.insert("compliance.profile".into(), profile.clone());
    }
    if let Some(facility_type) = facility.facility_type.as_ref() {
        metadata.insert("facility_type".into(), facility_type.clone());
    }
    registry.entities.insert(
        facility.id.clone(),
        EntityRecord {
            id: facility.id.clone(),
            name: facility.name.clone().or_else(|| Some(facility.id.clone())),
            display_name: facility.name.clone(),
            entity_type: EntityKind::Facility,
            parent_id: fleet_id.map(String::from),
            location: facility.location.as_ref().map(|loc| EntityLocation {
                coordinates: Some(loc.clone()),
                ..Default::default()
            }),
            lifecycle_state: EntityLifecycleState::Active,
            health_status: EntityHealthStatus::Healthy,
            readiness_status: EntityReadinessStatus::Ready,
            trust_status: EntityTrustStatus::Trusted,
            metadata,
            tags: vec!["facility".into()],
            ..Default::default()
        },
    );
    if let Some(fleet) = fleet_id {
        link(
            registry,
            fleet,
            &facility.id,
            EntityRelationshipKind::Contains,
            None,
        );
    }
}

fn upsert_building(registry: &mut EntityRegistry, building: &BuildingEntity) {
    registry.entities.insert(
        building.id.clone(),
        EntityRecord {
            id: building.id.clone(),
            name: building.name.clone().or_else(|| Some(building.id.clone())),
            display_name: building.name.clone(),
            entity_type: EntityKind::Building,
            parent_id: building.facility_id.clone(),
            lifecycle_state: EntityLifecycleState::Active,
            health_status: EntityHealthStatus::Healthy,
            readiness_status: EntityReadinessStatus::Ready,
            trust_status: EntityTrustStatus::Trusted,
            metadata: building
                .building_type
                .as_ref()
                .map(|kind| HashMap::from([("building_type".into(), kind.clone())]))
                .unwrap_or_default(),
            tags: vec!["building".into()],
            ..Default::default()
        },
    );
    if let Some(facility_id) = building.facility_id.as_ref() {
        if registry.entities.contains_key(facility_id) {
            link(
                registry,
                facility_id,
                &building.id,
                EntityRelationshipKind::Contains,
                None,
            );
        }
    }
}

fn upsert_operational_zone(registry: &mut EntityRegistry, zone: &ZoneEntity) {
    registry.entities.insert(
        zone.id.clone(),
        EntityRecord {
            id: zone.id.clone(),
            name: Some(zone.id.clone()),
            display_name: Some(zone.id.clone()),
            entity_type: EntityKind::Zone,
            parent_id: zone
                .building_id
                .clone()
                .or_else(|| zone.facility_id.clone()),
            location: zone.center.as_ref().map(|center| EntityLocation {
                coordinates: Some(center.clone()),
                zone_id: Some(zone.id.clone()),
                ..Default::default()
            }),
            lifecycle_state: EntityLifecycleState::Active,
            health_status: EntityHealthStatus::Healthy,
            readiness_status: EntityReadinessStatus::Ready,
            trust_status: EntityTrustStatus::Trusted,
            metadata: HashMap::from([(
                "zone_type".into(),
                zone.zone_type
                    .clone()
                    .unwrap_or_else(|| "operational".into()),
            )]),
            tags: vec!["zone".into()],
            ..Default::default()
        },
    );
    let parent = zone.building_id.as_deref().or(zone.facility_id.as_deref());
    if let Some(parent_id) = parent {
        if registry.entities.contains_key(parent_id) {
            link(
                registry,
                parent_id,
                &zone.id,
                EntityRelationshipKind::Contains,
                None,
            );
        }
    }
}

fn upsert_declared_entity_kind(
    registry: &mut EntityRegistry,
    declared: &DeclaredEntityKind,
    fleet_id: Option<&str>,
) {
    let entity_type = EntityKind::parse(&declared.kind);
    let mut metadata = HashMap::new();
    if let Some(package) = declared.package.as_ref() {
        metadata.insert("package".into(), package.clone());
    }
    if let Some(profile) = declared.compliance_profile.as_ref() {
        metadata.insert("compliance.profile".into(), profile.clone());
    }
    let parent = declared
        .parent_id
        .clone()
        .or_else(|| fleet_id.map(String::from));
    registry.entities.insert(
        declared.id.clone(),
        EntityRecord {
            id: declared.id.clone(),
            name: Some(declared.id.clone()),
            display_name: declared
                .display_name
                .clone()
                .or_else(|| Some(declared.id.clone())),
            entity_type,
            parent_id: parent.clone(),
            capabilities: declared.capabilities.clone(),
            lifecycle_state: EntityLifecycleState::Active,
            health_status: EntityHealthStatus::Healthy,
            readiness_status: EntityReadinessStatus::Ready,
            trust_status: EntityTrustStatus::Trusted,
            metadata,
            tags: vec!["declared".into(), declared.kind.clone()],
            ..Default::default()
        },
    );
    if let Some(parent_id) = parent.as_ref() {
        if registry.entities.contains_key(parent_id) {
            link(
                registry,
                parent_id,
                &declared.id,
                EntityRelationshipKind::Contains,
                Some("entity_kind"),
            );
        }
    }
}

fn upsert_spatial_session(
    registry: &mut EntityRegistry,
    session: &RemoteExpertSession,
    fleet_id: Option<&str>,
) {
    registry.entities.insert(
        session.id.clone(),
        EntityRecord {
            id: session.id.clone(),
            name: Some(session.id.clone()),
            display_name: Some(session.id.clone()),
            entity_type: EntityKind::SpatialSession,
            parent_id: fleet_id.map(String::from),
            capabilities: session.capabilities.clone(),
            lifecycle_state: EntityLifecycleState::Active,
            health_status: EntityHealthStatus::Healthy,
            readiness_status: EntityReadinessStatus::Ready,
            trust_status: EntityTrustStatus::Trusted,
            metadata: session
                .session_type
                .as_ref()
                .map(|kind| HashMap::from([("session_type".into(), kind.clone())]))
                .unwrap_or_default(),
            tags: vec!["spatial_session".into()],
            ..Default::default()
        },
    );
    if let Some(fleet) = fleet_id {
        link(
            registry,
            fleet,
            &session.id,
            EntityRelationshipKind::Contains,
            None,
        );
    }
    if let Some(human_id) = session.field_human_id.as_ref() {
        if registry.entities.contains_key(human_id) {
            link(
                registry,
                human_id,
                &session.id,
                EntityRelationshipKind::ParticipatesIn,
                Some("field"),
            );
        }
    }
    if let Some(robot_id) = session.robot_id.as_ref() {
        if registry.entities.contains_key(robot_id) {
            link(
                registry,
                robot_id,
                &session.id,
                EntityRelationshipKind::ParticipatesIn,
                Some("robot"),
            );
        }
    }
}

fn link_zone_references(registry: &mut EntityRegistry) {
    let entity_ids: Vec<String> = registry.entities.keys().cloned().collect();
    for entity_id in entity_ids {
        let Some(entity) = registry.entities.get(&entity_id).cloned() else {
            continue;
        };
        let Some(zone_id) = entity
            .location
            .as_ref()
            .and_then(|loc| loc.zone_id.clone())
            .or_else(|| zone_id_from_coordinates(entity.location.as_ref()))
        else {
            continue;
        };
        if !registry.entities.contains_key(&zone_id) {
            continue;
        }
        link_if_absent(
            registry,
            &entity_id,
            &zone_id,
            EntityRelationshipKind::LocatedIn,
            None,
        );
    }
}

fn zone_id_from_coordinates(location: Option<&EntityLocation>) -> Option<String> {
    let coordinates = location?.coordinates.as_ref()?;
    coordinates
        .get("zone")
        .or_else(|| coordinates.get("zone_id"))
        .and_then(|value| value.as_str())
        .map(String::from)
}

fn apply_compliance_metadata(registry: &mut EntityRegistry, resolved: &ResolvedSystemConfig) {
    let assurance_profile = resolved
        .assurance_config()
        .and_then(|cfg| cfg.get("profile"))
        .and_then(|value| value.as_str())
        .map(String::from);
    let readiness_profile = resolved
        .readiness_config()
        .and_then(|cfg| cfg.get("profile"))
        .and_then(|value| value.as_str())
        .map(String::from);
    let security_profile = resolved
        .section("security")
        .and_then(|cfg| cfg.get("profile"))
        .and_then(|value| value.as_str())
        .map(String::from);
    if assurance_profile.is_none() && readiness_profile.is_none() && security_profile.is_none() {
        return;
    }
    if let Some(fleet_id) = resolved.fleet_id() {
        stamp_compliance_metadata(
            registry,
            fleet_id,
            assurance_profile.as_deref(),
            readiness_profile.as_deref(),
            security_profile.as_deref(),
        );
    }
    for robot_id in resolved.robot_ids() {
        stamp_compliance_metadata(
            registry,
            robot_id,
            assurance_profile.as_deref(),
            readiness_profile.as_deref(),
            security_profile.as_deref(),
        );
    }
}

fn stamp_compliance_metadata(
    registry: &mut EntityRegistry,
    entity_id: &str,
    assurance: Option<&str>,
    readiness: Option<&str>,
    security: Option<&str>,
) {
    let Some(entity) = registry.entities.get_mut(entity_id) else {
        return;
    };
    if let Some(profile) = assurance {
        entity
            .metadata
            .insert("assurance.profile".into(), profile.to_string());
    }
    if let Some(profile) = readiness {
        entity
            .metadata
            .insert("readiness.profile".into(), profile.to_string());
    }
    if let Some(profile) = security {
        entity
            .metadata
            .insert("security.profile".into(), profile.to_string());
    }
}

impl Default for EntityRecord {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: None,
            display_name: None,
            description: None,
            entity_type: EntityKind::Device,
            parent_id: None,
            children_ids: Vec::new(),
            labels: Vec::new(),
            tags: Vec::new(),
            version: None,
            manufacturer: None,
            model: None,
            serial_number: None,
            hardware_revision: None,
            firmware_version: None,
            software_version: None,
            provider: None,
            package: None,
            location: None,
            capabilities: Vec::new(),
            health_status: EntityHealthStatus::Unknown,
            readiness_status: EntityReadinessStatus::Unknown,
            trust_status: EntityTrustStatus::Unknown,
            security: None,
            lifecycle_state: EntityLifecycleState::Unknown,
            owner: None,
            metadata: HashMap::new(),
            audit: None,
        }
    }
}
