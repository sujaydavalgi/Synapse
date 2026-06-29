//! In-memory IoT device, telemetry, and shadow store for package dispatch stubs.

use spanda_runtime::device_telemetry_sink::device_telemetry_sink;
use spanda_runtime::providers::{Command, DeviceShadow, IoTDevice, Telemetry};
use spanda_runtime::value::RuntimeValue;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static IOT_HUB: OnceLock<Mutex<IotHub>> = OnceLock::new();

fn hub() -> &'static Mutex<IotHub> {
    // Description:
    //     Hub.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: &'static Mutex<IotHub>
    //         Return value from `hub`.
    //
    // Example:

    //     let result = spanda_providers::iot_hub::hub();

    IOT_HUB.get_or_init(|| Mutex::new(IotHub::default()))
}

/// Shared in-memory IoT state for sim and stub integrations.
#[derive(Default)]
pub struct IotHub {
    devices: HashMap<String, IoTDevice>,
    telemetry: Vec<Telemetry>,
    shadows: HashMap<String, DeviceShadow>,
    modbus_registers: HashMap<u16, f64>,
    opcua_nodes: HashMap<String, String>,
    zigbee_attributes: HashMap<String, String>,
    lora_payloads: HashMap<String, String>,
    matter_clusters: HashMap<String, f64>,
    canbus_frames: HashMap<u32, f64>,
    bacnet_points: HashMap<String, String>,
    knx_groups: HashMap<String, String>,
    thread_endpoints: HashMap<String, String>,
    zwave_values: HashMap<String, String>,
    string_stubs: HashMap<String, String>,
}

impl IotHub {
    pub fn register_device(&mut self, device: IoTDevice) -> Result<(), String> {
        // Description:
        //     Register device.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     device: IoTDevice
        //         Caller-supplied device.
        //
        // Outputs:
        //     result: Result<(), String>
        //         Return value from `register_device`.
        //
        // Example:

        //     let result = spanda_providers::iot_hub::register_device(&mut self, device);

        if device.id.is_empty() {
            return Err("device id required".into());
        }
        self.devices.insert(device.id.clone(), device);
        Ok(())
    }

    pub fn publish_telemetry(&mut self, telemetry: Telemetry) {
        // Description:
        //     Publish telemetry.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     elemetry: Telemetry
        //         Caller-supplied elemetry.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_providers::iot_hub::publish_telemetry(&mut self, elemetry);

        self.telemetry.push(telemetry);
    }

    pub fn send_command(&mut self, command: Command) -> Result<(), String> {
        // Description:
        //     Send command.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     command: Command
        //         Caller-supplied command.
        //
        // Outputs:
        //     result: Result<(), String>
        //         Return value from `send_command`.
        //
        // Example:

        //     let result = spanda_providers::iot_hub::send_command(&mut self, command);

        if !self.devices.contains_key(&command.device_id) {
            return Err(format!("unknown device '{}'", command.device_id));
        }
        Ok(())
    }

    pub fn update_shadow(&mut self, shadow: DeviceShadow) {
        // Description:
        //     Update shadow.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     shadow: DeviceShadow
        //         Caller-supplied shadow.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_providers::iot_hub::update_shadow(&mut self, shadow);

        self.shadows.insert(shadow.device_id.clone(), shadow);
    }

    pub fn read_modbus_register(&self, address: u16) -> f64 {
        // Description:
        //     Read modbus register.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //     address: u16
        //         Caller-supplied address.
        //
        // Outputs:
        //     result: f64
        //         Return value from `read_modbus_register`.
        //
        // Example:

        //     let result = spanda_providers::iot_hub::read_modbus_register(&self, address);

        self.modbus_registers.get(&address).copied().unwrap_or(0.0)
    }

    pub fn write_modbus_register(&mut self, address: u16, value: f64) {
        // Description:
        //     Write modbus register.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     address: u16
        //         Caller-supplied address.
        //     value: f64
        //         Caller-supplied value.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_providers::iot_hub::write_modbus_register(&mut self, address, value);

        self.modbus_registers.insert(address, value);
    }

