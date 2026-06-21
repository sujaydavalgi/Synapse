//! GPS/GNSS positioning and wireless connectivity types, verification, and simulation faults.
//!
use crate::foundations::{
    ConnectivityPolicyDecl, GeofenceDecl, RequiresConnectivityDecl, SimFaultDecl,
};
use crate::hardware::{CompatItem, CompatSeverity, HardwareProfile};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Requirement level for a connectivity channel in `requires_connectivity`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectivityRequirement {
    Required,
    Optional,
}

/// Positioning and navigation native type names.
pub fn positioning_types() -> &'static [&'static str] {
    &[
        "GpsFix",
        "GnssFix",
        "GeoPoint",
        "GeoFence",
        "Altitude",
        "Heading",
        "SpeedOverGround",
        "SatelliteInfo",
        "PositionAccuracy",
        "NavigationStatus",
    ]
}

/// Wireless and network connection type names.
pub fn connectivity_types() -> &'static [&'static str] {
    &[
        "WifiConnection",
        "BluetoothConnection",
        "BleConnection",
        "CellularConnection",
        "LTEConnection",
        "FourGConnection",
        "FiveGConnection",
        "EthernetConnection",
        "MeshConnection",
        "NetworkStatus",
        "SignalStrength",
        "Bandwidth",
        "Latency",
        "PacketLoss",
        "RoamingStatus",
        "SimIdentity",
    ]
}

/// Hardware profile connectivity option identifiers.
pub fn connectivity_options() -> &'static [&'static str] {
    &[
        "WiFi",
        "WiFi6",
        "Bluetooth",
        "Bluetooth5",
        "BLE",
        "LTE",
        "FourG",
        "4G",
        "FiveG",
        "5G",
        "Ethernet",
        "Mesh",
        "GPS",
        "GNSS",
        "Satellite",
    ]
}

/// Map a requires_connectivity key to profile connectivity tokens.
pub fn connectivity_key_to_profile_tokens(key: &str) -> Vec<&'static str> {
    match key {
        "gps" => vec!["GPS"],
        "gnss" => vec!["GNSS", "GPS"],
        "wifi" => vec!["WiFi", "WiFi6"],
        "bluetooth" => vec!["Bluetooth", "Bluetooth5", "BLE"],
        "cellular" => vec!["LTE", "FourG", "4G", "FiveG", "5G"],
        "ethernet" => vec!["Ethernet"],
        "mesh" => vec!["Mesh"],
        "satellite" => vec!["Satellite"],
        _ => vec![],
    }
}

/// Connectivity-related simulation fault names.
pub fn connectivity_faults() -> &'static [&'static str] {
    &[
        "GPSLost",
        "GpsFailure",
        "GpsDrift",
        "GpsSpoofing",
        "NetworkOutage",
        "NetworkLatencySpike",
        "WeakWifi",
        "LteOutage",
        "SatelliteOutage",
        "FiveGHandoff",
        "BluetoothDisconnect",
        "PacketLoss",
        "LatencySpike",
    ]
}

/// Security capabilities for positioning and connectivity.
pub fn connectivity_capabilities() -> &'static [&'static str] {
    &[
        "gps.read",
        "network.status",
        "wifi.connect",
        "bluetooth.scan",
        "bluetooth.pair",
        "cellular.connect",
        "network.failover",
    ]
}

/// Runtime geofence circle loaded from a program declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeofenceRuntime {
    pub name: String,
    pub center_lat: f64,
    pub center_lon: f64,
    pub radius_m: f64,
}

/// Runtime connectivity failover policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectivityPolicyRuntime {
    pub name: String,
    pub preferred: String,
    pub fallback: String,
    pub emergency: Option<String>,
    pub switch_if_latency_ms: Option<f64>,
    pub switch_if_packet_loss_pct: Option<f64>,
}

/// Haversine distance in meters between two WGS84 points.
pub fn haversine_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6_371_000.0;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
    2.0 * R * a.sqrt().asin()
}

