//! Network and bus identity records for physical-to-logical device mapping.
//!
use crate::device_tree::{DeviceNode, DeviceTree};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::str::FromStr;
use std::time::Duration;

/// Trust posture for a configured device endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    #[default]
    Unknown,
    Unverified,
    Verified,
    Trusted,
    Restricted,
}

impl TrustLevel {
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "unverified" => Self::Unverified,
            "verified" => Self::Verified,
            "trusted" => Self::Trusted,
            "restricted" => Self::Restricted,
            _ => Self::Unknown,
        }
    }

    /// Whether trust is sufficient for operational mission use.
    pub fn is_operational(self) -> bool {
        matches!(self, Self::Verified | Self::Trusted)
    }
}

/// Physical device identity with network, bus, and security metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DeviceIdentityRecord {
    pub id: String,
    #[serde(rename = "type", default)]
    pub device_type: String,
    #[serde(default)]
    pub logical_name: Option<String>,
    #[serde(default)]
    pub serial: Option<String>,
    #[serde(default, alias = "mac")]
    pub mac_address: Option<String>,
    #[serde(default, alias = "ip")]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub dns_name: Option<String>,
    #[serde(default)]
    pub mdns_name: Option<String>,
    #[serde(default, alias = "endpoint")]
    pub endpoint_url: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub bus: Option<String>,
    #[serde(default)]
    pub can_id: Option<String>,
    #[serde(default)]
    pub usb_path: Option<String>,
    #[serde(default)]
    pub pci_path: Option<String>,
    #[serde(default)]
    pub bluetooth_address: Option<String>,
    #[serde(default)]
    pub ble_uuid: Option<String>,
    #[serde(default)]
    pub cellular_imei: Option<String>,
    #[serde(default)]
    pub sim_iccid: Option<String>,
    #[serde(default)]
    pub gps_device_id: Option<String>,
    #[serde(default, alias = "firmware")]
    pub firmware_version: Option<String>,
    #[serde(default)]
    pub hardware_revision: Option<String>,
    #[serde(default)]
    pub security_identity: Option<String>,
    #[serde(default)]
    pub certificate_fingerprint: Option<String>,
    #[serde(default)]
    pub trust_level: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub robot_id: Option<String>,
    #[serde(default)]
    pub redundant_group: Option<String>,
    #[serde(default)]
    pub failover_priority: Option<u32>,
    #[serde(default)]
    pub mount: Option<String>,
    #[serde(default)]
    pub lifecycle_state: Option<String>,
    #[serde(default)]
    pub assigned_robot: Option<String>,
    #[serde(default)]
    pub last_seen_ms: Option<f64>,
    #[serde(default)]
    pub provisioning_id: Option<String>,
    #[serde(default)]
    pub health_status: Option<String>,
    #[serde(default)]
    pub last_heartbeat_ms: Option<f64>,
    #[serde(default)]
    pub calibration_status: Option<String>,
    #[serde(default)]
    pub calibration_expiry_ms: Option<f64>,
    #[serde(default)]
    pub last_firmware_update_ms: Option<f64>,
    #[serde(default)]
    pub last_self_test_ms: Option<f64>,
    #[serde(default)]
    pub min_firmware_version: Option<String>,
}

impl DeviceIdentityRecord {
    pub fn trust_level_enum(&self) -> TrustLevel {
        self.trust_level
            .as_deref()
            .map(TrustLevel::parse)
            .unwrap_or(TrustLevel::Unknown)
    }

    pub fn is_networked(&self) -> bool {
        self.ip_address.is_some()
            || self.hostname.is_some()
            || self.dns_name.is_some()
            || self.mdns_name.is_some()
            || self.endpoint_url.is_some()
            || self.protocol.as_deref().is_some_and(|p| {
                matches!(
                    p.to_ascii_lowercase().as_str(),
                    "http" | "https" | "rtsp" | "mqtt" | "ws" | "wss" | "tcp" | "udp"
                )
            })
    }