    pub fn read_opcua_node(&self, node: &str) -> Option<String> {
        // Description:
        //     Read opcua node.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //     node: &str
        //         Caller-supplied node.
        //
        // Outputs:
        //     result: Option<String>
        //         Return value from `read_opcua_node`.
        //
        // Example:

        //     let result = spanda_providers::iot_hub::read_opcua_node(&self, node);

        self.opcua_nodes.get(node).cloned()
    }

    pub fn read_zigbee_attribute(&self, device: &str, cluster: &str) -> String {
        // Description:
        //     Read zigbee attribute.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //     device: &str
        //         Caller-supplied device.
        //     cluster: &str
        //         Caller-supplied cluster.
        //
        // Outputs:
        //     result: String
        //         Return value from `read_zigbee_attribute`.
        //
        // Example:

        //     let result = spanda_providers::iot_hub::read_zigbee_attribute(&self, device, cluster);

        let key = format!("{device}:{cluster}");
        self.zigbee_attributes
            .get(&key)
            .cloned()
            .unwrap_or_else(|| format!("zigbee:{device}:{cluster}"))
    }

    pub fn read_lora_payload(&self, device_id: &str) -> String {
        // Description:
        //     Read lora payload.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //     device_id: &str
        //         Caller-supplied device id.
        //
        // Outputs:
        //     result: String
        //         Return value from `read_lora_payload`.
        //
        // Example:

        //     let result = spanda_providers::iot_hub::read_lora_payload(&self, device_id);

        self.lora_payloads
            .get(device_id)
            .cloned()
            .unwrap_or_else(|| format!("lora:{device_id}"))
    }

    pub fn read_matter_cluster(&self, node: &str, cluster: &str) -> f64 {
        // Description:
        //     Read matter cluster.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //     node: &str
        //         Caller-supplied node.
        //     cluster: &str
        //         Caller-supplied cluster.
        //
        // Outputs:
        //     result: f64
        //         Return value from `read_matter_cluster`.
        //
        // Example:

        //     let result = spanda_providers::iot_hub::read_matter_cluster(&self, node, cluster);

        let key = format!("{node}:{cluster}");
        self.matter_clusters.get(&key).copied().unwrap_or(1.0)
    }

    pub fn read_canbus_frame(&self, can_id: u32) -> f64 {
        // Description:
        //     Read canbus frame.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //     can_id: u32
        //         Caller-supplied can id.
        //
        // Outputs:
        //     result: f64
        //         Return value from `read_canbus_frame`.
        //
        // Example:

        //     let result = spanda_providers::iot_hub::read_canbus_frame(&self, can_id);

        self.canbus_frames.get(&can_id).copied().unwrap_or(0.0)
    }

    pub fn read_bacnet_point(&self, device: &str, object_id: &str) -> String {
        let key = format!("{device}:{object_id}");
        self.bacnet_points
            .get(&key)
            .cloned()
            .unwrap_or_else(|| format!("bacnet:{device}:{object_id}"))
    }

    pub fn read_knx_group(&self, address: &str) -> String {
        self.knx_groups
            .get(address)
            .cloned()
            .unwrap_or_else(|| format!("knx:{address}"))
    }

    pub fn read_thread_endpoint(&self, device: &str) -> String {
        self.thread_endpoints
            .get(device)
            .cloned()
            .unwrap_or_else(|| format!("thread:{device}"))
    }

    pub fn read_zwave_value(&self, device: &str, command_class: &str) -> String {
        let key = format!("{device}:{command_class}");
        self.zwave_values
            .get(&key)
            .or_else(|| self.zwave_values.get(device))
            .cloned()
            .unwrap_or_else(|| format!("zwave:{device}:{command_class}"))
    }

    pub fn read_string_stub(&self, key: &str) -> String {
        self.string_stubs
            .get(key)
            .cloned()
            .unwrap_or_else(|| format!("stub:{key}"))
    }

