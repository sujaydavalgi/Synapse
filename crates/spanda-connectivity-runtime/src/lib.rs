//! GPS/GNSS positioning and wireless connectivity types, verification, and simulation faults.
//!
//! **Lean-core note:** Type names and verification rules stay in core. Driver implementations
//! belong in official packages (`spanda-gps`, `spanda-wifi`, `spanda-ble`, `spanda-cellular`).
//! This module remains as a compatibility shim until callers migrate to package imports.
//!
use serde::{Deserialize, Serialize};
use spanda_ast::foundations::{ConnectivityPolicyDecl, GeofenceDecl, SimFaultDecl};
use spanda_connectivity::HardwareProfile;
use std::collections::HashSet;

pub use spanda_connectivity::{
    apply_gps_position_faults, connectivity_capabilities, connectivity_faults,
    connectivity_key_to_profile_tokens, connectivity_options, connectivity_types,
    fault_to_connectivity, geofence_contains, hardware_event_to_connectivity, haversine_m,
    is_cellular_link, is_link_impaired, is_modem_bearer, is_satellite_link, is_wifi_link,
    positioning_types, ConnectivityRequirement, GeofenceRuntime,
};

pub mod connectivity_validate;

pub use connectivity_validate::{
    validate_connectivity_policy, validate_geofence, verify_requires_connectivity,
};

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

