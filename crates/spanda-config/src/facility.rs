//! Facility, building, and zone entities from solution blueprint TOML.
//!
use serde::{Deserialize, Serialize};

/// Top-level facility site (warehouse, hospital campus, plant).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FacilityEntity {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub facility_type: Option<String>,
    #[serde(default)]
    pub location: Option<toml::Value>,
    #[serde(default)]
    pub compliance_profile: Option<String>,
}

/// Building or wing within a facility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildingEntity {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub facility_id: Option<String>,
    #[serde(default)]
    pub building_type: Option<String>,
}

/// Operational or geofenced zone within a building or facility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZoneEntity {
    pub id: String,
    #[serde(default, rename = "type")]
    pub zone_type: Option<String>,
    #[serde(default)]
    pub facility_id: Option<String>,
    #[serde(default)]
    pub building_id: Option<String>,
    #[serde(default)]
    pub center: Option<toml::Value>,
    #[serde(default)]
    pub radius_m: Option<f64>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Package- or blueprint-declared custom entity kind instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeclaredEntityKind {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub package: Option<String>,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub compliance_profile: Option<String>,
}

/// Parsed spatial hierarchy and declared entity kinds from merged TOML.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FacilityRegistry {
    #[serde(default)]
    pub facilities: Vec<FacilityEntity>,
    #[serde(default)]
    pub buildings: Vec<BuildingEntity>,
    #[serde(default)]
    pub zones: Vec<ZoneEntity>,
    #[serde(default)]
    pub entity_kinds: Vec<DeclaredEntityKind>,
}

impl FacilityRegistry {
    /// Parse facility tables from merged configuration TOML.
    pub fn from_raw(raw: &toml::Value) -> Self {
        // Build spatial hierarchy and declared entity kinds from blueprint tables.
        //
        // Parameters:
        // - `raw` — merged resolved configuration value
        //
        // Returns:
        // Parsed facility registry, or empty when tables are absent.
        //
        // Options:
        // None.
        //
        // Example:
        // let facilities = FacilityRegistry::from_raw(&resolved.raw);

        let mut registry = Self::default();
        if let Some(table) = raw.get("facilities").and_then(|v| v.as_array()) {
            for entry in table {
                if let Ok(facility) = entry.clone().try_into() {
                    registry.facilities.push(facility);
                }
            }
        }
        if let Some(table) = raw.get("buildings").and_then(|v| v.as_array()) {
            for entry in table {
                if let Ok(building) = entry.clone().try_into() {
                    registry.buildings.push(building);
                }
            }
        }
        if let Some(table) = raw.get("zones").and_then(|v| v.as_array()) {
            for entry in table {
                if let Ok(zone) = entry.clone().try_into() {
                    registry.zones.push(zone);
                }
            }
        }
        if let Some(table) = raw.get("entity_kinds").and_then(|v| v.as_array()) {
            for entry in table {
                if let Ok(kind) = entry.clone().try_into() {
                    registry.entity_kinds.push(kind);
                }
            }
        }
        registry
    }

    pub fn facility(&self, id: &str) -> Option<&FacilityEntity> {
        self.facilities.iter().find(|f| f.id == id)
    }

    pub fn zone(&self, id: &str) -> Option<&ZoneEntity> {
        self.zones.iter().find(|z| z.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_facility_building_zone_tables() {
        let toml_str = r#"
[[facilities]]
id = "warehouse-a"
name = "Warehouse A"
compliance_profile = "iso13849"

[[buildings]]
id = "north-wing"
facility_id = "warehouse-a"

[[zones]]
id = "pick-zone-1"
building_id = "north-wing"
type = "operational"

[[entity_kinds]]
id = "calibration-bay"
kind = "calibration_station"
capabilities = ["calibrate"]
package = "acme.calibration"
"#;
        let raw: toml::Value = toml::from_str(toml_str).unwrap();
        let registry = FacilityRegistry::from_raw(&raw);
        assert_eq!(registry.facilities.len(), 1);
        assert_eq!(registry.buildings.len(), 1);
        assert_eq!(registry.zones.len(), 1);
        assert_eq!(registry.entity_kinds.len(), 1);
        assert_eq!(
            registry.facilities[0].compliance_profile.as_deref(),
            Some("iso13849")
        );
    }
}