/// Return true when `(lat, lon)` is inside the geofence circle.
pub fn geofence_contains(fence: &GeofenceRuntime, lat: f64, lon: f64) -> bool {
    haversine_m(lat, lon, fence.center_lat, fence.center_lon) <= fence.radius_m
}

/// Build runtime geofence from AST declaration.
pub fn geofence_from_decl(decl: &GeofenceDecl) -> GeofenceRuntime {
    let GeofenceDecl::GeofenceDecl {
        name,
        center_lat,
        center_lon,
        radius_m,
        ..
    } = decl;
    GeofenceRuntime {
        name: name.clone(),
        center_lat: *center_lat,
        center_lon: *center_lon,
        radius_m: *radius_m,
    }
}

/// Build runtime connectivity policy from AST declaration.
pub fn connectivity_policy_from_decl(decl: &ConnectivityPolicyDecl) -> ConnectivityPolicyRuntime {
    let ConnectivityPolicyDecl::ConnectivityPolicyDecl {
        name,
        preferred,
        fallback,
        emergency,
        switch_if_latency_ms,
        switch_if_packet_loss_pct,
        ..
    } = decl;
    ConnectivityPolicyRuntime {
        name: name.clone(),
        preferred: preferred.clone(),
        fallback: fallback.clone(),
        emergency: emergency.clone(),
        switch_if_latency_ms: *switch_if_latency_ms,
        switch_if_packet_loss_pct: *switch_if_packet_loss_pct,
    }
}

/// Map hardware monitor events to connectivity trigger `(domain, event)` pairs.
pub fn hardware_event_to_connectivity(event: &str) -> Option<(&'static str, &'static str)> {
    match event {
        "GpsFailure" => Some(("gps", "lost")),
        _ => None,
    }
}

/// Map comm bus fault names to connectivity trigger pairs.
pub fn fault_to_connectivity(fault: &str) -> Option<(&'static str, &'static str)> {
    match fault {
        "NetworkOutage" | "LteOutage" | "SatelliteOutage" | "WeakWifi" => Some(("network", "disconnected")),
        "BluetoothDisconnect" => Some(("bluetooth", "device_disconnected")),
        "FiveGHandoff" => Some(("cellular", "roaming")),
        "GpsSpoofing" => Some(("gps", "spoofed")),
        "GpsDrift" => Some(("gps", "drift")),
        _ => None,
    }
}

/// Apply GPS drift/spoofing simulation to WGS84 coordinates.
pub fn apply_gps_position_faults(
    faults: &HashSet<String>,
    true_lat: f64,
    true_lon: f64,
    sim_time_ms: f64,
) -> (f64, f64, f64) {
    if faults.contains("GpsSpoofing") {
        return (true_lat + 0.009, true_lon + 0.012, 0.3);
    }
    if faults.contains("GpsDrift") {
        let drift_m = (sim_time_ms / 1000.0) * 0.05;
        let d_deg = drift_m / 111_000.0;
        return (true_lat + d_deg, true_lon + d_deg * 0.5, 0.8);
    }
    (true_lat, true_lon, 1.0)
}

/// Rewrite a GpsFix object's coordinates after applying GPS simulation faults.
pub fn apply_gps_reading_faults(
    reading: crate::runtime::RuntimeValue,
    faults: &HashSet<String>,
    true_lat: f64,
    true_lon: f64,
    sim_time_ms: f64,
) -> crate::runtime::RuntimeValue {
    use crate::ast::UnitKind;
    use crate::runtime::RuntimeValue;
    let RuntimeValue::Object {
        type_name,
        mut fields,
    } = reading
    else {
        return reading;
    };
    if type_name != "GpsFix" && type_name != "GPSReading" {
        return RuntimeValue::Object { type_name, fields };
    }
    let (lat, lon, fix_quality) = apply_gps_position_faults(faults, true_lat, true_lon, sim_time_ms);
    fields.insert(
        "lat".into(),
        RuntimeValue::Number {
            value: lat,
            unit: UnitKind::None,
        },
    );
    fields.insert(
        "lon".into(),
        RuntimeValue::Number {
            value: lon,
            unit: UnitKind::None,
        },
    );
    if fields.contains_key("fix_quality") {
        fields.insert(
            "fix_quality".into(),
            RuntimeValue::Number {
                value: fix_quality,
                unit: UnitKind::None,
            },
        );
    }
    RuntimeValue::Object { type_name, fields }
}

