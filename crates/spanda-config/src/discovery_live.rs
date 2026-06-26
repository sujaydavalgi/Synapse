//! Host-backed discovery probes for BLE, USB, CAN, MQTT, mDNS, and ROS2.
//!
use crate::device_identity::{DiscoveryMatch, NetworkHostProbe};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, UdpSocket};
use std::process::{Command, Stdio};
use std::time::Duration;

fn run_command_output(program: &str, args: &[&str], timeout_ms: u64) -> Option<String> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    std::thread::sleep(Duration::from_millis(timeout_ms.min(5000)));
    let _ = child.kill();
    let output = child.wait_with_output().ok()?;
    if output.stdout.is_empty() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn discovery_match(
    device_id: &str,
    logical_name: Option<&str>,
    ip: &str,
    transport: &str,
) -> DiscoveryMatch {
    DiscoveryMatch {
        device_id: device_id.into(),
        logical_name: logical_name.map(str::to_string),
        configured_ip: ip.into(),
        probe: NetworkHostProbe {
            ip: ip.into(),
            reachable: true,
            open_ports: vec![],
            latency_ms: None,
        },
        matched_by: transport.into(),
    }
}

/// Default subnet CIDR for discovery when callers omit `subnet`.
pub fn default_discovery_subnet() -> Option<String> {
    std::env::var("SPANDA_DISCOVERY_SUBNET")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(detect_local_subnet)
}

fn detect_local_subnet() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let local = socket.local_addr().ok()?.ip();
    if let IpAddr::V4(v4) = local {
        let octets = v4.octets();
        return Some(format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2]));
    }
    None
}

/// Probe mDNS services via host tooling (`dns-sd`, `avahi-browse`).
pub fn probe_mdns(timeout_ms: u64) -> Vec<DiscoveryMatch> {
    if let Ok(custom) = std::env::var("SPANDA_DISCOVERY_MDNS_MATCHES") {
        return custom
            .split(',')
            .filter(|entry| !entry.trim().is_empty())
            .enumerate()
            .map(|(index, entry)| {
                let parts: Vec<_> = entry.split('@').collect();
                let id = parts.first().copied().unwrap_or("mdns-device");
                let host = parts.get(1).copied().unwrap_or("127.0.0.1");
                discovery_match(
                    &format!("mdns-{id}-{index}"),
                    Some("_spanda._tcp.local"),
                    host,
                    "mdns",
                )
            })
            .collect();
    }

    let services = ["_spanda._tcp", "_ros-master._tcp", "_http._tcp"];
    let mut matches = Vec::new();

    if Command::new("dns-sd").arg("-V").output().is_ok() {
        for service in services {
            if let Some(output) = run_command_output("dns-sd", &["-B", service, "local"], timeout_ms)
            {
                for (index, line) in output.lines().enumerate() {
                    if !line.contains("Add") {
                        continue;
                    }
                    let name = line
                        .split_whitespace()
                        .nth(3)
                        .unwrap_or("mdns-device")
                        .trim_end_matches('.');
                    matches.push(discovery_match(
                        &format!("mdns-{}-{index}", name.replace(' ', "-")),
                        Some(service),
                        "0.0.0.0",
                        "mdns",
                    ));
                }
            }
        }
    }

    if matches.is_empty() && Command::new("avahi-browse").arg("--version").output().is_ok() {
        for service in services {
            if let Some(output) =
                run_command_output("avahi-browse", &["-rt", service], timeout_ms)
            {
                for (index, line) in output.lines().enumerate() {
                    if !line.contains(';') {
                        continue;
                    }
                    let name = line.split(';').nth(3).unwrap_or("mdns-device");
                    matches.push(discovery_match(
                        &format!("mdns-{}-{index}", name.replace(' ', "-")),
                        Some(service),
                        "0.0.0.0",
                        "mdns",
                    ));
                }
            }
        }
    }

    matches
}

/// Probe BLE adapters and paired devices via `bluetoothctl` or macOS profiler.
pub fn probe_ble() -> Vec<DiscoveryMatch> {
    if let Ok(custom) = std::env::var("SPANDA_DISCOVERY_BLE_MATCHES") {
        return custom
            .split(',')
            .filter(|entry| !entry.trim().is_empty())
            .enumerate()
            .map(|(index, mac)| {
                discovery_match(
                    &format!("ble-{index}"),
                    None,
                    mac.trim(),
                    "ble",
                )
            })
            .collect();
    }

    if let Some(output) = run_command_output("bluetoothctl", &["devices"], 500) {
        let matches: Vec<_> = output
            .lines()
            .filter(|line| line.starts_with("Device "))
            .enumerate()
            .map(|(index, line)| {
                let mac = line.split_whitespace().nth(1).unwrap_or("ble");
                let name = line.split_whitespace().skip(2).collect::<Vec<_>>().join("-");
                discovery_match(
                    &format!("ble-{}", if name.is_empty() { index } else { index }),
                    Some(&name),
                    mac,
                    "ble",
                )
            })
            .collect();
        if !matches.is_empty() {
            return matches;
        }
    }

    if let Some(output) = run_command_output("system_profiler", &["SPBluetoothDataType"], 1500) {
        let matches: Vec<_> = output
            .lines()
            .filter(|line| line.contains("Address:"))
            .enumerate()
            .map(|(index, line)| {
                let mac = line.split(':').skip(1).collect::<String>().trim().to_string();
                discovery_match(&format!("ble-{index}"), None, &mac, "ble")
            })
            .collect();
        if !matches.is_empty() {
            return matches;
        }
    }

    Vec::new()
}

