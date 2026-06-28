//! Human operators, wearables, and spatial devices in fleet configuration.
//!
use crate::device_identity::{DeviceIdentityRecord, TrustLevel};
use crate::device_tree::{DeviceTree, FleetNode};
use crate::mission_approval::MissionApprovalSeed;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Expiring operator certification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CertificationRecord {
    pub id: String,
    #[serde(default)]
    pub expires: Option<String>,
    #[serde(default)]
    pub issuer: Option<String>,
}

/// Human operator or collaborator entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HumanEntity {
    pub id: String,
    pub role: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub certifications: Vec<CertificationRecord>,
    #[serde(default)]
    pub assignments: Option<toml::Value>,
    #[serde(default)]
    pub availability: Option<String>,
    #[serde(default)]
    pub trust_level: Option<String>,
    #[serde(default)]
    pub location: Option<toml::Value>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub health_status: Option<String>,
}

/// Wearable device bound to a human operator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WearableEntity {
    pub id: String,
    #[serde(rename = "type")]
    pub device_type: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub human_id: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default, alias = "endpoint")]
    pub endpoint_url: Option<String>,
    #[serde(default)]
    pub trust_level: Option<String>,
}

/// AR, VR, drone, or IoT spatial device node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatialDeviceEntity {
    pub id: String,
    #[serde(rename = "type")]
    pub device_type: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub human_id: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default, alias = "endpoint")]
    pub endpoint_url: Option<String>,
    #[serde(default)]
    pub trust_level: Option<String>,
}

/// Geofenced or operational hazard zone for human–robot context awareness.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HazardZoneEntity {
    pub id: String,
    #[serde(rename = "type", default)]
    pub zone_type: Option<String>,
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub center: Option<toml::Value>,
    #[serde(default)]
    pub radius_m: Option<f64>,
    #[serde(default)]
    pub linked_robots: Vec<String>,
    #[serde(default)]
    pub alert_on_entry: Option<bool>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Digital twin metadata for humans, teams, or training sessions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwinEntity {
    pub id: String,
    pub entity_id: String,
    pub entity_type: String,
    #[serde(default)]
    pub mirror: Vec<String>,
    #[serde(default)]
    pub replay: Option<bool>,
    #[serde(default)]
    pub telemetry_sync: Option<bool>,
    #[serde(default)]
    pub training_session_id: Option<String>,
}

/// Remote expert or collaborative spatial session configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoteExpertSession {
    pub id: String,
    #[serde(rename = "type", default)]
    pub session_type: Option<String>,
    #[serde(default)]
    pub field_human_id: Option<String>,
    #[serde(default)]
    pub expert_human_id: Option<String>,
    #[serde(default)]
    pub robot_id: Option<String>,
    #[serde(default)]
    pub ar_device_id: Option<String>,
    #[serde(default)]
    pub camera_device_id: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// Control Center logical node in the fleet tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlCenterEntity {
    pub id: String,
    #[serde(rename = "type", default = "default_control_center_type")]
    pub node_type: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

fn default_control_center_type() -> String {
    "ControlCenter".into()
}

/// Parsed humans, wearables, and spatial devices from fleet and flat TOML tables.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HumanRegistry {
    #[serde(default)]
    pub humans: Vec<HumanEntity>,
    #[serde(default)]
    pub wearables: Vec<WearableEntity>,
    #[serde(default)]
    pub ar_devices: Vec<SpatialDeviceEntity>,
    #[serde(default)]
    pub vr_devices: Vec<SpatialDeviceEntity>,
    #[serde(default)]
    pub drones: Vec<SpatialDeviceEntity>,
    #[serde(default)]
    pub iot_devices: Vec<SpatialDeviceEntity>,
    #[serde(default)]
    pub control_center: Vec<ControlCenterEntity>,
    #[serde(default)]
    pub spatial_sessions: Vec<RemoteExpertSession>,
    #[serde(default)]
    pub hazard_zones: Vec<HazardZoneEntity>,
    #[serde(default)]
    pub twins: Vec<TwinEntity>,
    #[serde(default)]
    pub mission_approvals: Vec<MissionApprovalSeed>,
}

