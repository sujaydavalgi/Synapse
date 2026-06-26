# Device Discovery

Spanda discovers devices through **provider interfaces** — core ships built-in transports; live backends ship as registry packages.

## Built-in transports

| Transport | CLI/API flag | Notes |
|-----------|--------------|-------|
| Subnet scan | `subnet` | TCP probe on common ports; auto-detects local `/24` when omitted |
| mDNS | `mdns` | Host `dns-sd` / `avahi-browse` when installed; stub fallback |
| BLE | `ble` | `bluetoothctl` / macOS Bluetooth profiler; env `SPANDA_DISCOVERY_BLE_MATCHES` |
| USB | `usb` | `lsusb` / macOS USB profiler; env `SPANDA_DISCOVERY_USB_MATCHES` |
| CAN | `can` | SocketCAN sysfs / `ip link type can`; env `SPANDA_DISCOVERY_CAN_MATCHES` |
| MQTT | `mqtt` | TCP probe to `SPANDA_MQTT_BROKER` (default `127.0.0.1:1883`) |
| ROS2/DDS | `ros2` | `ros2 topic list` when CLI installed; env `SPANDA_DISCOVERY_ROS2_DISABLE` |
| WiFi | `wifi` | Subnet correlation; env `SPANDA_DISCOVERY_WIFI_MATCHES` |
| Cellular | `cellular` | `mmcli -L` when ModemManager installed; env `SPANDA_DISCOVERY_CELLULAR_MATCHES` |
| Serial | `serial` | `/dev/ttyUSB*` / `/dev/ttyACM*`; env `SPANDA_DISCOVERY_SERIAL_MATCHES` |

## CLI

```bash
spanda device discover --subnet 192.168.1.0/24
spanda device discover --transport mdns --json
```

## API

```bash
# GET (query params)
curl "http://127.0.0.1:8080/v1/discovery?transport=subnet&subnet=192.168.1.0/24"

# POST (multi-transport)
curl -X POST http://127.0.0.1:8080/v1/devices/discover \
  -H "Authorization: Bearer $SPANDA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"subnet":"192.168.1.0/24","transports":["subnet","mdns"],"timeout_ms":2000}'
```

Discovered matches are registered into the device pool (`registered` array in the response). Responses include `installed_packages` when registry discovery packages (for example `spanda-discovery-mdns`) are present under `packages/registry/` — matches are tagged `mdns:spanda-discovery-mdns` instead of bare `mdns`.

## Registry packages (runtime)

When `packages/registry/spanda-discovery-mdns/` is on disk, the mDNS transport is wrapped with registry package metadata at runtime (`list_installed_discovery_packages()`). Additional transports (`spanda-discovery-ble`, `spanda-discovery-usb`, …) follow the same pattern as they land in the registry.

Environment overrides for CI and headless hosts:

- `SPANDA_DISCOVERY_SUBNET` — default CIDR when `subnet` is omitted
- `SPANDA_DISCOVERY_MDNS_MATCHES` — comma list `name@ip` for deterministic mDNS results
- `SPANDA_MQTT_BROKER` — broker host:port for MQTT probe
- `SPANDA_DISCOVERY_NO_STUB=1` — return empty results instead of stub devices when live probes find nothing

## Provider contract

Packages implement `DeviceDiscoveryTransport`:

```rust
pub trait DeviceDiscoveryTransport: Send + Sync {
    fn transport_name(&self) -> &'static str;
    fn discover(&self, options: &DiscoveryOptions) -> Result<DiscoveryTransportResult, String>;
}
```

Register packages in `spanda.providers.toml`; do not hardcode vendor logic in `spanda-config`.

## Unknown devices

Newly discovered devices enter **quarantined** or **discovered** state with `trust_level = unknown` until an operator approves trust.

## Related

- [device-pool.md](./device-pool.md)
- [device-quarantine.md](./device-quarantine.md)
- [packages.md](./packages.md)