    pub fn is_remote_actuator(&self) -> bool {
        let dtype = self.device_type.to_ascii_lowercase();
        let is_actuator = dtype.contains("drive")
            || dtype.contains("actuator")
            || dtype.contains("motor")
            || self
                .capabilities
                .iter()
                .any(|c| c == "move" || c == "stop" || c == "emergency_stop");
        is_actuator && self.is_networked()
    }

    pub fn endpoint_is_insecure(&self) -> bool {
        let Some(ref url) = self.endpoint_url else {
            return false;
        };
        let lower = url.to_ascii_lowercase();
        lower.starts_with("http://")
            || lower.starts_with("ws://")
            || lower.starts_with("mqtt://")
            || lower.starts_with("rtsp://")
    }

    pub fn normalized_mac(&self) -> Option<String> {
        self.mac_address
            .as_ref()
            .map(|m| m.to_ascii_uppercase().replace('-', ":"))
    }
}

/// Registry of physical device identities from flat `[[devices]]` and fleet tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DeviceRegistry {
    pub devices: Vec<DeviceIdentityRecord>,
}

impl DeviceRegistry {
    pub fn from_resolved_parts(tree: &DeviceTree, raw: &toml::Value) -> Self {
        let mut by_id: HashMap<String, DeviceIdentityRecord> = HashMap::new();
        for record in parse_flat_devices(raw) {
            by_id.insert(record.id.clone(), record);
        }
        for record in records_from_device_tree(tree) {
            by_id
                .entry(record.id.clone())
                .and_modify(|existing| merge_identity(existing, &record))
                .or_insert(record);
        }
        let mut devices: Vec<DeviceIdentityRecord> = by_id.into_values().collect();
        devices.sort_by(|a, b| a.id.cmp(&b.id));
        Self { devices }
    }

    pub fn get(&self, id: &str) -> Option<&DeviceIdentityRecord> {
        self.devices.iter().find(|d| d.id == id)
    }

    pub fn by_logical_name(&self, logical: &str) -> Vec<&DeviceIdentityRecord> {
        self.devices
            .iter()
            .filter(|d| d.logical_name.as_deref() == Some(logical))
            .collect()
    }

    pub fn network_devices(&self) -> Vec<&DeviceIdentityRecord> {
        self.devices.iter().filter(|d| d.is_networked()).collect()
    }
}

fn merge_identity(target: &mut DeviceIdentityRecord, overlay: &DeviceIdentityRecord) {
    macro_rules! fill {
        ($field:ident) => {
            if target.$field.is_none() {
                target.$field = overlay.$field.clone();
            }
        };
    }
    fill!(logical_name);
    fill!(serial);
    fill!(mac_address);
    fill!(ip_address);
    fill!(hostname);
    fill!(dns_name);
    fill!(mdns_name);
    fill!(endpoint_url);
    fill!(protocol);
    fill!(port);
    fill!(bus);
    fill!(can_id);
    fill!(usb_path);
    fill!(pci_path);
    fill!(bluetooth_address);
    fill!(ble_uuid);
    fill!(cellular_imei);
    fill!(sim_iccid);
    fill!(gps_device_id);
    fill!(firmware_version);
    fill!(hardware_revision);
    fill!(security_identity);
    fill!(certificate_fingerprint);
    fill!(trust_level);
    fill!(provider);
    fill!(robot_id);
    fill!(redundant_group);
    fill!(failover_priority);
    fill!(mount);
    fill!(lifecycle_state);
    fill!(assigned_robot);
    fill!(last_seen_ms);
    fill!(provisioning_id);
    fill!(health_status);
    fill!(last_heartbeat_ms);
    fill!(calibration_status);
    fill!(calibration_expiry_ms);
    fill!(last_firmware_update_ms);
    fill!(last_self_test_ms);
    fill!(min_firmware_version);
    if target.capabilities.is_empty() {
        target.capabilities = overlay.capabilities.clone();
    }
    if target.device_type.is_empty() {
        target.device_type = overlay.device_type.clone();
    }
}

