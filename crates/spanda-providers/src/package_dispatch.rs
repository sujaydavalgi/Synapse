//! Runtime dispatch from official package module exports to provider registry backends.
//!
use crate::anomaly_onnx::scan_learned_score;
use crate::automotive_hub::{read_lidar_distance, read_radar_distance, read_ultrasonic_distance};
use crate::iot_hub::{
    number_arg, publish_telemetry, read_canbus_frame, read_lora_payload, read_matter_cluster,
    read_modbus_register, read_opcua_node, read_zigbee_attribute, register_device, send_command,
    string_arg, update_shadow,
};
use spanda_runtime::fusion::{weight_for_sensor_type, weighted_confidence};
use spanda_runtime::providers::{transport_registry_key, ProviderRegistry};
use spanda_runtime::providers::{Command, DeviceShadow, IoTDevice, Telemetry};
use spanda_runtime::replay::MissionTrace;
use spanda_runtime::telemetry::RuntimeTelemetry;
use spanda_runtime::value::RuntimeValue;
use std::time::Instant;

/// Optional observability sinks for provider dispatch.
pub struct ProviderDispatchContext<'a> {
    pub telemetry: Option<&'a mut RuntimeTelemetry>,
    pub mission_trace: Option<&'a mut MissionTrace>,
    pub sim_time_ms: f64,
}

/// Map a dotted module import path to its backing official package name.
pub fn official_package_for_module(module_path: &str) -> Option<&'static str> {
    // Description:
    //     Official package for module.
    //
    // Inputs:
    //     odule_path: &str
    //         Caller-supplied odule path.
    //
    // Outputs:
    //     result: Option<&'static str>
    //         Return value from `official_package_for_module`.
    //
    // Example:

    //     let result = spanda_providers::package_dispatch::official_package_for_module(odule_path);

    match module_path {
        "positioning.gps" => Some("spanda-gps"),
        "connectivity.wifi" => Some("spanda-wifi"),
        "connectivity.ble" => Some("spanda-ble"),
        "connectivity.cellular" => Some("spanda-cellular"),
        "navigation.path_planning" => Some("spanda-nav"),
        "navigation.slam" => Some("spanda-slam"),
        "robotics.fleet" => Some("spanda-fleet"),
        "communication.mqtt" => Some("spanda-mqtt"),
        "communication.dds" => Some("spanda-dds"),
        "robotics.ros2" => Some("spanda-ros2"),
        "deploy.ota" => Some("spanda-ota"),
        "vision.opencv" => Some("spanda-opencv"),
        "vision.yolo" => Some("spanda-yolo"),
        "sim.gazebo" => Some("spanda-gazebo"),
        "sim.webots" => Some("spanda-webots"),
        "ai.openai" => Some("spanda-openai"),
        "provenance.ledger" => Some("spanda-ledger"),
        "iot.device" => Some("spanda-iot-core"),
        "iot.telemetry" => Some("spanda-iot-core"),
        "iot.command" => Some("spanda-iot-core"),
        "iot.shadow" => Some("spanda-iot-core"),
        "iot.modbus" => Some("spanda-modbus"),
        "iot.opcua" => Some("spanda-opcua"),
        "iot.zigbee" => Some("spanda-zigbee"),
        "iot.lora" => Some("spanda-lora"),
        "iot.matter" => Some("spanda-matter"),
        "iot.canbus" => Some("spanda-canbus"),
        "sensors.radar" => Some("spanda-radar"),
        "sensors.lidar" => Some("spanda-lidar"),
        "sensors.ultrasonic" => Some("spanda-ultrasonic"),
        "automotive.ethernet" => Some("spanda-automotive-ethernet"),
        "automotive.lin" => Some("spanda-lin"),
        "automotive.uds" => Some("spanda-uds"),
        "automotive.v2x" => Some("spanda-v2x"),
        "assurance.evidence" => Some("spanda-assurance"),
        "assurance.knowledge" => Some("spanda-knowledge-model"),
        "assurance.anomaly" => Some("spanda-anomaly"),
        "assurance.diagnosis" => Some("spanda-diagnosis"),
        "assurance.prognostics" => Some("spanda-prognostics"),
        "assurance.mission" => Some("spanda-mission-planning"),
        "assurance.continuity" => Some("spanda-mission-continuity"),
        "assurance.resilience" => Some("spanda-resilience"),
        "assurance.fusion" => Some("spanda-fusion"),
        "wearable.smartwatch" => Some("spanda-smartwatch"),
        "wearable.industrial" => Some("spanda-industrial-wearables"),
        "wearable.bodycam" => Some("spanda-bodycam"),
        "spatial.hololens" => Some("spanda-hololens"),
        "spatial.arkit" => Some("spanda-arkit"),
        "spatial.arcore" => Some("spanda-arcore"),
        "spatial.vision_pro" => Some("spanda-vision-pro"),
        "spatial.magic_leap" => Some("spanda-magic-leap"),
        "spatial.openxr" => Some("spanda-openxr"),
        "hri.voice" => Some("spanda-voice"),
        "hri.gesture" => Some("spanda-gesture"),
        "hri.eye" => Some("spanda-eye-tracking"),
        _ => None,
    }
}

