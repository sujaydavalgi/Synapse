//! Package-backed device discovery transport contract.
//!
use crate::discovery_registry::wrap_with_registry_package;
use crate::device_identity::{DiscoveryMatch, DeviceIdentityRecord, NetworkHostProbe};
use crate::discovery_live::{
    default_discovery_subnet, probe_ble, probe_can, probe_cellular, probe_mdns, probe_mqtt,
    probe_ros2, probe_serial, probe_usb, probe_wifi,
};
use serde::{Deserialize, Serialize};

/// Options passed to discovery transports (mDNS, BLE, subnet scan, …).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscoveryOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub transports: Vec<String>,
}

/// Result envelope from a discovery transport.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscoveryTransportResult {
    pub transport: String,
    pub matches: Vec<DiscoveryMatch>,
}

/// Contract implemented by optional discovery packages (`spanda-discovery-mdns`, …).
pub trait DeviceDiscoveryTransport: Send + Sync {
    fn transport_name(&self) -> &'static str;
    fn discover(&self, options: &DiscoveryOptions) -> Result<DiscoveryTransportResult, String>;
}

/// Built-in subnet discovery using the core network scanner.
pub struct SubnetDiscoveryTransport;

impl DeviceDiscoveryTransport for SubnetDiscoveryTransport {
    fn transport_name(&self) -> &'static str {
        "subnet"
    }

    fn discover(&self, options: &DiscoveryOptions) -> Result<DiscoveryTransportResult, String> {
        let default_subnet = default_discovery_subnet();
        let subnet = options
            .subnet
            .as_deref()
            .or(default_subnet.as_deref());
        let Some(subnet) = subnet else {
            return Ok(DiscoveryTransportResult {
                transport: self.transport_name().into(),
                matches: Vec::new(),
            });
        };
        let timeout = options.timeout_ms.unwrap_or(200);
        let hosts = crate::device_identity::scan_subnet(subnet, &[80, 443, 554], timeout);
        let matches = hosts
            .into_iter()
            .map(|probe| DiscoveryMatch {
                device_id: format!("discovered-{}", probe.ip),
                logical_name: None,
                configured_ip: probe.ip.clone(),
                probe,
                matched_by: self.transport_name().into(),
            })
            .collect();
        Ok(DiscoveryTransportResult {
            transport: self.transport_name().into(),
            matches,
        })
    }
}

/// Mock mDNS transport with host-backed probe fallback.
pub struct MockMdnsDiscoveryTransport;

impl DeviceDiscoveryTransport for MockMdnsDiscoveryTransport {
    fn transport_name(&self) -> &'static str {
        "mdns"
    }

    fn discover(&self, options: &DiscoveryOptions) -> Result<DiscoveryTransportResult, String> {
        let timeout = options.timeout_ms.unwrap_or(2000);
        let live = probe_mdns(timeout);
        if !live.is_empty() {
            return Ok(DiscoveryTransportResult {
                transport: self.transport_name().into(),
                matches: live,
            });
        }
        if std::env::var("SPANDA_DISCOVERY_NO_STUB")
            .ok()
            .is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        {
            return Ok(DiscoveryTransportResult {
                transport: self.transport_name().into(),
                matches: Vec::new(),
            });
        }
        Ok(DiscoveryTransportResult {
            transport: self.transport_name().into(),
            matches: vec![DiscoveryMatch {
                device_id: "mdns-stub-robot".into(),
                logical_name: Some("_spanda._tcp.local".into()),
                configured_ip: "0.0.0.0".into(),
                probe: NetworkHostProbe {
                    ip: "0.0.0.0".into(),
                    reachable: true,
                    open_ports: vec![],
                    latency_ms: None,
                },
                matched_by: self.transport_name().into(),
            }],
        })
    }
}

macro_rules! live_transport {
    ($name:ident, $transport:expr, $probe:expr, $stub_id:expr) => {
        pub struct $name;
        impl DeviceDiscoveryTransport for $name {
            fn transport_name(&self) -> &'static str {
                $transport
            }
            fn discover(
                &self,
                options: &DiscoveryOptions,
            ) -> Result<DiscoveryTransportResult, String> {
                let live = $probe(options);
                if !live.is_empty() {
                    return Ok(DiscoveryTransportResult {
                        transport: self.transport_name().into(),
                        matches: live,
                    });
                }
                if std::env::var("SPANDA_DISCOVERY_NO_STUB")
                    .ok()
                    .is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
                {
                    return Ok(DiscoveryTransportResult {
                        transport: self.transport_name().into(),
                        matches: Vec::new(),
                    });
                }
                Ok(DiscoveryTransportResult {
                    transport: self.transport_name().into(),
                    matches: vec![DiscoveryMatch {
                        device_id: $stub_id.into(),
                        logical_name: None,
                        configured_ip: "stub".into(),
                        probe: NetworkHostProbe {
                            ip: "stub".into(),
                            reachable: true,
                            open_ports: vec![],
                            latency_ms: None,
                        },
                        matched_by: self.transport_name().into(),
                    }],
                })
            }
        }
    };
}

fn probe_ble_options(_options: &DiscoveryOptions) -> Vec<DiscoveryMatch> {
    probe_ble()
}

fn probe_usb_options(_options: &DiscoveryOptions) -> Vec<DiscoveryMatch> {
    probe_usb()
}