impl HumanEntity {
    pub fn trust_level_enum(&self) -> TrustLevel {
        self.trust_level
            .as_deref()
            .map(TrustLevel::parse)
            .unwrap_or(TrustLevel::Unknown)
    }

    pub fn is_available(&self) -> bool {
        matches!(
            self.availability
                .as_deref()
                .map(str::to_ascii_lowercase)
                .as_deref(),
            Some("available") | None
        )
    }

    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    pub fn has_valid_certification(&self, cert_id: &str, today: &str) -> bool {
        self.certifications
            .iter()
            .any(|cert| cert.id == cert_id && cert_expires_on_or_after(&cert.expires, today))
    }
}

impl HumanRegistry {
    pub fn from_resolved_parts(tree: &DeviceTree, raw: &toml::Value) -> Self {
        // Merge fleet-nested and flat `[[humans]]` / wearable tables into one registry.
        //
        // Parameters:
        // - `tree` — parsed device tree with optional fleet human nodes
        // - `raw` — merged TOML for flat tables
        //
        // Returns:
        // Combined human collaboration registry.
        //
        // Options:
        // None.
        //
        // Example:
        // let registry = HumanRegistry::from_resolved_parts(&tree, &merged);

        let mut registry = Self::default();
        if let Some(ref fleet) = tree.fleet {
            merge_fleet_humans(fleet, &mut registry);
        }
        merge_flat_tables(raw, &mut registry);
        dedupe_registry(&mut registry);
        registry
    }

    pub fn human(&self, id: &str) -> Option<&HumanEntity> {
        self.humans.iter().find(|h| h.id == id)
    }

    pub fn wearables_for_human(&self, human_id: &str) -> Vec<&WearableEntity> {
        self.wearables
            .iter()
            .filter(|w| w.human_id.as_deref() == Some(human_id))
            .collect()
    }

    pub fn has_operators(&self) -> bool {
        !self.humans.is_empty()
    }

    pub fn operator_ids(&self) -> Vec<String> {
        self.humans.iter().map(|h| h.id.clone()).collect()
    }

    pub fn spatial_session(&self, id: &str) -> Option<&RemoteExpertSession> {
        self.spatial_sessions.iter().find(|s| s.id == id)
    }

    pub fn hazard_zone(&self, id: &str) -> Option<&HazardZoneEntity> {
        self.hazard_zones.iter().find(|z| z.id == id)
    }

    pub fn human_twins(&self) -> Vec<&TwinEntity> {
        self.twins
            .iter()
            .filter(|twin| twin.entity_type.eq_ignore_ascii_case("human"))
            .collect()
    }

    pub fn twin_for_entity(&self, entity_id: &str) -> Option<&TwinEntity> {
        self.twins.iter().find(|t| t.entity_id == entity_id)
    }
}

pub fn operator_capability_names() -> &'static [&'static str] {
    &[
        "operate_robot",
        "approve_mission",
        "approve_recovery",
        "emergency_override",
        "drone_pilot",
        "medical_responder",
        "hazmat_certified",
        "remote_expert",
        "maintenance_technician",
        "forklift_operator",
        "search_rescue_operator",
    ]
}

pub fn is_operator_capability(name: &str) -> bool {
    operator_capability_names().contains(&name)
}