pub fn parse_flat_devices(value: &toml::Value) -> Vec<DeviceIdentityRecord> {
    let Some(arr) = value.get("devices").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|entry| entry.clone().try_into().ok())
        .collect()
}

pub fn records_from_device_tree(tree: &DeviceTree) -> Vec<DeviceIdentityRecord> {
    let mut out = Vec::new();
    for (robot, _compute, device) in tree.all_devices() {
        out.push(identity_from_device_node(&robot.id, device));
    }
    out
}

pub fn identity_from_device_node(robot_id: &str, device: &DeviceNode) -> DeviceIdentityRecord {
    DeviceIdentityRecord {
        id: device.id.clone(),
        device_type: device.device_type.clone(),
        logical_name: device.logical_name.clone(),
        serial: device.serial.clone(),
        mac_address: device.mac_address.clone(),
        ip_address: device.ip_address.clone(),
        hostname: device.hostname.clone(),
        dns_name: device.dns_name.clone(),
        mdns_name: device.mdns_name.clone(),
        endpoint_url: device.endpoint_url.clone(),
        protocol: device.protocol.clone(),
        port: device
            .network_port
            .or(device.port.as_ref().and_then(|p| p.parse().ok())),
        bus: device.bus.clone().or(device.port.clone()),
        can_id: device.can_id.clone(),
        usb_path: device.usb_path.clone(),
        pci_path: device.pci_path.clone(),
        bluetooth_address: device.bluetooth_address.clone(),
        ble_uuid: device.ble_uuid.clone(),
        cellular_imei: device.cellular_imei.clone(),
        sim_iccid: device.sim_iccid.clone(),
        gps_device_id: device.gps_device_id.clone(),
        firmware_version: device.firmware_version.clone().or(device.firmware.clone()),
        hardware_revision: device.hardware_revision.clone(),
        security_identity: device.security_identity.clone().or(device.identity.clone()),
        certificate_fingerprint: device.certificate_fingerprint.clone(),
        trust_level: device.trust_level.clone(),
        provider: device.provider.clone(),
        capabilities: device.capabilities.clone(),
        robot_id: Some(robot_id.into()),
        redundant_group: device.redundant_group.clone(),
        failover_priority: device.failover_priority,
        mount: device.mount.clone(),
        lifecycle_state: device.lifecycle_state.clone(),
        assigned_robot: device.assigned_robot.clone(),
        last_seen_ms: None,
        provisioning_id: None,
        health_status: device.health_status.clone(),
        last_heartbeat_ms: device.last_heartbeat_ms,
        calibration_status: device.calibration_status.clone(),
        calibration_expiry_ms: device.calibration_expiry_ms,
        last_firmware_update_ms: device.last_firmware_update_ms,
        last_self_test_ms: device.last_self_test_ms,
        min_firmware_version: device.min_firmware_version.clone(),
    }
}

/// Parsed IPv4 subnet for network scanning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ipv4Subnet {
    pub network: Ipv4Addr,
    pub prefix_len: u8,
}

impl Ipv4Subnet {
    pub fn parse(cidr: &str) -> Option<Self> {
        let (addr, prefix) = cidr.split_once('/')?;
        let network = Ipv4Addr::from_str(addr.trim()).ok()?;
        let prefix_len = prefix.trim().parse::<u8>().ok()?;
        if prefix_len > 32 {
            return None;
        }
        Some(Self {
            network,
            prefix_len,
        })
    }

    pub fn hosts(&self) -> Vec<Ipv4Addr> {
        let base = u32::from(self.network);
        let host_bits = 32u32.saturating_sub(self.prefix_len as u32);
        let count = 1u32.checked_shl(host_bits).unwrap_or(0).min(256);
        let mut hosts = Vec::new();
        for offset in 1..count.saturating_sub(1).max(1) {
            if hosts.len() >= 254 {
                break;
            }
            hosts.push(Ipv4Addr::from(base + offset));
        }
        hosts
    }
}

/// Result of probing a single host on the network.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkHostProbe {
    pub ip: String,
    pub reachable: bool,
    pub open_ports: Vec<u16>,
    pub latency_ms: Option<u64>,
}