fn probe_can_options(_options: &DiscoveryOptions) -> Vec<DiscoveryMatch> {
    probe_can()
}

fn probe_mqtt_options(options: &DiscoveryOptions) -> Vec<DiscoveryMatch> {
    probe_mqtt(options.timeout_ms.unwrap_or(500))
}

fn probe_ros2_options(options: &DiscoveryOptions) -> Vec<DiscoveryMatch> {
    probe_ros2(options.timeout_ms.unwrap_or(2000))
}

fn probe_wifi_options(options: &DiscoveryOptions) -> Vec<DiscoveryMatch> {
    probe_wifi(options.timeout_ms.unwrap_or(500))
}

fn probe_cellular_options(_options: &DiscoveryOptions) -> Vec<DiscoveryMatch> {
    probe_cellular()
}

fn probe_serial_options(_options: &DiscoveryOptions) -> Vec<DiscoveryMatch> {
    probe_serial()
}

live_transport!(MockBleDiscoveryTransport, "ble", probe_ble_options, "ble-stub-device");
live_transport!(MockUsbDiscoveryTransport, "usb", probe_usb_options, "usb-stub-device");
live_transport!(MockCanDiscoveryTransport, "can", probe_can_options, "can-stub-device");
live_transport!(MockMqttDiscoveryTransport, "mqtt", probe_mqtt_options, "mqtt-stub-device");
live_transport!(MockRos2DiscoveryTransport, "ros2", probe_ros2_options, "ros2-stub-device");
live_transport!(MockWifiDiscoveryTransport, "wifi", probe_wifi_options, "wifi-stub-device");
live_transport!(
    MockCellularDiscoveryTransport,
    "cellular",
    probe_cellular_options,
    "cellular-stub-device"
);
live_transport!(
    MockSerialDiscoveryTransport,
    "serial",
    probe_serial_options,
    "serial-stub-device"
);

/// Resolve a discovery transport by name (built-in stubs; packages extend via registry).
pub fn discovery_transport_by_name(name: &str) -> Option<Box<dyn DeviceDiscoveryTransport>> {
    match name.to_ascii_lowercase().as_str() {
        "subnet" => Some(Box::new(SubnetDiscoveryTransport)),
        "mdns" => Some(wrap_with_registry_package(
            "mdns",
            Box::new(MockMdnsDiscoveryTransport),
        )),
        "ble" | "bluetooth" => Some(wrap_with_registry_package(
            "ble",
            Box::new(MockBleDiscoveryTransport),
        )),
        "usb" => Some(wrap_with_registry_package(
            "usb",
            Box::new(MockUsbDiscoveryTransport),
        )),
        "wifi" => Some(wrap_with_registry_package(
            "wifi",
            Box::new(MockWifiDiscoveryTransport),
        )),
        "cellular" | "lte" | "5g" => Some(wrap_with_registry_package(
            "cellular",
            Box::new(MockCellularDiscoveryTransport),
        )),
        "serial" => Some(wrap_with_registry_package(
            "serial",
            Box::new(MockSerialDiscoveryTransport),
        )),
        "can" => Some(Box::new(MockCanDiscoveryTransport)),
        "mqtt" => Some(Box::new(MockMqttDiscoveryTransport)),
        "ros2" | "dds" => Some(Box::new(MockRos2DiscoveryTransport)),
        _ => None,
    }
}

/// Run discovery across one or more named transports.
pub fn run_discovery_transports(
    options: &DiscoveryOptions,
) -> Vec<Result<DiscoveryTransportResult, String>> {
    let names: Vec<String> = if options.transports.is_empty() {
        vec!["subnet".into()]
    } else {
        options.transports.clone()
    };
    names
        .iter()
        .map(|name| {
            discovery_transport_by_name(name)
                .ok_or_else(|| format!("unknown discovery transport '{name}'"))
                .and_then(|t| t.discover(options))
        })
        .collect()
}

/// Build a registry record from a discovery match for pool ingestion.
pub fn discovery_match_to_record(match_entry: &DiscoveryMatch) -> DeviceIdentityRecord {
    let mut record = DeviceIdentityRecord {
        id: match_entry.device_id.clone(),
        device_type: match_entry.matched_by.clone(),
        logical_name: match_entry.logical_name.clone(),
        ip_address: Some(match_entry.configured_ip.clone()),
        ..Default::default()
    };
    if record.device_type == "usb" {
        record.usb_path = Some(match_entry.configured_ip.clone());
    }
    if record.device_type == "ble" {
        record.bluetooth_address = Some(match_entry.configured_ip.clone());
    }
    if record.device_type == "can" {
        record.bus = match_entry.logical_name.clone();
    }
    record
}

/// Register all discovery matches into a device registry.
pub fn ingest_discovery_matches(
    registry: &mut crate::device_identity::DeviceRegistry,
    results: &[DiscoveryTransportResult],
) -> Vec<crate::device_operations::DeviceOperationResult> {
    let mut registered = Vec::new();
    for result in results {
        for match_entry in &result.matches {
            registered.push(registry.register_discovered(discovery_match_to_record(match_entry)));
        }
    }
    registered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_mdns_returns_stub_match() {
        let transport = MockMdnsDiscoveryTransport;
        let result = transport.discover(&DiscoveryOptions::default()).unwrap();
        assert_eq!(result.matches.len(), 1);
    }
}