fn project_provider_key(package: &str) -> String {
    // Description:
    //     Project provider key.
    //
    // Inputs:
    //     package: &str
    //         Caller-supplied package.
    //
    // Outputs:
    //     result: String
    //         Return value from `project_provider_key`.
    //
    // Example:

    //     let result = spanda_providers::package_dispatch::project_provider_key(package);

    format!("{package}::project")
}

fn wearable_device_arg(args: &[RuntimeValue]) -> &str {
    args.first()
        .and_then(|value| match value {
            RuntimeValue::String { value } => Some(value.as_str()),
            _ => None,
        })
        .unwrap_or("")
}

fn ok_int() -> RuntimeValue {
    // Description:
    //     Ok int.
    //
    // Inputs:
    //     None.
    //
    // Outputs:
    //     result: RuntimeValue
    //         Return value from `ok_int`.
    //
    // Example:

    //     let result = spanda_providers::package_dispatch::ok_int();

    RuntimeValue::Number {
        value: 1.0,
        unit: spanda_ast::nodes::UnitKind::None,
    }
}

#[allow(clippy::too_many_arguments)]
fn record_call(
    telemetry: Option<&mut RuntimeTelemetry>,
    mission_trace: Option<&mut MissionTrace>,
    sim_time_ms: f64,
    key: &str,
    category: &str,
    module_path: &str,
    function_name: &str,
    started: Instant,
    failed: bool,
) {
    // Description:
    //     Record call.
    //
    // Inputs:
    //     elemetry: Option<&mut RuntimeTelemetry>
    //         Caller-supplied elemetry.
    //     ission_trace: Option<&mut MissionTrace>
    //         Caller-supplied ission trace.
    //     sim_time_ms: f64
    //         Caller-supplied sim time ms.
    //     key: &str
    //         Caller-supplied key.
    //     category: &str
    //         Caller-supplied category.
    //     odule_path: &str
    //         Caller-supplied odule path.
    //     function_name: &str
    //         Caller-supplied function name.
    //     started: Instant
    //         Caller-supplied started.
    //     failed: bool
    //         Caller-supplied failed.
    //
    // Outputs:
    //     None.
    //
    // Example:

    //     let result = spanda_providers::package_dispatch::record_call(elemetry, ission_trace, sim_time_ms, key, category, odule_path, function_name, started, failed);

    if let Some(telemetry) = telemetry {
        let duration_ms = started.elapsed().as_secs_f64() * 1000.0;
        telemetry.record_provider_call(key, category, duration_ms, failed);
    }
    if let Some(trace) = mission_trace {
        trace.record(
            sim_time_ms,
            "provider_call",
            serde_json::json!({
                "module": module_path,
                "function": function_name,
                "provider_key": key,
                "category": category,
                "failed": failed,
            }),
        );
    }
}