/// Scan a subnet for reachable hosts (TCP connect probe).
pub fn scan_subnet(cidr: &str, ports: &[u16], timeout_ms: u64) -> Vec<NetworkHostProbe> {
    let Some(subnet) = Ipv4Subnet::parse(cidr) else {
        return Vec::new();
    };
    let timeout = Duration::from_millis(timeout_ms);
    let probe_ports: Vec<u16> = if ports.is_empty() {
        vec![80, 443, 554, 1883, 22]
    } else {
        ports.to_vec()
    };
    let mut results = Vec::new();
    for host in subnet.hosts() {
        let mut open_ports = Vec::new();
        let started = std::time::Instant::now();
        for port in &probe_ports {
            let addr = SocketAddr::from((host, *port));
            if TcpStream::connect_timeout(&addr, timeout).is_ok() {
                open_ports.push(*port);
            }
        }
        let reachable = !open_ports.is_empty();
        results.push(NetworkHostProbe {
            ip: host.to_string(),
            reachable,
            open_ports,
            latency_ms: if reachable {
                Some(started.elapsed().as_millis() as u64)
            } else {
                None
            },
        });
    }
    results
}

/// Match discovered hosts to configured devices by IP or hostname.
pub fn discover_matches(
    registry: &DeviceRegistry,
    probes: &[NetworkHostProbe],
) -> Vec<DiscoveryMatch> {
    let mut matches = Vec::new();
    for device in &registry.devices {
        let Some(ref ip) = device.ip_address else {
            continue;
        };
        if let Some(probe) = probes.iter().find(|p| p.ip == *ip) {
            matches.push(DiscoveryMatch {
                device_id: device.id.clone(),
                logical_name: device.logical_name.clone(),
                configured_ip: ip.clone(),
                probe: probe.clone(),
                matched_by: "ip_address".into(),
            });
        }
    }
    matches
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscoveryMatch {
    pub device_id: String,
    pub logical_name: Option<String>,
    pub configured_ip: String,
    pub probe: NetworkHostProbe,
    pub matched_by: String,
}

pub fn provider_protocol_mismatch(provider: &str, protocol: &str) -> bool {
    let expected = expected_protocols_for_provider(provider);
    if expected.is_empty() {
        return false;
    }
    let proto = protocol.to_ascii_lowercase();
    !expected.iter().any(|e| *e == proto)
}

fn expected_protocols_for_provider(provider: &str) -> Vec<&'static str> {
    let p = provider.to_ascii_lowercase();
    if p.contains("camera") {
        vec!["rtsp", "http", "https", "onvif"]
    } else if p.contains("gps") {
        vec!["nmea", "serial", "tcp", "udp"]
    } else if p.contains("canbus") || p.contains("can") {
        vec!["can", "socketcan"]
    } else if p.contains("lidar") {
        vec!["serial", "udp", "tcp", "velodyne"]
    } else if p.contains("mqtt") {
        vec!["mqtt", "mqtts", "tcp"]
    } else if p.contains("modbus") {
        vec!["modbus", "tcp", "rtu"]
    } else {
        vec![]
    }
}

pub fn check_ip_reachable(ip: &str, port: u16, timeout_ms: u64) -> bool {
    let Ok(addr) = ip.parse::<Ipv4Addr>() else {
        return false;
    };
    TcpStream::connect_timeout(
        &SocketAddr::from((addr, port)),
        Duration::from_millis(timeout_ms),
    )
    .is_ok()
}

pub fn logical_name_index(registry: &DeviceRegistry) -> HashMap<String, Vec<String>> {
    let mut index: HashMap<String, Vec<String>> = HashMap::new();
    for device in &registry.devices {
        if let Some(ref logical) = device.logical_name {
            index
                .entry(logical.clone())
                .or_default()
                .push(device.id.clone());
        }
    }
    index
}