pub fn records_from_human_registry(registry: &HumanRegistry) -> Vec<DeviceIdentityRecord> {
    // Convert human collaboration entities into device registry records.
    //
    // Parameters:
    // - `registry` — human/wearable/spatial registry
    //
    // Returns:
    // Device identity rows for pool ingest and traceability.
    //
    // Options:
    // None.
    //
    // Example:
    // let rows = records_from_human_registry(&human_registry);

    let mut out = Vec::new();
    for human in &registry.humans {
        out.push(DeviceIdentityRecord {
            id: human.id.clone(),
            device_type: "Human".into(),
            logical_name: human.display_name.clone(),
            capabilities: human.capabilities.clone(),
            trust_level: human.trust_level.clone(),
            health_status: human.health_status.clone(),
            lifecycle_state: human.availability.clone(),
            ..DeviceIdentityRecord::default()
        });
    }
    for wearable in &registry.wearables {
        out.push(spatial_to_identity_wearable(wearable));
    }
    for ar in &registry.ar_devices {
        out.push(spatial_to_identity(ar.id.as_str(), "ARDevice", ar));
    }
    for vr in &registry.vr_devices {
        out.push(spatial_to_identity(vr.id.as_str(), "VRDevice", vr));
    }
    for drone in &registry.drones {
        out.push(spatial_to_identity(drone.id.as_str(), "Drone", drone));
    }
    for iot in &registry.iot_devices {
        out.push(spatial_to_identity(iot.id.as_str(), "IoTDevice", iot));
    }
    for cc in &registry.control_center {
        out.push(DeviceIdentityRecord {
            id: cc.id.clone(),
            device_type: "ControlCenter".into(),
            capabilities: cc.capabilities.clone(),
            ..DeviceIdentityRecord::default()
        });
    }
    out
}

fn spatial_to_identity(id: &str, kind: &str, entity: &SpatialDeviceEntity) -> DeviceIdentityRecord {
    DeviceIdentityRecord {
        id: id.into(),
        device_type: kind.into(),
        endpoint_url: entity.endpoint_url.clone(),
        protocol: entity.protocol.clone(),
        provider: entity.provider.clone(),
        capabilities: entity.capabilities.clone(),
        trust_level: entity.trust_level.clone(),
        ..DeviceIdentityRecord::default()
    }
}

fn spatial_to_identity_wearable(wearable: &WearableEntity) -> DeviceIdentityRecord {
    DeviceIdentityRecord {
        id: wearable.id.clone(),
        device_type: "Wearable".into(),
        endpoint_url: wearable.endpoint_url.clone(),
        protocol: wearable.protocol.clone(),
        provider: wearable.provider.clone(),
        capabilities: wearable.capabilities.clone(),
        trust_level: wearable.trust_level.clone(),
        ..DeviceIdentityRecord::default()
    }
}

fn merge_fleet_humans(fleet: &FleetNode, registry: &mut HumanRegistry) {
    registry.humans.extend(fleet.humans.clone());
    registry.wearables.extend(fleet.wearables.clone());
    registry.ar_devices.extend(fleet.ar_devices.clone());
    registry.vr_devices.extend(fleet.vr_devices.clone());
    registry.drones.extend(fleet.drones.clone());
    registry.iot_devices.extend(fleet.iot_devices.clone());
    registry.control_center.extend(fleet.control_center.clone());
    registry.hazard_zones.extend(fleet.hazard_zones.clone());
}

fn merge_flat_tables(raw: &toml::Value, registry: &mut HumanRegistry) {
    append_table(raw, "humans", &mut registry.humans);
    append_table(raw, "wearables", &mut registry.wearables);
    append_table(raw, "ar_devices", &mut registry.ar_devices);
    append_table(raw, "vr_devices", &mut registry.vr_devices);
    append_table(raw, "drones", &mut registry.drones);
    append_table(raw, "iot_devices", &mut registry.iot_devices);
    append_table(raw, "control_center", &mut registry.control_center);
    append_table(raw, "spatial_sessions", &mut registry.spatial_sessions);
    append_table(raw, "hazard_zones", &mut registry.hazard_zones);
    append_table(raw, "twins", &mut registry.twins);
    append_table(raw, "mission_approvals", &mut registry.mission_approvals);
}

