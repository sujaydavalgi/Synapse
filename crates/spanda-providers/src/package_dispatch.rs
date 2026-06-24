//! Runtime dispatch from official package module exports to provider registry backends.
//!
use crate::iot_hub::{
    number_arg, publish_telemetry, read_canbus_frame, read_lora_payload, read_matter_cluster,
    read_modbus_register, read_opcua_node, read_zigbee_attribute, register_device, send_command,
    string_arg, update_shadow,
};
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
        "assurance.evidence" => Some("spanda-assurance"),
        "assurance.knowledge" => Some("spanda-knowledge-model"),
        "assurance.anomaly" => Some("spanda-anomaly"),
        "assurance.diagnosis" => Some("spanda-diagnosis"),
        "assurance.prognostics" => Some("spanda-prognostics"),
        "assurance.mission" => Some("spanda-mission-planning"),
        "assurance.resilience" => Some("spanda-resilience"),
        _ => None,
    }
}

fn project_provider_key(package: &str) -> String {
    format!("{package}::project")
}

fn ok_int() -> RuntimeValue {
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
        ("assurance.anomaly", "scan_learned") if registry.has_capability("assurance.anomaly.scan") => {
            let observed = if args.len() > 1 {
                number_arg(args, 1)
            } else {
                1.0
            };
            let score = if observed < 0.85 { 1.0 } else { 0.0 };
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
        ("assurance.anomaly", "backend_name") if registry.has_capability("assurance.anomaly.scan") => {
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
        _ => None,
    };

    dispatched
}
