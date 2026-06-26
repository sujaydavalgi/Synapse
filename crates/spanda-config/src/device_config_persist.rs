//! Persist device pool mutations back to configuration fragments on disk.
//!
use crate::device_identity::DeviceIdentityRecord;
use crate::error::{ConfigError, ConfigResult};
use crate::manifest::SpandaManifest;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Result of writing a device record to disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DevicePersistResult {
    pub path: PathBuf,
    pub updated: bool,
    pub created: bool,
}

/// Resolve candidate fragment paths for device records (network first, then devices).
pub fn device_fragment_paths(project_root: &Path, manifest: &SpandaManifest) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(ref rel) = manifest.config.network_devices {
        paths.push(project_root.join(rel));
    }
    if let Some(ref rel) = manifest.config.devices {
        let p = project_root.join(rel);
        if !paths.contains(&p) {
            paths.push(p);
        }
    }
    paths
}

/// Update a device record in the first matching configuration fragment.
pub fn persist_device_record(
    project_root: &Path,
    manifest: &SpandaManifest,
    record: &DeviceIdentityRecord,
) -> ConfigResult<DevicePersistResult> {
    for path in device_fragment_paths(project_root, manifest) {
        if !path.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&path).map_err(|source| ConfigError::Io {
            path: path.clone(),
            source,
        })?;
        let mut value: toml::Value =
            toml::from_str(&content).map_err(|source| ConfigError::TomlParse {
                path: path.clone(),
                source,
            })?;
        let mut updated = false;
        let mut created = false;
        if update_flat_devices(&mut value, record, &mut updated) {
            // flat [[devices]] updated
        } else if update_fleet_devices(&mut value, record, &mut updated) {
            // fleet tree device updated
        } else {
            append_flat_device(&mut value, record);
            created = true;
            updated = true;
        }
        if updated {
            let out = toml::to_string_pretty(&value).map_err(|e| ConfigError::InvalidManifest {
                detail: e.to_string(),
            })?;
            std::fs::write(&path, out).map_err(|source| ConfigError::Io {
                path: path.clone(),
                source,
            })?;
            return Ok(DevicePersistResult {
                path,
                updated: !created,
                created,
            });
        }
    }
    Err(ConfigError::InvalidManifest {
        detail: format!("device '{}' not found in any fragment", record.id),
    })
}

fn update_flat_devices(
    value: &mut toml::Value,
    record: &DeviceIdentityRecord,
    updated: &mut bool,
) -> bool {
    let Some(arr) = value.get_mut("devices").and_then(|v| v.as_array_mut()) else {
        return false;
    };
    for entry in arr.iter_mut() {
        let Some(id) = entry.get("id").and_then(|v| v.as_str()) else {
            continue;
        };
        if id != record.id {
            continue;
        }
        merge_record_into_toml(entry, record);
        *updated = true;
        return true;
    }
    false
}

fn update_fleet_devices(
    value: &mut toml::Value,
    record: &DeviceIdentityRecord,
    updated: &mut bool,
) -> bool {
    let Some(robots) = value
        .get_mut("fleet")
        .and_then(|f| f.get_mut("robots"))
        .and_then(|r| r.as_array_mut())
    else {
        return false;
    };
    for robot in robots.iter_mut() {
        let Some(devices) = robot
            .get_mut("compute")
            .and_then(|c| c.get_mut("devices"))
            .and_then(|d| d.as_array_mut())
        else {
            continue;
        };
        for entry in devices.iter_mut() {
            let Some(id) = entry.get("id").and_then(|v| v.as_str()) else {
                continue;
            };
            if id != record.id {
                continue;
            }
            merge_record_into_toml(entry, record);
            *updated = true;
            return true;
        }
    }
    false
}

fn append_flat_device(value: &mut toml::Value, record: &DeviceIdentityRecord) {
    let table = record_to_toml_table(record);
    match value.get_mut("devices") {
        Some(toml::Value::Array(arr)) => arr.push(toml::Value::Table(table)),
        _ => {
            value.as_table_mut().expect("toml root").insert(
                "devices".into(),
                toml::Value::Array(vec![toml::Value::Table(table)]),
            );
        }
    }
}

fn merge_record_into_toml(entry: &mut toml::Value, record: &DeviceIdentityRecord) {
    let table = entry.as_table_mut().expect("device table");
    set_opt(table, "logical_name", &record.logical_name);
    set_opt(table, "lifecycle_state", &record.lifecycle_state);
    set_opt(table, "assigned_robot", &record.assigned_robot);
    set_opt(table, "trust_level", &record.trust_level);
    set_opt(table, "robot_id", &record.robot_id);
    set_opt(table, "redundant_group", &record.redundant_group);
    if let Some(p) = record.failover_priority {
        table.insert("failover_priority".into(), toml::Value::Integer(p as i64));
    }
    set_opt(table, "calibration_status", &record.calibration_status);
    set_opt(table, "health_status", &record.health_status);
    if let Some(ms) = record.calibration_expiry_ms {
        table.insert("calibration_expiry_ms".into(), toml::Value::Float(ms));
    }
}

fn record_to_toml_table(record: &DeviceIdentityRecord) -> toml::map::Map<String, toml::Value> {
    let mut table = toml::map::Map::new();
    table.insert("id".into(), toml::Value::String(record.id.clone()));
    table.insert(
        "type".into(),
        toml::Value::String(record.device_type.clone()),
    );
    set_opt(&mut table, "logical_name", &record.logical_name);
    set_opt(&mut table, "lifecycle_state", &record.lifecycle_state);
    set_opt(&mut table, "assigned_robot", &record.assigned_robot);
    set_opt(&mut table, "trust_level", &record.trust_level);
    set_opt(&mut table, "robot_id", &record.robot_id);
    set_opt(&mut table, "provider", &record.provider);
    if !record.capabilities.is_empty() {
        table.insert(
            "capabilities".into(),
            toml::Value::Array(
                record
                    .capabilities
                    .iter()
                    .map(|c| toml::Value::String(c.clone()))
                    .collect(),
            ),
        );
    }
    table
}

fn set_opt(table: &mut toml::map::Map<String, toml::Value>, key: &str, value: &Option<String>) {
    if let Some(ref v) = value {
        table.insert(key.into(), toml::Value::String(v.clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn persists_flat_device_fields() {
        let dir = std::env::temp_dir().join(format!("spanda-persist-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("spanda.toml"),
            r#"
[project]
name = "test"
version = "0.1.0"
[config]
network_devices = "network.toml"
"#,
        )
        .unwrap();
        fs::write(
            dir.join("network.toml"),
            r#"
[[devices]]
id = "cam-1"
type = "Camera"
"#,
        )
        .unwrap();
        let manifest = SpandaManifest::load_from_dir(&dir).unwrap();
        let record = DeviceIdentityRecord {
            id: "cam-1".into(),
            device_type: "Camera".into(),
            lifecycle_state: Some("assigned".into()),
            assigned_robot: Some("rover-1".into()),
            logical_name: Some("front_camera".into()),
            ..Default::default()
        };
        let result = persist_device_record(&dir, &manifest, &record).unwrap();
        assert!(result.updated);
        let body = fs::read_to_string(result.path).unwrap();
        assert!(body.contains("front_camera"));
        assert!(body.contains("assigned"));
        let _ = fs::remove_dir_all(&dir);
    }
}
