//! Package-backed device discovery transport contract.
//!
use crate::device_identity::{DiscoveryMatch, NetworkHostProbe};
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
        let Some(subnet) = options.subnet.as_deref() else {
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

/// Mock mDNS transport for package contract tests (live backend in `spanda-discovery-mdns`).
pub struct MockMdnsDiscoveryTransport;

impl DeviceDiscoveryTransport for MockMdnsDiscoveryTransport {
    fn transport_name(&self) -> &'static str {
        "mdns"
    }

    fn discover(&self, _options: &DiscoveryOptions) -> Result<DiscoveryTransportResult, String> {
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

macro_rules! stub_transport {
    ($name:ident, $transport:expr, $device_id:expr) => {
        pub struct $name;
        impl DeviceDiscoveryTransport for $name {
            fn transport_name(&self) -> &'static str {
                $transport
            }
            fn discover(
                &self,
                _options: &DiscoveryOptions,
            ) -> Result<DiscoveryTransportResult, String> {
                Ok(DiscoveryTransportResult {
                    transport: self.transport_name().into(),
                    matches: vec![DiscoveryMatch {
                        device_id: $device_id.into(),
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

stub_transport!(MockBleDiscoveryTransport, "ble", "ble-stub-device");
stub_transport!(MockUsbDiscoveryTransport, "usb", "usb-stub-device");
stub_transport!(MockCanDiscoveryTransport, "can", "can-stub-device");
stub_transport!(MockMqttDiscoveryTransport, "mqtt", "mqtt-stub-device");
stub_transport!(MockRos2DiscoveryTransport, "ros2", "ros2-stub-device");

/// Resolve a discovery transport by name (built-in stubs; packages extend via registry).
pub fn discovery_transport_by_name(name: &str) -> Option<Box<dyn DeviceDiscoveryTransport>> {
    match name.to_ascii_lowercase().as_str() {
        "subnet" => Some(Box::new(SubnetDiscoveryTransport)),
        "mdns" => Some(Box::new(MockMdnsDiscoveryTransport)),
        "ble" | "bluetooth" => Some(Box::new(MockBleDiscoveryTransport)),
        "usb" => Some(Box::new(MockUsbDiscoveryTransport)),
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