fn append_table<T: for<'de> Deserialize<'de>>(raw: &toml::Value, key: &str, target: &mut Vec<T>) {
    let Some(arr) = raw.get(key).and_then(|v| v.as_array()) else {
        return;
    };
    for entry in arr {
        if let Ok(item) = entry.clone().try_into() {
            target.push(item);
        }
    }
}

fn dedupe_registry(registry: &mut HumanRegistry) {
    dedupe_by_id(&mut registry.humans, |h| h.id.clone());
    dedupe_by_id(&mut registry.wearables, |w| w.id.clone());
    dedupe_by_id(&mut registry.ar_devices, |d| d.id.clone());
    dedupe_by_id(&mut registry.vr_devices, |d| d.id.clone());
    dedupe_by_id(&mut registry.drones, |d| d.id.clone());
    dedupe_by_id(&mut registry.iot_devices, |d| d.id.clone());
    dedupe_by_id(&mut registry.control_center, |c| c.id.clone());
    dedupe_by_id(&mut registry.spatial_sessions, |s| s.id.clone());
    dedupe_by_id(&mut registry.hazard_zones, |z| z.id.clone());
    dedupe_by_id(&mut registry.twins, |t| t.id.clone());
    dedupe_by_id(&mut registry.mission_approvals, |m| m.id.clone());
}

fn dedupe_by_id<T, F>(items: &mut Vec<T>, id_fn: F)
where
    F: Fn(&T) -> String,
{
    let mut seen = HashMap::new();
    let mut deduped = Vec::with_capacity(items.len());
    for item in items.drain(..) {
        let id = id_fn(&item);
        if seen.insert(id, ()).is_none() {
            deduped.push(item);
        }
    }
    *items = deduped;
}

fn cert_expires_on_or_after(expires: &Option<String>, today: &str) -> bool {
    match expires {
        Some(date) => date.as_str() >= today,
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device_tree::DeviceTree;

    #[test]
    fn parses_fleet_humans_and_wearables() {
        let toml_str = r#"
[fleet]
id = "collab-fleet"

[[fleet.humans]]
id = "operator-001"
role = "operator"
capabilities = ["operate_robot"]
availability = "available"
trust_level = "trusted"

[[fleet.wearables]]
id = "watch-001"
type = "SmartWatch"
human_id = "operator-001"
capabilities = ["heart_rate"]
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();
        let tree = DeviceTree::from_toml_value(&value);
        let registry = HumanRegistry::from_resolved_parts(&tree, &value);
        assert_eq!(registry.humans.len(), 1);
        assert_eq!(registry.wearables.len(), 1);
        assert!(registry
            .human("operator-001")
            .unwrap()
            .has_capability("operate_robot"));
    }

    #[test]
    fn merges_flat_humans_table() {
        let toml_str = r#"
[[humans]]
id = "supervisor-001"
role = "supervisor"
capabilities = ["approve_mission"]
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();
        let tree = DeviceTree::default();
        let registry = HumanRegistry::from_resolved_parts(&tree, &value);
        assert_eq!(registry.humans.len(), 1);
        assert_eq!(registry.humans[0].role, "supervisor");
    }

    #[test]
    fn parses_hazard_zones_table() {
        let toml_str = r#"
[[hazard_zones]]
id = "restricted-a"
type = "restricted"
severity = "high"
radius_m = 50.0
linked_robots = ["AMR"]
alert_on_entry = true
"#;
        let value: toml::Value = toml::from_str(toml_str).unwrap();
        let tree = DeviceTree::default();
        let registry = HumanRegistry::from_resolved_parts(&tree, &value);
        assert_eq!(registry.hazard_zones.len(), 1);
        assert_eq!(
            registry
                .hazard_zone("restricted-a")
                .unwrap()
                .zone_type
                .as_deref(),
            Some("restricted")
        );
    }
}