/// Map an active connectivity link name to the default transport kind.
pub fn connectivity_link_to_transport(link: &str) -> crate::comm::TransportKind {
    use crate::comm::TransportKind;
    match link.to_ascii_lowercase().as_str() {
        "wifi" => TransportKind::Mqtt,
        "cellular" | "lte" | "4g" | "fiveg" | "5g" => TransportKind::Dds,
        "bluetooth" | "ble" => TransportKind::Websocket,
        "ethernet" => TransportKind::Ros2,
        "satellite" => TransportKind::Websocket,
        "network" => TransportKind::Sim,
        _ => TransportKind::Sim,
    }
}

/// Produce a [`GpsFix`]-shaped runtime object from lat/lon and optional metadata.
pub fn runtime_gps_fix(
    lat: f64,
    lon: f64,
    altitude: f64,
    fix_quality: f64,
) -> crate::runtime::RuntimeValue {
    use crate::ast::UnitKind;
    use crate::runtime::RuntimeValue;
    use std::collections::HashMap;
    RuntimeValue::Object {
        type_name: "GpsFix".into(),
        fields: HashMap::from([
            (
                "lat".into(),
                RuntimeValue::Number {
                    value: lat,
                    unit: UnitKind::None,
                },
            ),
            (
                "lon".into(),
                RuntimeValue::Number {
                    value: lon,
                    unit: UnitKind::None,
                },
            ),
            (
                "altitude".into(),
                RuntimeValue::Number {
                    value: altitude,
                    unit: UnitKind::M,
                },
            ),
            (
                "fix_quality".into(),
                RuntimeValue::Number {
                    value: fix_quality,
                    unit: UnitKind::None,
                },
            ),
        ]),
    }
}

/// Return true when the active link name refers to a cellular bearer (not satellite).
pub fn is_cellular_link(link: &str) -> bool {
    matches!(
        link.to_ascii_lowercase().as_str(),
        "cellular" | "lte" | "4g" | "fourg" | "fiveg" | "5g"
    )
}

/// Return true when the active link name refers to a satellite backhaul bearer.
pub fn is_satellite_link(link: &str) -> bool {
    link.eq_ignore_ascii_case("satellite")
}

/// Return true when the active link supports SIM/modem attestation simulation.
pub fn is_modem_bearer(link: &str) -> bool {
    is_cellular_link(link) || is_satellite_link(link)
}

/// Return true when a connectivity link name refers to Wi-Fi.
pub fn is_wifi_link(link: &str) -> bool {
    matches!(
        link.to_ascii_lowercase().as_str(),
        "wifi" | "wi-fi" | "wifi6"
    )
}

/// Return true when simulation faults disable the given connectivity link.
pub fn is_link_impaired(link: &str, faults: &HashSet<String>) -> bool {
    let link = link.to_ascii_lowercase();
    for fault in faults {
        match fault.as_str() {
            "NetworkOutage" => {
                if is_satellite_link(&link) || link == "bluetooth" || link == "ble" {
                    continue;
                }
                if is_wifi_link(&link) || is_cellular_link(&link) || link == "network" || link == "ethernet"
                {
                    return true;
                }
            }
            "WeakWifi" if is_wifi_link(&link) || link == "network" => return true,
            "LteOutage" if is_cellular_link(&link) => return true,
            "SatelliteOutage" if is_satellite_link(&link) => return true,
            _ => {}
        }
    }
    false
}

