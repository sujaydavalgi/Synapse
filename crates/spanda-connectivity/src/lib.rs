//! Connectivity and positioning type catalogs extracted from Spanda core.
//!
pub mod adapter_bridge;
pub mod hardware_types;
pub mod runtime_sim;

pub use hardware_types::{CompatItem, CompatSeverity, HardwareProfile};

use serde::{Deserialize, Serialize};

pub use adapter_bridge::{invoke_nav2_bridge, invoke_slam_bridge};
pub use runtime_sim::{
    apply_gps_position_faults, fault_to_connectivity, geofence_contains,
    hardware_event_to_connectivity, haversine_m, is_link_impaired, GeofenceRuntime,
};

/// Requirement level for a connectivity channel in `requires_connectivity`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectivityRequirement {
    Required,
    Optional,
}

/// Suggested transport backend for a connectivity link name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectivityTransport {
    Mqtt,
    Dds,
    Websocket,
    Ros2,
    Sim,
}

/// Positioning and navigation native type names.
pub fn positioning_types() -> &'static [&'static str] {
    // Description:
    //     Positioning types.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: &'static [&'static str]
    //         Return value from `positioning_types`.
    //
    // Example:

    //     let result = spanda_connectivity::positioning_types();

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
    // Description:
    //     Connectivity types.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: &'static [&'static str]
    //         Return value from `connectivity_types`.
    //
    // Example:

    //     let result = spanda_connectivity::connectivity_types();

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
    // Description:
    //     Connectivity options.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: &'static [&'static str]
    //         Return value from `connectivity_options`.
    //
    // Example:

    //     let result = spanda_connectivity::connectivity_options();

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
    // Description:
    //     Connectivity key to profile tokens.
    //
    // Inputs:
    //     key: &str
    //         Caller-supplied key.
    //
    // Outputs:
    //     result: Vec<&'static str>
    //         Return value from `connectivity_key_to_profile_tokens`.
    //
    // Example:

    //     let result = spanda_connectivity::connectivity_key_to_profile_tokens(key);

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
    // Description:
    //     Connectivity faults.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: &'static [&'static str]
    //         Return value from `connectivity_faults`.
    //
    // Example:

    //     let result = spanda_connectivity::connectivity_faults();

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
    // Description:
    //     Connectivity capabilities.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: &'static [&'static str]
    //         Return value from `connectivity_capabilities`.
    //
    // Example:

    //     let result = spanda_connectivity::connectivity_capabilities();

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

/// Map an active connectivity link name to the default transport backend.
pub fn connectivity_link_to_transport(link: &str) -> ConnectivityTransport {
    // Description:
    //     Connectivity link to transport.
    //
    // Inputs:
    //     link: &str
    //         Caller-supplied link.
    //
    // Outputs:
    //     result: ConnectivityTransport
    //         Return value from `connectivity_link_to_transport`.
    //
    // Example:

    //     let result = spanda_connectivity::connectivity_link_to_transport(link);

    match link.to_ascii_lowercase().as_str() {
        "wifi" => ConnectivityTransport::Mqtt,
        "cellular" | "lte" | "4g" | "fiveg" | "5g" => ConnectivityTransport::Dds,
        "bluetooth" | "ble" => ConnectivityTransport::Websocket,
        "ethernet" => ConnectivityTransport::Ros2,
        "satellite" => ConnectivityTransport::Websocket,
        "network" => ConnectivityTransport::Sim,
        _ => ConnectivityTransport::Sim,
    }
}

pub fn is_cellular_link(link: &str) -> bool {
    // Description:
    //     Is cellular link.
    //
    // Inputs:
    //     link: &str
    //         Caller-supplied link.
    //
    // Outputs:
    //     result: bool
    //         Return value from `is_cellular_link`.
    //
    // Example:

    //     let result = spanda_connectivity::is_cellular_link(link);

    matches!(
        link.to_ascii_lowercase().as_str(),
        "cellular" | "lte" | "4g" | "fourg" | "fiveg" | "5g"
    )
}

pub fn is_satellite_link(link: &str) -> bool {
    // Description:
    //     Is satellite link.
    //
    // Inputs:
    //     link: &str
    //         Caller-supplied link.
    //
    // Outputs:
    //     result: bool
    //         Return value from `is_satellite_link`.
    //
    // Example:

    //     let result = spanda_connectivity::is_satellite_link(link);

    link.eq_ignore_ascii_case("satellite")
}

pub fn is_modem_bearer(link: &str) -> bool {
    // Description:
    //     Is modem bearer.
    //
    // Inputs:
    //     link: &str
    //         Caller-supplied link.
    //
    // Outputs:
    //     result: bool
    //         Return value from `is_modem_bearer`.
    //
    // Example:

    //     let result = spanda_connectivity::is_modem_bearer(link);

    is_cellular_link(link) || is_satellite_link(link)
}

pub fn is_wifi_link(link: &str) -> bool {
    // Description:
    //     Is wifi link.
    //
    // Inputs:
    //     link: &str
    //         Caller-supplied link.
    //
    // Outputs:
    //     result: bool
    //         Return value from `is_wifi_link`.
    //
    // Example:

    //     let result = spanda_connectivity::is_wifi_link(link);

    matches!(
        link.to_ascii_lowercase().as_str(),
        "wifi" | "wi-fi" | "wifi6"
    )
}