/// Probe USB devices via `lsusb` or macOS `system_profiler`.
pub fn probe_usb() -> Vec<DiscoveryMatch> {
    if let Ok(custom) = std::env::var("SPANDA_DISCOVERY_USB_MATCHES") {
        return custom
            .split(',')
            .filter(|entry| !entry.trim().is_empty())
            .enumerate()
            .map(|(index, id)| discovery_match(&format!("usb-{index}"), None, id.trim(), "usb"))
            .collect();
    }

    if let Some(output) = run_command_output("lsusb", &[], 500) {
        let matches: Vec<_> = output
            .lines()
            .enumerate()
            .map(|(index, line)| {
                let id = line
                    .split_whitespace()
                    .nth(5)
                    .unwrap_or("usb-device")
                    .replace(':', "-");
                discovery_match(&format!("usb-{id}-{index}"), None, "usb", "usb")
            })
            .collect();
        if !matches.is_empty() {
            return matches;
        }
    }

    if let Some(output) = run_command_output("system_profiler", &["SPUSBDataType"], 1500) {
        let matches: Vec<_> = output
            .lines()
            .filter(|line| line.contains("Serial Number:"))
            .enumerate()
            .map(|(index, line)| {
                let serial = line.split(':').skip(1).collect::<String>().trim().to_string();
                discovery_match(&format!("usb-{serial}-{index}"), None, "usb", "usb")
            })
            .collect();
        if !matches.is_empty() {
            return matches;
        }
    }

    Vec::new()
}

/// Probe SocketCAN interfaces from sysfs or `ip link`.
pub fn probe_can() -> Vec<DiscoveryMatch> {
    if let Ok(custom) = std::env::var("SPANDA_DISCOVERY_CAN_MATCHES") {
        return custom
            .split(',')
            .filter(|entry| !entry.trim().is_empty())
            .enumerate()
            .map(|(index, iface)| {
                discovery_match(&format!("can-{index}"), Some(iface.trim()), "can", "can")
            })
            .collect();
    }

    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        let matches: Vec<_> = entries
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("can")
            })
            .enumerate()
            .map(|(index, entry)| {
                let iface = entry.file_name().to_string_lossy().into_owned();
                discovery_match(&format!("can-{iface}-{index}"), Some(&iface), "can", "can")
            })
            .collect();
        if !matches.is_empty() {
            return matches;
        }
    }

    if let Some(output) = run_command_output("ip", &["-json", "link", "show", "type", "can"], 500) {
        if output.contains("\"ifname\"") {
            for (index, line) in output.lines().filter(|line| line.contains("\"ifname\"")).enumerate()
            {
                if let Some(name) = line.split('"').nth(3) {
                    return vec![discovery_match(
                        &format!("can-{name}-{index}"),
                        Some(name),
                        "can",
                        "can",
                    )];
                }
            }
        }
    }

    Vec::new()
}

/// Probe an MQTT broker via TCP connect (`SPANDA_MQTT_BROKER`, default `127.0.0.1:1883`).
pub fn probe_mqtt(timeout_ms: u64) -> Vec<DiscoveryMatch> {
    let broker = std::env::var("SPANDA_MQTT_BROKER").unwrap_or_else(|_| "127.0.0.1:1883".into());
    let (host, port) = parse_host_port(&broker, 1883);
    let Some(addr) = resolve_socket_addr(&host, port) else {
        return Vec::new();
    };
    if TcpStream::connect_timeout(&addr, Duration::from_millis(timeout_ms)).is_ok() {
        return vec![discovery_match(
            "mqtt-broker",
            Some("mqtt"),
            &host,
            "mqtt",
        )];
    }
    Vec::new()
}

/// Probe ROS2 graph via `ros2 topic list` when the CLI is installed.
pub fn probe_ros2(timeout_ms: u64) -> Vec<DiscoveryMatch> {
    if std::env::var("SPANDA_DISCOVERY_ROS2_DISABLE").is_ok() {
        return Vec::new();
    }
    if Command::new("ros2").arg("--help").output().is_err() {
        return Vec::new();
    }
    let domain = std::env::var("ROS_DOMAIN_ID").unwrap_or_else(|_| "0".into());
    if let Some(output) = run_command_output("ros2", &["topic", "list", "--no-daemon"], timeout_ms)
    {
        let topics: Vec<_> = output.lines().filter(|line| line.starts_with('/')).collect();
        if !topics.is_empty() {
            return vec![discovery_match(
                &format!("ros2-domain-{domain}"),
                Some("ros2"),
                "ros2",
                "ros2",
            )];
        }
    }
    Vec::new()
}