/// Produce a [`SimIdentity`]-shaped runtime object for SIM/eSIM attestation simulation.
pub fn runtime_sim_identity(link: &str, attested: bool) -> crate::runtime::RuntimeValue {
    use crate::runtime::RuntimeValue;
    use std::collections::HashMap;
    let link_lower = link.to_ascii_lowercase();
    let iccid = format!(
        "89{:010}00000000000",
        link_lower.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
            % 10_000_000_000
    );
    let carrier = if is_satellite_link(link) {
        "sim-satellite"
    } else if link_lower.contains("5g") || link_lower.contains("fiveg") {
        "sim-5g"
    } else if is_cellular_link(link) {
        "sim-lte"
    } else {
        "sim-unknown"
    };
    let esim = link_lower.contains("5g") || link_lower.contains("fiveg");
    RuntimeValue::Object {
        type_name: "SimIdentity".into(),
        fields: HashMap::from([
            (
                "iccid".into(),
                RuntimeValue::String {
                    value: iccid,
                },
            ),
            (
                "carrier".into(),
                RuntimeValue::String {
                    value: carrier.into(),
                },
            ),
            (
                "esim".into(),
                RuntimeValue::Bool { value: esim },
            ),
            (
                "attested".into(),
                RuntimeValue::Bool { value: attested },
            ),
        ]),
    }
}

fn pass(category: &str, message: impl Into<String>, line: u32, column: u32) -> CompatItem {
    CompatItem {
        category: category.into(),
        message: message.into(),
        severity: CompatSeverity::Pass,
        line,
        column,
    }
}

fn warn(category: &str, message: impl Into<String>, line: u32, column: u32) -> CompatItem {
    CompatItem {
        category: category.into(),
        message: message.into(),
        severity: CompatSeverity::Warning,
        line,
        column,
    }
}

fn error(category: &str, message: impl Into<String>, line: u32, column: u32) -> CompatItem {
    CompatItem {
        category: category.into(),
        message: message.into(),
        severity: CompatSeverity::Error,
        line,
        column,
    }
}

/// Verify `requires_connectivity` against a hardware profile's connectivity list and network metrics.
pub fn verify_requires_connectivity(
    req: &RequiresConnectivityDecl,
    profile: &HardwareProfile,
) -> Vec<CompatItem> {
    let RequiresConnectivityDecl::RequiresConnectivityDecl {
        channels,
        latency_ms_max,
        bandwidth_mbps_min,
        packet_loss_pct_max,
        span,
    } = req;
    let mut items = Vec::new();
    let line = span.start.line;
    let column = span.start.column;
    let profile_set: HashSet<String> = profile.connectivity.iter().cloned().collect();

    for (key, level) in channels {
        if *level != ConnectivityRequirement::Required {
            continue;
        }
        let tokens = connectivity_key_to_profile_tokens(key);
        if tokens.is_empty() {
            items.push(warn(
                "connectivity",
                format!("Unknown connectivity key '{key}' in requires_connectivity"),
                line,
                column,
            ));
            continue;
        }
        let satisfied = tokens.iter().any(|t| profile_set.contains(*t));
        if satisfied {
            items.push(pass(
                "connectivity",
                format!(
                    "Required connectivity '{key}' present on '{}'",
                    profile.name
                ),
                line,
                column,
            ));
        } else {
            items.push(error(
                "connectivity",
                format!(
                    "Required connectivity '{key}' not on '{}' [{}]",
                    profile.name,
                    profile.connectivity.join(", ")
                ),
                line,
                column,
            ));
        }
    }

    if let Some(min_bw) = bandwidth_mbps_min {
        match profile.network_bandwidth_mbps {
            Some(bw) if bw >= *min_bw => items.push(pass(
                "connectivity",
                format!("Bandwidth {bw} Mbps meets connectivity requirement >= {min_bw} Mbps"),
                line,
                column,
            )),
            Some(bw) => items.push(error(
                "connectivity",
                format!(
                    "Connectivity bandwidth requirement {min_bw} Mbps exceeds target {bw} Mbps"
                ),
                line,
                column,
            )),
            None => items.push(warn(
                "connectivity",
                "Target bandwidth unknown — cannot verify connectivity bandwidth requirement",
                line,
                column,
            )),
        }
    }

    if let Some(max_lat) = latency_ms_max {
        match profile.network_latency_ms {
            Some(lat) if lat <= *max_lat => items.push(pass(
                "connectivity",
                format!("Latency {lat} ms meets connectivity requirement <= {max_lat} ms"),
                line,
                column,
            )),
            Some(lat) => items.push(error(
                "connectivity",
                format!(
                    "Connectivity latency requirement {max_lat} ms exceeded by target {lat} ms"
                ),
                line,
                column,
            )),
            None => items.push(warn(
                "connectivity",
                "Target latency unknown — cannot verify connectivity latency requirement",
                line,
                column,
            )),
        }
    }

    if let Some(max_loss) = packet_loss_pct_max {
        match profile.packet_loss_pct {
            Some(loss) if loss <= *max_loss => items.push(pass(
                "connectivity",
                format!("Packet loss {loss}% meets requirement <= {max_loss}%"),
                line,
                column,
            )),
            Some(loss) => items.push(error(
                "connectivity",
                format!("Packet loss {loss}% exceeds requirement <= {max_loss}%"),
                line,
                column,
            )),
            None => items.push(warn(
                "connectivity",
                "Target packet loss unknown — cannot verify packet_loss requirement",
                line,
                column,
            )),
        }
    }

    items
}