/// Dispatch an imported official-package function to a registered provider when installed.
pub fn dispatch_official_package_call(
    registry: &mut ProviderRegistry,
    module_path: &str,
    function_name: &str,
    args: &[RuntimeValue],
    telemetry: Option<&mut RuntimeTelemetry>,
    mission_trace: Option<&mut MissionTrace>,
    sim_time_ms: f64,
) -> Option<RuntimeValue> {
    // Description:
    //     Dispatch official package call.
    //
    // Inputs:
    //     registry: &mut ProviderRegistry
    //         Caller-supplied registry.
    //     odule_path: &str
    //         Caller-supplied odule path.
    //     function_name: &str
    //         Caller-supplied function name.
    //     args: &[RuntimeValue]
    //         Caller-supplied args.
    //     elemetry: Option<&mut RuntimeTelemetry>
    //         Caller-supplied elemetry.
    //     ission_trace: Option<&mut MissionTrace>
    //         Caller-supplied ission trace.
    //     sim_time_ms: f64
    //         Caller-supplied sim time ms.
    //
    // Outputs:
    //     result: Option<RuntimeValue>
    //         Return value from `dispatch_official_package_call`.
    //
    // Example:

    //     let result = spanda_providers::package_dispatch::dispatch_official_package_call(registry, odule_path, function_name, args, elemetry, ission_trace, sim_time_ms);

    let package = official_package_for_module(module_path)?;
    if !registry.has_official_package(package) {
        return None;
    }

    let key = project_provider_key(package);
    let started = Instant::now();
    let category = module_path.split('.').next().unwrap_or("provider");

    let dispatched = match (module_path, function_name) {
        ("positioning.gps", "read") if registry.has_capability("positioning.read") => registry
            .with_positioning(&key, |provider| provider.read_fix())
            .inspect(|_| {
                record_call(
                    telemetry,
                    mission_trace,
                    sim_time_ms,
                    &key,
                    category,
                    module_path,
                    function_name,
                    started,
                    false,
                );
            }),
        ("connectivity.wifi", "connect") if registry.has_capability("connectivity.wifi") => {
            registry
                .with_connectivity(&key, |provider| provider.connect("wifi"))
                .map(|result| {
                    let failed = result.is_err();
                    record_call(
                        telemetry,
                        mission_trace,
                        sim_time_ms,
                        &key,
                        category,
                        module_path,
                        function_name,
                        started,
                        failed,
                    );
                    ok_int()
                })
        }
        ("connectivity.ble", "scan") if registry.has_capability("connectivity.ble") => registry
            .with_connectivity(&key, |provider| provider.connect("ble"))
            .map(|result| {
                let failed = result.is_err();
                record_call(
                    telemetry,
                    mission_trace,
                    sim_time_ms,
                    &key,
                    category,
                    module_path,
                    function_name,
                    started,
                    failed,
                );
                ok_int()
            }),
        ("navigation.path_planning", "navigate") if registry.has_capability("navigation.plan") => {
            let goal = args.first().cloned().unwrap_or(RuntimeValue::Void);
            registry
                .with_navigation(&key, |provider| provider.navigate_to(goal))
                .map(|result| {
                    let failed = result.is_err();
                    record_call(
                        telemetry,
                        mission_trace,
                        sim_time_ms,
                        &key,
                        category,
                        module_path,
                        function_name,
                        started,
                        failed,
                    );
                    result.unwrap_or(RuntimeValue::Void)
                })
        }
        ("navigation.slam", "localize") if registry.has_capability("slam.localize") => registry
            .with_slam(&key, |provider| provider.localize())
            .map(|result| {
                let failed = result.is_err();
                record_call(
                    telemetry,
                    mission_trace,
                    sim_time_ms,
                    &key,
                    category,
                    module_path,
                    function_name,
                    started,
                    failed,
                );
                result.unwrap_or(RuntimeValue::Void)
            }),
        ("vision.opencv", "detect") | ("vision.yolo", "detect")
            if registry.has_capability("vision.detect") =>
        {
            let request = args.first().cloned().unwrap_or(RuntimeValue::Void);
            registry
                .with_vision(&key, |provider| provider.detect(request))
                .inspect(|_| {
                    record_call(
                        telemetry,
                        mission_trace,
                        sim_time_ms,
                        &key,
                        category,
                        module_path,
                        function_name,
                        started,
                        false,
                    );
                })
        }
        ("wearable.smartwatch", "read_telemetry")
        | ("wearable.industrial", "read_telemetry")
        | ("wearable.bodycam", "read_telemetry")
            if registry.has_capability("wearable.telemetry") =>
        {
            let device_id = wearable_device_arg(args);
            registry
                .with_wearable_telemetry(&key, |provider| provider.read_telemetry(device_id))
                .map(|result| {
                    let failed = result.is_err();
                    record_call(
                        telemetry,
                        mission_trace,
                        sim_time_ms,
                        &key,
                        category,
                        module_path,
                        function_name,
                        started,
                        failed,
                    );
                    result.unwrap_or(RuntimeValue::Void)
                })
        }
        ("wearable.smartwatch", "connectivity_status")
            if registry.has_capability("wearable.telemetry") =>
        {
            let device_id = wearable_device_arg(args);
            let connected = registry
                .with_wearable_telemetry(&key, |provider| provider.connectivity_status(device_id))
                .unwrap_or(false);
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                category,
                module_path,
                function_name,
                started,
                false,
            );
            Some(RuntimeValue::Bool { value: connected })
        }
        ("spatial.hololens", "start_session")
        | ("spatial.arkit", "start_session")
        | ("spatial.arcore", "start_session")
        | ("spatial.vision_pro", "start_session")
        | ("spatial.magic_leap", "start_session")
        | ("spatial.openxr", "start_session")
            if registry.has_capability("spatial.session") =>
        {
            let device_id = wearable_device_arg(args);
            registry
                .with_spatial_session(&key, |provider| provider.start_session(device_id))
                .map(|result| {
                    let failed = result.is_err();
                    record_call(
                        telemetry,
                        mission_trace,
                        sim_time_ms,
                        &key,
                        category,
                        module_path,
                        function_name,
                        started,
                        failed,
                    );
                    if result.is_ok() {
                        ok_int()
                    } else {
                        RuntimeValue::Void
                    }
                })
        }
        ("spatial.hololens", "stop_session")
        | ("spatial.arkit", "stop_session")
        | ("spatial.arcore", "stop_session")
        | ("spatial.vision_pro", "stop_session")
        | ("spatial.magic_leap", "stop_session")
        | ("spatial.openxr", "stop_session")
            if registry.has_capability("spatial.session") =>
        {
            let session_id = wearable_device_arg(args);
            registry
                .with_spatial_session(&key, |provider| provider.stop_session(session_id))
                .map(|result| {
                    let failed = result.is_err();
                    record_call(
                        telemetry,
                        mission_trace,
                        sim_time_ms,
                        &key,
                        category,
                        module_path,
                        function_name,
                        started,
                        failed,
                    );
                    if result.is_ok() {
                        ok_int()
                    } else {
                        RuntimeValue::Void
                    }
                })
        }
        ("hri.voice", "poll_events")
        | ("hri.gesture", "poll_events")
        | ("hri.eye", "poll_events")
            if registry.has_capability("hri.input") =>
        {
            registry
                .with_hri_input(&key, |provider| provider.poll_events())
                .map(|result| {
                    let failed = result.is_err();
                    record_call(
                        telemetry,
                        mission_trace,
                        sim_time_ms,
                        &key,
                        category,
                        module_path,
                        function_name,
                        started,
                        failed,
                    );
                    result
                        .map(|events| RuntimeValue::Number {
                            value: events.len() as f64,
                            unit: spanda_ast::nodes::UnitKind::None,
                        })
                        .unwrap_or(RuntimeValue::Void)
                })
        }
        ("spatial.hololens", "subscribe_overlay") if registry.has_capability("hri.overlay") => {
            let layer = wearable_device_arg(args);
            let device_id = args
                .get(1)
                .and_then(|value| match value {
                    RuntimeValue::String { value } => Some(value.as_str()),
                    _ => None,
                })
                .unwrap_or("");
            registry
                .with_overlay(&key, |provider| {
                    provider.subscribe_overlay(layer, device_id)
                })
                .map(|result| {
                    let failed = result.is_err();
                    record_call(
                        telemetry,
                        mission_trace,
                        sim_time_ms,
                        &key,
                        category,
                        module_path,
                        function_name,
                        started,
                        failed,
                    );
                    if result.is_ok() {
                        ok_int()
                    } else {
                        RuntimeValue::Void
                    }
                })
        }
        ("sim.gazebo", "step") | ("sim.webots", "step")
            if registry.has_capability("simulation.step") =>
        {
            registry.with_simulation(&key, |provider| provider.step(10.0));
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                category,
                module_path,
                function_name,
                started,
                false,
            );
            Some(ok_int())
        }
        ("robotics.fleet", "dispatch") if registry.has_capability("fleet.orchestrate") => registry
            .with_fleet(&key, |provider| {
                let task = args.first().cloned().unwrap_or(RuntimeValue::Void);
                provider.dispatch_task("default", task)
            })
            .map(|result| {
                let failed = result.is_err();
                record_call(
                    telemetry,
                    mission_trace,
                    sim_time_ms,
                    &key,
                    category,
                    module_path,
                    function_name,
                    started,
                    failed,
                );
                ok_int()
            }),
        ("communication.mqtt", "publish_topic") if registry.has_capability("mqtt.publish") => {
            let topic = string_arg(args, 0);
            let payload = string_arg(args, 1);
            let transport_key = transport_registry_key(package);
            registry.with_transport(&transport_key, |transport| {
                transport.publish(
                    &topic,
                    "std_msgs/String",
                    RuntimeValue::String { value: payload },
                );
            });
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &transport_key,
                "transport",
                module_path,
                function_name,
                started,
                false,
            );
            Some(ok_int())
        }
        ("deploy.ota", "rollout") if registry.has_capability("deploy.rollout") => {
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                category,
                module_path,
                function_name,
                started,
                false,
            );
            Some(ok_int())
        }
        ("provenance.ledger", "append") if registry.has_capability("audit.append") => registry
            .with_ledger(&key, |provider| {
                let record = args.first().cloned().unwrap_or(RuntimeValue::Void);
                provider.append(record)
            })
            .map(|result| {
                let failed = result.is_err();
                record_call(
                    telemetry,
                    mission_trace,
                    sim_time_ms,
                    &key,
                    category,
                    module_path,
                    function_name,
                    started,
                    failed,
                );
                ok_int()
            }),
        ("iot.device", "register") if registry.has_capability("iot.device") => {
            let device = IoTDevice {
                id: string_arg(args, 0),
                protocol: string_arg(args, 1),
                topic: args.get(2).and_then(|v| match v {
                    RuntimeValue::String { value } => Some(value.clone()),
                    _ => None,
                }),
            };
            let failed = register_device(device).is_err();
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                "iot",
                module_path,
                function_name,
                started,
                failed,
            );
            if failed {
                None
            } else {
                Some(ok_int())
            }
        }
        ("iot.telemetry", "publish") if registry.has_capability("iot.telemetry") => {
            publish_telemetry(Telemetry {
                device_id: string_arg(args, 0),
                metric: string_arg(args, 1),
                value: args.get(2).cloned().unwrap_or(RuntimeValue::Void),
                timestamp_ms: sim_time_ms,
            });
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                "iot",
                module_path,
                function_name,
                started,
                false,
            );
            Some(ok_int())
        }
        ("iot.command", "send") if registry.has_capability("iot.command") => {
            let command = Command {
                device_id: string_arg(args, 0),
                action: string_arg(args, 1),
                payload: args.get(2).cloned().unwrap_or(RuntimeValue::Void),
            };
            let failed = send_command(command).is_err();
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                "iot",
                module_path,
                function_name,
                started,
                failed,
            );
            if failed {
                None
            } else {
                Some(ok_int())
            }
        }
        ("iot.shadow", "update") if registry.has_capability("iot.shadow") => {
            update_shadow(DeviceShadow {
                device_id: string_arg(args, 0),
                desired: args.get(1).cloned().unwrap_or(RuntimeValue::Void),
                reported: args.get(2).cloned().unwrap_or(RuntimeValue::Void),
            });
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                "iot",
                module_path,
                function_name,
                started,
                false,
            );
            Some(ok_int())
        }
        ("iot.modbus", "read_register") if registry.has_capability("iot.modbus") => {
            let address = number_arg(args, 0) as u16;
            let value = read_modbus_register(address);
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                "iot",
                module_path,
                function_name,
                started,
                false,
            );
            Some(RuntimeValue::Number {
                value,
                unit: spanda_ast::nodes::UnitKind::None,
            })
        }
        ("iot.opcua", "read_node") if registry.has_capability("iot.opcua") => {
            let node = string_arg(args, 0);
            let value = read_opcua_node(&node).unwrap_or_else(|| "unknown".into());
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                "iot",
                module_path,
                function_name,
                started,
                false,
            );
            Some(RuntimeValue::String { value })
        }
        ("iot.zigbee", "read_attribute") if registry.has_capability("iot.zigbee") => {
            let device = string_arg(args, 0);
            let cluster = string_arg(args, 1);
            let value = read_zigbee_attribute(&device, &cluster);
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                "iot",
                module_path,
                function_name,
                started,
                false,
            );
            Some(RuntimeValue::String { value })
        }
        ("iot.lora", "read_payload") if registry.has_capability("iot.lora") => {
            let device_id = string_arg(args, 0);
            let value = read_lora_payload(&device_id);
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                "iot",
                module_path,
                function_name,
                started,
                false,
            );
            Some(RuntimeValue::String { value })
        }
        ("iot.matter", "read_cluster") if registry.has_capability("iot.matter") => {
            let node = string_arg(args, 0);
            let cluster = string_arg(args, 1);
            let value = read_matter_cluster(&node, &cluster);
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                "iot",
                module_path,
                function_name,
                started,
                false,
            );
            Some(RuntimeValue::Number {
                value,
                unit: spanda_ast::nodes::UnitKind::None,
            })
        }
        ("iot.canbus", "read_frame") if registry.has_capability("iot.canbus") => {
            let can_id = number_arg(args, 0) as u32;
            let value = read_canbus_frame(can_id);
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                "iot",
                module_path,
                function_name,
                started,
                false,
            );
            Some(RuntimeValue::Number {
                value,
                unit: spanda_ast::nodes::UnitKind::None,
            })
        }
        ("sensors.radar", "read") if registry.has_capability("sensors.radar.read") => {
            let sensor = if args.is_empty() {
                "front-radar".to_string()
            } else {
                string_arg(args, 0)
            };
            let value = read_radar_distance(&sensor);
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                "sensors",
                module_path,
                function_name,
                started,
                false,
            );
            Some(RuntimeValue::Number {
                value,
                unit: spanda_ast::nodes::UnitKind::None,
            })
        }
        ("sensors.lidar", "read") if registry.has_capability("sensors.lidar.read") => {
            let sensor = if args.is_empty() {
                "front-lidar".to_string()
            } else {
                string_arg(args, 0)
            };
            let value = read_lidar_distance(&sensor);
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                "sensors",
                module_path,
                function_name,
                started,
                false,
            );
            Some(RuntimeValue::Number {
                value,
                unit: spanda_ast::nodes::UnitKind::None,
            })
        }
        ("sensors.ultrasonic", "read") if registry.has_capability("sensors.ultrasonic.read") => {
            let sensor = if args.is_empty() {
                "ultrasonic-array".to_string()
            } else {
                string_arg(args, 0)
            };
            let value = read_ultrasonic_distance(&sensor);
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                "sensors",
                module_path,
                function_name,
                started,
                false,
            );
            Some(RuntimeValue::Number {
                value,
                unit: spanda_ast::nodes::UnitKind::None,
            })
        }
        ("assurance.anomaly", "scan_learned")
            if registry.has_capability("assurance.anomaly.scan") =>
        {
            let detector = string_arg(args, 0);
            let observed = if args.len() > 1 {
                number_arg(args, 1)
            } else {
                1.0
            };
            let volatility = if args.len() > 2 {
                number_arg(args, 2)
            } else {
                0.0
            };
            let score = scan_learned_score(&detector, observed, volatility);
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                "assurance",
                module_path,
                function_name,
                started,
                false,
            );
            Some(RuntimeValue::Number {
                value: score,
                unit: spanda_ast::nodes::UnitKind::None,
            })
        }
        ("assurance.anomaly", "backend_name")
            if registry.has_capability("assurance.anomaly.scan") =>
        {
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                "assurance",
                module_path,
                function_name,
                started,
                false,
            );
            Some(RuntimeValue::String {
                value: "assurance.anomaly".into(),
            })
        }
        ("assurance.fusion", "weight_for_sensor")
            if registry.has_capability("assurance.fusion.weight") =>
        {
            let sensor_type = string_arg(args, 0);
            let value = weight_for_sensor_type(&sensor_type);
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                "assurance",
                module_path,
                function_name,
                started,
                false,
            );
            Some(RuntimeValue::Number {
                value,
                unit: spanda_ast::nodes::UnitKind::None,
            })
        }
        ("assurance.fusion", "confidence_for_types")
            if registry.has_capability("assurance.fusion.weight") =>
        {
            let types_csv = string_arg(args, 0);
            let types: Vec<&str> = types_csv
                .split(',')
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .collect();
            let value = weighted_confidence(&types);
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                "assurance",
                module_path,
                function_name,
                started,
                false,
            );
            Some(RuntimeValue::Number {
                value,
                unit: spanda_ast::nodes::UnitKind::None,
            })
        }
        ("assurance.fusion", "backend_name")
            if registry.has_capability("assurance.fusion.weight") =>
        {
            record_call(
                telemetry,
                mission_trace,
                sim_time_ms,
                &key,
                "assurance",
                module_path,
                function_name,
                started,
                false,
            );
            Some(RuntimeValue::String {
                value: "assurance.fusion".into(),
            })
        }
        _ => None,
    };

    dispatched
}