    pub fn seed_protocol_demo(&mut self) {
        // Description:
        //     Seed protocol demo.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_providers::iot_hub::seed_protocol_demo(&mut self);

        self.opcua_nodes
            .insert("ns=2;s=Temperature".into(), "22.5".into());
        self.zigbee_attributes
            .insert("sensor-1:temp".into(), "21.0".into());
        self.lora_payloads
            .insert("node-a".into(), "payload:ok".into());
        self.matter_clusters.insert("light:onoff".into(), 1.0);
        self.canbus_frames.insert(0x100, 42.0);
        self.bacnet_points
            .insert("ahu-12:present-value".into(), "72.0".into());
        self.knx_groups
            .insert("1/2/3".into(), "occupied".into());
        self.thread_endpoints
            .insert("matter-hub-backup".into(), "online".into());
        self.zwave_values
            .insert("leak-basement".into(), "dry".into());
        self.string_stubs.insert(
            "energy:solar-001".into(),
            "generation_kw:4.2".into(),
        );
        self.string_stubs.insert(
            "building:tower-demo".into(),
            "readiness:85".into(),
        );
        self.string_stubs.insert("lock:lock-front".into(), "locked".into());
        self.string_stubs.insert("environment:co2-lobby".into(), "co2:620".into());
        self.string_stubs
            .insert("home_assistant:climate.living_room".into(), "heat".into());
    }

    pub fn device_count(&self) -> usize {
        // Description:
        //     Device count.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: usize
        //         Return value from `device_count`.
        //
        // Example:

        //     let result = spanda_providers::iot_hub::device_count(&self);

        self.devices.len()
    }

    pub fn telemetry_count(&self) -> usize {
        // Description:
        //     Telemetry count.
        //
        // Inputs:
        //     &self: input value
        //         Caller-supplied &self.
        //
        // Outputs:
        //     result: usize
        //         Return value from `telemetry_count`.
        //
        // Example:

        //     let result = spanda_providers::iot_hub::telemetry_count(&self);

        self.telemetry.len()
    }
}

/// Register a device in the in-memory IoT hub.
pub fn register_device(device: IoTDevice) -> Result<(), String> {
    // Description:
    //     Register device.
    //
    // Inputs:
    //     device: IoTDevice
    //         Caller-supplied device.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `register_device`.
    //
    // Example:

    //     let result = spanda_providers::iot_hub::register_device(device);

    let device_id = device.id.clone();
    let protocol = device.protocol.clone();
    hub().lock().unwrap().register_device(device)?;
    if device_telemetry_sink().persist_enabled() {
        let sink = device_telemetry_sink();
        sink.record_device_heartbeat(
            &device_id,
            sink.wall_timestamp_ms(),
            None,
            Some(protocol.as_str()),
            5000.0,
        );
    }
    Ok(())
}

/// Publish telemetry to the in-memory IoT hub.
pub fn publish_telemetry(telemetry: Telemetry) {
    // Description:
    //     Publish telemetry.
    //
    // Inputs:
    //     elemetry: Telemetry
    //         Caller-supplied elemetry.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::iot_hub::publish_telemetry(elemetry);

    let sink = device_telemetry_sink();
    if sink.persist_enabled() {
        let Telemetry {
            device_id,
            metric,
            value,
            timestamp_ms,
        } = &telemetry;
        sink.record_device_telemetry(device_id, metric, value, *timestamp_ms, None);
        if sink.is_heartbeat_metric(metric) {
            sink.record_device_heartbeat(device_id, *timestamp_ms, None, None, 5000.0);
        }
    }
    hub().lock().unwrap().publish_telemetry(telemetry);
}

/// Send a remote command through the in-memory IoT hub.
pub fn send_command(command: Command) -> Result<(), String> {
    // Description:
    //     Send command.
    //
    // Inputs:
    //     command: Command
    //         Caller-supplied command.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `send_command`.
    //
    // Example:

    //     let result = spanda_providers::iot_hub::send_command(command);

    hub().lock().unwrap().send_command(command)
}