/// Apply a connectivity or positioning simulation fault to a hardware profile.
pub fn apply_connectivity_fault(
    mut profile: HardwareProfile,
    fault: &SimFaultDecl,
) -> HardwareProfile {
    match fault.fault_type.as_str() {
        "GPSLost" | "GpsFailure" => {
            profile.sensors.retain(|s| s != "GPS" && s != "GNSS");
            profile.connectivity.retain(|c| c != "GPS" && c != "GNSS");
        }
        "GpsDrift" | "GpsSpoofing" => {}
        "NetworkOutage" | "LteOutage" => {
            profile.network_bandwidth_mbps = Some(0.0);
            profile.network_latency_ms = Some(10_000.0);
            profile.connectivity.retain(|c| {
                !matches!(
                    c.as_str(),
                    "WiFi"
                        | "WiFi6"
                        | "LTE"
                        | "FourG"
                        | "4G"
                        | "FiveG"
                        | "5G"
                        | "Ethernet"
                        | "Mesh"
                )
            });
        }
        "SatelliteOutage" => {
            profile.network_bandwidth_mbps = Some(0.0);
            profile.network_latency_ms = Some(10_000.0);
            profile.connectivity.retain(|c| c != "Satellite");
        }
        "WeakWifi" => {
            profile.network_bandwidth_mbps = Some(1.0);
            profile.network_latency_ms = Some(500.0);
        }
        "NetworkLatencySpike" | "LatencySpike" => {
            profile.network_latency_ms = Some(2000.0);
        }
        "FiveGHandoff" => {
            profile.network_latency_ms = Some(150.0);
        }
        "BluetoothDisconnect" => {
            profile
                .connectivity
                .retain(|c| !matches!(c.as_str(), "Bluetooth" | "Bluetooth5" | "BLE"));
        }
        "PacketLoss" => {
            profile.packet_loss_pct = Some(10.0);
        }
        _ => {}
    }
    profile
}

/// Validate geofence declaration geometry.
pub fn validate_geofence(geofence: &GeofenceDecl) -> Vec<CompatItem> {
    let GeofenceDecl::GeofenceDecl {
        name,
        center_lat,
        center_lon,
        radius_m,
        span,
    } = geofence;
    let mut items = Vec::new();
    let line = span.start.line;
    let column = span.start.column;

    if !(-90.0..=90.0).contains(center_lat) {
        items.push(error(
            "geofence",
            format!("Geofence '{name}' center latitude {center_lat} out of range [-90, 90]"),
            line,
            column,
        ));
    } else if !(-180.0..=180.0).contains(center_lon) {
        items.push(error(
            "geofence",
            format!("Geofence '{name}' center longitude {center_lon} out of range [-180, 180]"),
            line,
            column,
        ));
    } else if *radius_m <= 0.0 {
        items.push(error(
            "geofence",
            format!("Geofence '{name}' radius must be positive"),
            line,
            column,
        ));
    } else {
        items.push(pass(
            "geofence",
            format!("Geofence '{name}' geometry valid"),
            line,
            column,
        ));
    }
    items
}