fn parse_host_port(value: &str, default_port: u16) -> (String, u16) {
    if let Some((host, port)) = value.rsplit_once(':') {
        if let Ok(parsed) = port.parse::<u16>() {
            return (host.to_string(), parsed);
        }
    }
    (value.to_string(), default_port)
}

fn resolve_socket_addr(host: &str, port: u16) -> Option<SocketAddr> {
    if let Ok(ip) = host.parse::<Ipv4Addr>() {
        return Some(SocketAddr::new(IpAddr::V4(ip), port));
    }
    format!("{host}:{port}").parse().ok()
}

fn env_match_list(env_key: &str, transport: &str, default_logical: &str) -> Vec<DiscoveryMatch> {
    let Ok(custom) = std::env::var(env_key) else {
        return Vec::new();
    };
    custom
        .split(',')
        .filter(|entry| !entry.trim().is_empty())
        .enumerate()
        .map(|(index, entry)| {
            let parts: Vec<_> = entry.split('@').collect();
            let id = parts.first().copied().unwrap_or("device");
            let host = parts.get(1).copied().unwrap_or("127.0.0.1");
            discovery_match(
                &format!("{transport}-{id}-{index}"),
                Some(default_logical),
                host,
                transport,
            )
        })
        .collect()
}

/// Probe WiFi-associated hosts via env override or subnet correlation.
pub fn probe_wifi(timeout_ms: u64) -> Vec<DiscoveryMatch> {
    let env_matches = env_match_list("SPANDA_DISCOVERY_WIFI_MATCHES", "wifi", "wifi");
    if !env_matches.is_empty() {
        return env_matches;
    }
    if let Some(subnet) = default_discovery_subnet() {
        let hosts = crate::device_identity::scan_subnet(&subnet, &[80, 443, 8080], timeout_ms);
        if !hosts.is_empty() {
            return hosts
                .into_iter()
                .enumerate()
                .map(|(index, probe)| DiscoveryMatch {
                    device_id: format!("wifi-host-{index}"),
                    logical_name: Some("wifi".into()),
                    configured_ip: probe.ip.clone(),
                    probe,
                    matched_by: "wifi".into(),
                })
                .collect();
        }
    }
    Vec::new()
}

/// Probe cellular modems via env override, `mmcli`, or ModemManager D-Bus listing.
pub fn probe_cellular() -> Vec<DiscoveryMatch> {
    let env_matches = env_match_list(
        "SPANDA_DISCOVERY_CELLULAR_MATCHES",
        "cellular",
        "cellular",
    );
    if !env_matches.is_empty() {
        return env_matches;
    }

    if let Some(output) = run_command_output("mmcli", &["-L"], 1500) {
        let matches: Vec<_> = output
            .lines()
            .filter(|line| line.contains("/Modem/"))
            .enumerate()
            .map(|(index, line)| {
                let modem_id = line
                    .split('/')
                    .next_back()
                    .unwrap_or("0")
                    .trim();
                discovery_match(
                    &format!("cellular-modem-{modem_id}-{index}"),
                    Some("lte"),
                    modem_id,
                    "cellular",
                )
            })
            .collect();
        if !matches.is_empty() {
            return matches;
        }
    }

    Vec::new()
}

/// Probe serial devices via env override or shallow `/dev/tty*` listing.
pub fn probe_serial() -> Vec<DiscoveryMatch> {
    let env_matches = env_match_list("SPANDA_DISCOVERY_SERIAL_MATCHES", "serial", "serial");
    if !env_matches.is_empty() {
        return env_matches;
    }
    let mut matches = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/dev") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("ttyUSB") || name.starts_with("ttyACM") {
                let path = format!("/dev/{name}");
                matches.push(discovery_match(
                    &format!("serial-{name}"),
                    Some("serial"),
                    &path,
                    "serial",
                ));
            }
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_subnet_from_env() {
        std::env::set_var("SPANDA_DISCOVERY_SUBNET", "10.0.0.0/24");
        assert_eq!(default_discovery_subnet().as_deref(), Some("10.0.0.0/24"));
        std::env::remove_var("SPANDA_DISCOVERY_SUBNET");
    }

    #[test]
    fn mdns_env_override_returns_matches() {
        std::env::set_var("SPANDA_DISCOVERY_MDNS_MATCHES", "rover@192.168.1.10");
        let matches = probe_mdns(100);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].configured_ip, "192.168.1.10");
        std::env::remove_var("SPANDA_DISCOVERY_MDNS_MATCHES");
    }
}