/// Update a device shadow in the in-memory IoT hub.
pub fn update_shadow(shadow: DeviceShadow) {
    // Description:
    //     Update shadow.
    //
    // Inputs:
    //     shadow: DeviceShadow
    //         Caller-supplied shadow.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::iot_hub::update_shadow(shadow);

    hub().lock().unwrap().update_shadow(shadow);
}

/// Read a Modbus register from the in-memory IoT hub.
pub fn read_modbus_register(address: u16) -> f64 {
    // Description:
    //     Read modbus register.
    //
    // Inputs:
    //     address: u16
    //         Caller-supplied address.
    //
    // Outputs:
    //     result: f64
    //         Return value from `read_modbus_register`.
    //
    // Example:

    //     let result = spanda_providers::iot_hub::read_modbus_register(address);

    if let Some(value) = crate::iot_live::read_modbus_register_live(address) {
        return value;
    }
    hub().lock().unwrap().read_modbus_register(address)
}

/// Read an OPC-UA node value from the in-memory IoT hub.
pub fn read_opcua_node(node: &str) -> Option<String> {
    // Description:
    //     Read opcua node.
    //
    // Inputs:
    //     node: &str
    //         Caller-supplied node.
    //
    // Outputs:
    //     result: Option<String>
    //         Return value from `read_opcua_node`.
    //
    // Example:

    //     let result = spanda_providers::iot_hub::read_opcua_node(node);

    if let Some(value) = crate::iot_live::read_opcua_node_live(node) {
        return Some(value);
    }
    hub().lock().unwrap().read_opcua_node(node)
}

/// Read a CAN bus frame value from the in-memory IoT hub.
pub fn read_canbus_frame(can_id: u32) -> f64 {
    // Description:
    //     Read canbus frame.
    //
    // Inputs:
    //     can_id: u32
    //         Caller-supplied can id.
    //
    // Outputs:
    //     result: f64
    //         Return value from `read_canbus_frame`.
    //
    // Example:

    //     let result = spanda_providers::iot_hub::read_canbus_frame(can_id);

    if let Some(value) = crate::iot_live::read_canbus_frame_live(can_id) {
        return value;
    }
    hub().lock().unwrap().read_canbus_frame(can_id)
}

/// Read a Zigbee attribute from the in-memory IoT hub.
pub fn read_zigbee_attribute(device: &str, cluster: &str) -> String {
    // Description:
    //     Read zigbee attribute.
    //
    // Inputs:
    //     device: &str
    //         Caller-supplied device.
    //     cluster: &str
    //         Caller-supplied cluster.
    //
    // Outputs:
    //     result: String
    //         Return value from `read_zigbee_attribute`.
    //
    // Example:

    //     let result = spanda_providers::iot_hub::read_zigbee_attribute(device, cluster);

    if let Some(value) = crate::iot_live::read_zigbee_attribute_live(device, cluster) {
        return value;
    }
    hub().lock().unwrap().read_zigbee_attribute(device, cluster)
}

/// Read a LoRa payload from the in-memory IoT hub.
pub fn read_lora_payload(device_id: &str) -> String {
    // Description:
    //     Read lora payload.
    //
    // Inputs:
    //     device_id: &str
    //         Caller-supplied device id.
    //
    // Outputs:
    //     result: String
    //         Return value from `read_lora_payload`.
    //
    // Example:

    //     let result = spanda_providers::iot_hub::read_lora_payload(device_id);

    if let Some(value) = crate::iot_live::read_lora_payload_live(device_id) {
        return value;
    }
    hub().lock().unwrap().read_lora_payload(device_id)
}