/// Build runtime geofence from AST declaration.
pub fn geofence_from_decl(decl: &GeofenceDecl) -> GeofenceRuntime {
    // Description:
    //     Geofence from decl.
    //
    // Inputs:
    //     decl: &GeofenceDecl
    //         Caller-supplied decl.
    //
    // Outputs:
    //     result: GeofenceRuntime
    //         Return value from `geofence_from_decl`.
    //
    // Example:

    //     let result = spanda_connectivity_runtime::geofence_from_decl(decl);

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
    // Description:
    //     Connectivity policy from decl.
    //
    // Inputs:
    //     decl: &ConnectivityPolicyDecl
    //         Caller-supplied decl.
    //
    // Outputs:
    //     result: ConnectivityPolicyRuntime
    //         Return value from `connectivity_policy_from_decl`.
    //
    // Example:

    //     let result = spanda_connectivity_runtime::connectivity_policy_from_decl(decl);

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

/// Rewrite a GpsFix object's coordinates after applying GPS simulation faults.
pub fn apply_gps_reading_faults(
    reading: spanda_runtime::value::RuntimeValue,
    faults: &HashSet<String>,
    true_lat: f64,
    true_lon: f64,
    sim_time_ms: f64,
) -> spanda_runtime::value::RuntimeValue {
    // Description:
    //     Apply gps reading faults.
    //
    // Inputs:
    //     reading: spanda_runtime::value::RuntimeValue
    //         Caller-supplied reading.
    //     faults: &HashSet<String>
    //         Caller-supplied faults.
    //     rue_la: f64
    //         Caller-supplied rue la.
    //     rue_lon: f64
    //         Caller-supplied rue lon.
    //     sim_time_ms: f64
    //         Caller-supplied sim time ms.
    //
    // Outputs:
    //     result: spanda_runtime::value::RuntimeValue
    //         Return value from `apply_gps_reading_faults`.
    //
    // Example:

    //     let result = spanda_connectivity_runtime::apply_gps_reading_faults(reading, faults, rue_la, rue_lon, sim_time_ms);

    use spanda_ast::nodes::UnitKind;
    use spanda_runtime::value::RuntimeValue;
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
    let (lat, lon, fix_quality) =
        apply_gps_position_faults(faults, true_lat, true_lon, sim_time_ms);
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
pub fn connectivity_link_to_transport(link: &str) -> spanda_comm::TransportKind {
    // Description:
    //     Connectivity link to transport.
    //
    // Inputs:
    //     link: &str
    //         Caller-supplied link.
    //
    // Outputs:
    //     result: spanda_comm::TransportKind
    //         Return value from `connectivity_link_to_transport`.
    //
    // Example:

    //     let result = spanda_connectivity_runtime::connectivity_link_to_transport(link);

    use spanda_comm::TransportKind;
    use spanda_connectivity::ConnectivityTransport;
    match spanda_connectivity::connectivity_link_to_transport(link) {
        ConnectivityTransport::Mqtt => TransportKind::Mqtt,
        ConnectivityTransport::Dds => TransportKind::Dds,
        ConnectivityTransport::Websocket => TransportKind::Websocket,
        ConnectivityTransport::Ros2 => TransportKind::Ros2,
        ConnectivityTransport::Sim => TransportKind::Sim,
    }
}

/// Produce a [`GpsFix`]-shaped runtime object from lat/lon and optional metadata.
pub fn runtime_gps_fix(
    lat: f64,
    lon: f64,
    altitude: f64,
    fix_quality: f64,
) -> spanda_runtime::value::RuntimeValue {
    // Description:
    //     Runtime gps fix.
    //
    // Inputs:
    //     la: f64
    //         Caller-supplied la.
    //     lon: f64
    //         Caller-supplied lon.
    //     altitude: f64
    //         Caller-supplied altitude.
    //     fix_quality: f64
    //         Caller-supplied fix quality.
    //
    // Outputs:
    //     result: spanda_runtime::value::RuntimeValue
    //         Return value from `runtime_gps_fix`.
    //
    // Example:

    //     let result = spanda_connectivity_runtime::runtime_gps_fix(la, lon, altitude, fix_quality);

    use spanda_ast::nodes::UnitKind;
    use spanda_runtime::value::RuntimeValue;
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

/// Produce a [`SimIdentity`]-shaped runtime object for SIM/eSIM attestation simulation.
pub fn runtime_sim_identity(link: &str, attested: bool) -> spanda_runtime::value::RuntimeValue {
    // Description:
    //     Runtime sim identity.
    //
    // Inputs:
    //     link: &str
    //         Caller-supplied link.
    //     attested: bool
    //         Caller-supplied attested.
    //
    // Outputs:
    //     result: spanda_runtime::value::RuntimeValue
    //         Return value from `runtime_sim_identity`.
    //
    // Example:

    //     let result = spanda_connectivity_runtime::runtime_sim_identity(link, attested);

    use spanda_runtime::value::RuntimeValue;
    use std::collections::HashMap;
    let link_lower = link.to_ascii_lowercase();
    let iccid = format!(
        "89{:010}00000000000",
        link_lower
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
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
            ("iccid".into(), RuntimeValue::String { value: iccid }),
            (
                "carrier".into(),
                RuntimeValue::String {
                    value: carrier.into(),
                },
            ),
            ("esim".into(), RuntimeValue::Bool { value: esim }),
            ("attested".into(), RuntimeValue::Bool { value: attested }),
        ]),
    }
}

/// Apply a connectivity or positioning simulation fault to a hardware profile.
pub fn apply_connectivity_fault(
    mut profile: HardwareProfile,
    fault: &SimFaultDecl,
) -> HardwareProfile {
    // Description:
    //     Apply connectivity fault.
    //
    // Inputs:
    //     profile: HardwareProfile
    //         Caller-supplied profile.
    //     faul: &SimFaultDecl
    //         Caller-supplied faul.
    //
    // Outputs:
    //     result: HardwareProfile
    //         Return value from `apply_connectivity_fault`.
    //
    // Example:

    //     let result = spanda_connectivity_runtime::apply_connectivity_fault(profile, faul);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn haversine_zero_distance() {
        // Description:
        //     Haversine zero distance.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_connectivity_runtime::haversine_zero_distance();

        let d = haversine_m(30.0, -97.0, 30.0, -97.0);
        assert!(d.abs() < 0.01);
    }

    #[test]
    fn connectivity_link_to_transport_maps_wifi() {
        // Description:
        //     Connectivity link to transport maps wifi.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_connectivity_runtime::connectivity_link_to_transport_maps_wifi();

        use spanda_comm::TransportKind;
        assert_eq!(connectivity_link_to_transport("wifi"), TransportKind::Mqtt);
        assert_eq!(
            connectivity_link_to_transport("cellular"),
            TransportKind::Dds
        );
    }

    #[test]
    fn geofence_contains_center() {
        // Description:
        //     Geofence contains center.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_connectivity_runtime::geofence_contains_center();

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
        // Description:
        //     Gps spoofing offsets coordinates.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_connectivity_runtime::gps_spoofing_offsets_coordinates();

        use std::collections::HashSet;
        let faults = HashSet::from(["GpsSpoofing".to_string()]);
        let (lat, lon, fq) = apply_gps_position_faults(&faults, 30.0, -97.0, 0.0);
        assert!((lat - 30.009).abs() < 1e-6);
        assert!((lon - (-96.988)).abs() < 1e-6);
        assert!((fq - 0.3).abs() < 1e-6);
    }

    #[test]
    fn gps_drift_increases_with_sim_time() {
        // Description:
        //     Gps drift increases with sim time.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_connectivity_runtime::gps_drift_increases_with_sim_time();

        use std::collections::HashSet;
        let faults = HashSet::from(["GpsDrift".to_string()]);
        let (lat0, _, _) = apply_gps_position_faults(&faults, 30.0, -97.0, 0.0);
        let (lat1, _, _) = apply_gps_position_faults(&faults, 30.0, -97.0, 10_000.0);
        assert!(lat1 > lat0);
    }

    #[test]
    fn gps_drift_maps_to_trigger() {
        // Description:
        //     Gps drift maps to trigger.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_connectivity_runtime::gps_drift_maps_to_trigger();

        assert_eq!(fault_to_connectivity("GpsDrift"), Some(("gps", "drift")));
    }

    #[test]
    fn connectivity_link_to_transport_maps_satellite() {
        // Description:
        //     Connectivity link to transport maps satellite.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_connectivity_runtime::connectivity_link_to_transport_maps_satellite();

        assert_eq!(
            connectivity_link_to_transport("satellite"),
            spanda_comm::TransportKind::Websocket
        );
    }

    #[test]
    fn is_link_impaired_for_lte_outage_on_cellular() {
        // Description:
        //     Is link impaired for lte outage on cellular.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_connectivity_runtime::is_link_impaired_for_lte_outage_on_cellular();

        use std::collections::HashSet;
        let faults = HashSet::from(["LteOutage".to_string()]);
        assert!(is_link_impaired("cellular", &faults));
        assert!(!is_link_impaired("satellite", &faults));
    }

    #[test]
    fn runtime_sim_identity_cellular_attested() {
        // Description:
        //     Runtime sim identity cellular attested.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_connectivity_runtime::runtime_sim_identity_cellular_attested();

        use spanda_runtime::value::RuntimeValue;
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
        // Description:
        //     Runtime gps fix has gpsfix type.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_connectivity_runtime::runtime_gps_fix_has_gpsfix_type();

        let fix = runtime_gps_fix(30.0, -97.0, 150.0, 1.0);
        assert!(matches!(
            fix,
            spanda_runtime::value::RuntimeValue::Object { type_name, .. } if type_name == "GpsFix"
        ));
    }
}