pub fn redundant_groups(registry: &DeviceRegistry) -> HashMap<String, Vec<&DeviceIdentityRecord>> {
    let mut groups: HashMap<String, Vec<&DeviceIdentityRecord>> = HashMap::new();
    for device in &registry.devices {
        if let Some(ref group) = device.redundant_group {
            groups.entry(group.clone()).or_default().push(device);
        }
    }
    groups
}

/// Identity anomaly detected during discovery or validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentityAnomaly {
    pub device_id: String,
    pub anomaly_type: String,
    pub message: String,
}

/// Detect duplicate identities, unknown certificates, and insecure endpoints.
pub fn detect_identity_anomalies(registry: &DeviceRegistry) -> Vec<IdentityAnomaly> {
    let mut anomalies = Vec::new();
    let mut ips: HashMap<String, String> = HashMap::new();
    let mut macs: HashMap<String, String> = HashMap::new();
    let mut serials: HashMap<String, String> = HashMap::new();
    for device in &registry.devices {
        if let Some(ref ip) = device.ip_address {
            if let Some(other) = ips.insert(ip.clone(), device.id.clone()) {
                anomalies.push(IdentityAnomaly {
                    device_id: device.id.clone(),
                    anomaly_type: "duplicate_ip".into(),
                    message: format!("duplicate IP '{ip}' shared with '{other}'"),
                });
            }
        }
        if let Some(ref mac) = device.normalized_mac() {
            if let Some(other) = macs.insert(mac.clone(), device.id.clone()) {
                anomalies.push(IdentityAnomaly {
                    device_id: device.id.clone(),
                    anomaly_type: "duplicate_mac".into(),
                    message: format!("duplicate MAC '{mac}' shared with '{other}'"),
                });
            }
        }
        if let Some(ref serial) = device.serial {
            if let Some(other) = serials.insert(serial.clone(), device.id.clone()) {
                anomalies.push(IdentityAnomaly {
                    device_id: device.id.clone(),
                    anomaly_type: "duplicate_serial".into(),
                    message: format!("duplicate serial '{serial}' shared with '{other}'"),
                });
            }
        }
        if device.certificate_fingerprint.is_none()
            && device.is_networked()
            && device.security_identity.is_some()
        {
            anomalies.push(IdentityAnomaly {
                device_id: device.id.clone(),
                anomaly_type: "unknown_certificate".into(),
                message: "security_identity declared without certificate_fingerprint".into(),
            });
        }
        if device.endpoint_is_insecure() {
            anomalies.push(IdentityAnomaly {
                device_id: device.id.clone(),
                anomaly_type: "insecure_endpoint".into(),
                message: "endpoint uses insecure transport scheme".into(),
            });
        }
        if let Some(ref min) = device.min_firmware_version {
            if device
                .firmware_version
                .as_deref()
                .map(|v| v < min.as_str())
                .unwrap_or(true)
            {
                anomalies.push(IdentityAnomaly {
                    device_id: device.id.clone(),
                    anomaly_type: "unsupported_firmware".into(),
                    message: format!(
                        "firmware {:?} below minimum '{min}'",
                        device.firmware_version
                    ),
                });
            }
        }
    }
    anomalies
}

pub fn traceability_rows(registry: &DeviceRegistry) -> Vec<TraceabilityRow> {
    registry
        .devices
        .iter()
        .map(|d| TraceabilityRow {
            device_id: d.id.clone(),
            logical_name: d.logical_name.clone(),
            device_type: d.device_type.clone(),
            provider: d.provider.clone(),
            ip_address: d.ip_address.clone(),
            mac_address: d.mac_address.clone(),
            serial: d.serial.clone(),
            endpoint_url: d.endpoint_url.clone(),
            trust_level: d.trust_level.clone(),
            robot_id: d.robot_id.clone(),
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceabilityRow {
    pub device_id: String,
    pub logical_name: Option<String>,
    pub device_type: String,
    pub provider: Option<String>,
    pub ip_address: Option<String>,
    pub mac_address: Option<String>,
    pub serial: Option<String>,
    pub endpoint_url: Option<String>,
    pub trust_level: Option<String>,
    pub robot_id: Option<String>,
}