/// Validate connectivity failover policy link names.
pub fn validate_connectivity_policy(policy: &ConnectivityPolicyDecl) -> Vec<CompatItem> {
    let ConnectivityPolicyDecl::ConnectivityPolicyDecl {
        name,
        preferred,
        fallback,
        emergency,
        span,
        ..
    } = policy;
    let line = span.start.line;
    let column = span.start.column;
    let mut items = vec![pass(
        "connectivity_policy",
        format!("Connectivity policy '{name}' parsed: preferred={preferred}, fallback={fallback}"),
        line,
        column,
    )];
    if preferred == fallback {
        items.push(warn(
            "connectivity_policy",
            format!("Policy '{name}' preferred and fallback are the same link"),
            line,
            column,
        ));
    }
    if let Some(em) = emergency {
        if em == preferred || em == fallback {
            items.push(warn(
                "connectivity_policy",
                format!("Policy '{name}' emergency link duplicates preferred or fallback"),
                line,
                column,
            ));
        }
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn haversine_zero_distance() {
        let d = haversine_m(30.0, -97.0, 30.0, -97.0);
        assert!(d.abs() < 0.01);
    }

    #[test]
    fn connectivity_link_to_transport_maps_wifi() {
        use crate::comm::TransportKind;
        assert_eq!(
            connectivity_link_to_transport("wifi"),
            TransportKind::Mqtt
        );
        assert_eq!(
            connectivity_link_to_transport("cellular"),
            TransportKind::Dds
        );
    }

    #[test]
    fn geofence_contains_center() {
        let fence = GeofenceRuntime {
            name: "Home".into(),
            center_lat: 30.2672,
            center_lon: -97.7431,
            radius_m: 100.0,
        };
        assert!(geofence_contains(&fence, 30.2672, -97.7431));
    }

    #[test]
    fn gps_spoofing_offsets_coordinates() {
        use std::collections::HashSet;
        let faults = HashSet::from(["GpsSpoofing".to_string()]);
        let (lat, lon, fq) = apply_gps_position_faults(&faults, 30.0, -97.0, 0.0);
        assert!((lat - 30.009).abs() < 1e-6);
        assert!((lon - (-96.988)).abs() < 1e-6);
        assert!((fq - 0.3).abs() < 1e-6);
    }

    #[test]
    fn gps_drift_increases_with_sim_time() {
        use std::collections::HashSet;
        let faults = HashSet::from(["GpsDrift".to_string()]);
        let (lat0, _, _) = apply_gps_position_faults(&faults, 30.0, -97.0, 0.0);
        let (lat1, _, _) = apply_gps_position_faults(&faults, 30.0, -97.0, 10_000.0);
        assert!(lat1 > lat0);
    }

    #[test]
    fn gps_drift_maps_to_trigger() {
        assert_eq!(
            fault_to_connectivity("GpsDrift"),
            Some(("gps", "drift"))
        );
    }

    #[test]
    fn connectivity_link_to_transport_maps_satellite() {
        assert_eq!(
            connectivity_link_to_transport("satellite"),
            crate::comm::TransportKind::Websocket
        );
    }

    #[test]
    fn is_link_impaired_for_lte_outage_on_cellular() {
        use std::collections::HashSet;
        let faults = HashSet::from(["LteOutage".to_string()]);
        assert!(is_link_impaired("cellular", &faults));
        assert!(!is_link_impaired("satellite", &faults));
    }

    #[test]
    fn runtime_sim_identity_cellular_attested() {
        use crate::runtime::RuntimeValue;
        let value = runtime_sim_identity("cellular", true);
        let RuntimeValue::Object { type_name, fields } = value else {
            panic!("expected object");
        };
        assert_eq!(type_name, "SimIdentity");
        assert!(fields.contains_key("iccid"));
        assert!(fields.contains_key("attested"));
    }

    #[test]
    fn runtime_gps_fix_has_gpsfix_type() {
        let fix = runtime_gps_fix(30.0, -97.0, 150.0, 1.0);
        assert!(matches!(
            fix,
            crate::runtime::RuntimeValue::Object { type_name, .. } if type_name == "GpsFix"
        ));
    }
}