/// Read a Matter cluster value from the in-memory IoT hub.
pub fn read_matter_cluster(node: &str, cluster: &str) -> f64 {
    // Description:
    //     Read matter cluster.
    //
    // Inputs:
    //     node: &str
    //         Caller-supplied node.
    //     cluster: &str
    //         Caller-supplied cluster.
    //
    // Outputs:
    //     result: f64
    //         Return value from `read_matter_cluster`.
    //
    // Example:

    //     let result = spanda_providers::iot_hub::read_matter_cluster(node, cluster);

    if let Some(value) = crate::iot_live::read_matter_cluster_live(node, cluster) {
        return value;
    }
    hub().lock().unwrap().read_matter_cluster(node, cluster)
}

pub fn read_bacnet_point(device: &str, object_id: &str) -> String {
    if let Some(value) = crate::iot_live::read_bacnet_point_live(device, object_id) {
        return value;
    }
    hub().lock().unwrap().read_bacnet_point(device, object_id)
}

pub fn read_knx_group(address: &str) -> String {
    if let Some(value) = crate::iot_live::read_knx_group_live(address) {
        return value;
    }
    hub().lock().unwrap().read_knx_group(address)
}

pub fn read_thread_endpoint(device: &str) -> String {
    if let Some(value) = crate::iot_live::read_thread_endpoint_live(device) {
        return value;
    }
    hub().lock().unwrap().read_thread_endpoint(device)
}

pub fn read_zwave_value(device: &str, command_class: &str) -> String {
    if let Some(value) = crate::iot_live::read_zwave_value_live(device, command_class) {
        return value;
    }
    hub().lock().unwrap().read_zwave_value(device, command_class)
}

pub fn read_string_stub(key: &str) -> String {
    hub().lock().unwrap().read_string_stub(key)
}

/// Seed demo protocol values for golden-path tests.
pub fn seed_protocol_demos() {
    // Description:
    //     Seed protocol demos.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::iot_hub::seed_protocol_demos();

    hub().lock().unwrap().seed_protocol_demo();
}

/// Snapshot hub metrics for tests and diagnostics.
pub fn hub_stats() -> (usize, usize) {
    // Description:
    //     Hub stats.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: (usize, usize)
    //         Return value from `hub_stats`.
    //
    // Example:

    //     let result = spanda_providers::iot_hub::hub_stats();

    let hub = hub().lock().unwrap();
    (hub.device_count(), hub.telemetry_count())
}

/// Seed a default Modbus register for golden-path demos.
pub fn seed_modbus_demo_register(address: u16, value: f64) {
    // Description:
    //     Seed modbus demo register.
    //
    // Inputs:
    //     address: u16
    //         Caller-supplied address.
    //     value: f64
    //         Caller-supplied value.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::iot_hub::seed_modbus_demo_register(address, value);

    hub().lock().unwrap().write_modbus_register(address, value);
}

/// Extract string argument from runtime values.
pub fn string_arg(args: &[RuntimeValue], index: usize) -> String {
    // Description:
    //     String arg.
    //
    // Inputs:
    //     args: &[RuntimeValue]
    //         Caller-supplied args.
    //     index: usize
    //         Caller-supplied index.
    //
    // Outputs:
    //     result: String
    //         Return value from `string_arg`.
    //
    // Example:

    //     let result = spanda_providers::iot_hub::string_arg(args, index);

    match args.get(index) {
        Some(RuntimeValue::String { value }) => value.clone(),
        _ => String::new(),
    }
}

/// Extract numeric argument from runtime values.
pub fn number_arg(args: &[RuntimeValue], index: usize) -> f64 {
    // Description:
    //     Number arg.
    //
    // Inputs:
    //     args: &[RuntimeValue]
    //         Caller-supplied args.
    //     index: usize
    //         Caller-supplied index.
    //
    // Outputs:
    //     result: f64
    //         Return value from `number_arg`.
    //
    // Example:

    //     let result = spanda_providers::iot_hub::number_arg(args, index);

    match args.get(index) {
        Some(RuntimeValue::Number { value, .. }) => *value,
        _ => 0.0,
    }
}
