//! Sensor reading and observe-fusion helpers for the interpreter.
//!

use super::{
    pose_from_state, Interpreter, IntoSpandaError, RobotBackend, RuntimeError, RuntimeValue,
};
use spanda_ast::nodes::UnitKind;
use spanda_error::SpandaError;
use spanda_lib_registry::{get_sensor_driver, read_with_driver, DriverContext, SimState};
use spanda_runtime::fusion::{parse_fusion_input, weight_for_sensor_type, weighted_confidence};
use std::collections::HashMap;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn read_sensor_value(
        &mut self,
        target: &RuntimeValue,
    ) -> Result<RuntimeValue, SpandaError> {
        // Description:
        //     Read sensor value.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     arge: &RuntimeValue
        //         Caller-supplied arge.
        //
        // Outputs:
        //     result: Result<RuntimeValue, SpandaError>
        //         Return value from `read_sensor_value`.
        //
        // Example:
        //     let result = spanda_interpreter::runtime_sensors::read_sensor_value(&mut self, arge);

        // Compute RuntimeValue for the following logic.
        let RuntimeValue::Sensor {
            name,
            sensor_type,
            library,
            hal_binding,
            topic,
        } = target
        // Handle any remaining cases.
        else {
            return Ok(RuntimeValue::Void);
        };
        let state = self.backend.get_state();
        let reading = if let Some(lib) = library {
            // Emit output when get sensor driver provides a driver.
            if let Some(driver) = get_sensor_driver(lib, sensor_type) {
                let ctx = DriverContext {
                    hal: Some(&self.hal),
                    hal_binding: hal_binding.as_deref(),
                    topic: topic.as_deref(),
                    sim_state: Some(SimState {
                        pose: state.pose.clone(),
                    }),
                };
                read_with_driver(&driver, &ctx)
            } else {
                self.backend
                    .read_sensor(name, sensor_type, topic.as_deref())
            }
        } else {
            self.backend
                .read_sensor(name, sensor_type, topic.as_deref())
        };
        let reading = if matches!(sensor_type.as_str(), "GPS" | "GNSS") {
            self.host.apply_gps_reading_faults(
                reading,
                self.hardware_monitor.injected_faults(),
                state.pose.x,
                state.pose.y,
                self.sim_time_ms,
            )
        } else {
            reading
        };
        self.hardware_monitor
            .record_sensor_reading(name, sensor_type, &reading);
        let _ = spanda_telemetry_store::record_sensor_reading(
            name,
            sensor_type,
            &reading,
            self.sim_time_ms,
            None,
        );
        Ok(reading)
    }

    pub(super) fn read_fused_observation(
        &mut self,
        input_paths: &[String],
        estimator: Option<&str>,
    ) -> Result<RuntimeValue, SpandaError> {
        // Description:
        //     Read fused observation.
        //
        // Inputs:
        //     &mut self: input value
        //         Caller-supplied &mut self.
        //     input_paths: &[String]
        //         Caller-supplied input paths.
        //     estimator: Option<&str>
        //         Caller-supplied estimator.
        //
        // Outputs:
        //     result: Result<RuntimeValue, SpandaError>
        //         Return value from `read_fused_observation`.
        //
        // Example:

        //     let result = spanda_interpreter::runtime_sensors::read_fused_observation(&mut self, input_paths, estimator);

        let mut fields = HashMap::new();
        let mut sensor_types = Vec::new();
        let mut sources = Vec::new();

        // Read each declared fusion input and record its sensor type weight.
        for input_path in input_paths {
            let (sensor_name, field) = parse_fusion_input(input_path);
            let sensor_val = self.env.get(sensor_name).cloned().ok_or_else(|| {
                RuntimeError::new(format!("Unknown observe sensor '{sensor_name}'"), 0)
                    .into_spanda()
            })?;
            let sensor_type = if let RuntimeValue::Sensor { sensor_type, .. } = &sensor_val {
                sensor_type.clone()
            } else {
                "Unknown".into()
            };
            sensor_types.push(sensor_type.clone());
            let reading = self.read_sensor_value(&sensor_val)?;
            fields.insert(sensor_name.to_string(), reading);
            sources.push(if let Some(field) = field {
                format!("{sensor_name}.{field}")
            } else {
                sensor_name.to_string()
            });
            let _ = weight_for_sensor_type(&sensor_type);
        }

        let state = self.backend.get_state();
        fields.insert("pose".into(), pose_from_state(&state.pose));
        fields.insert(
            "count".into(),
            RuntimeValue::Number {
                value: input_paths.len() as f64,
                unit: UnitKind::None,
            },
        );
        if let Some(name) = estimator {
            fields.insert(
                "estimator".into(),
                RuntimeValue::String {
                    value: name.to_string(),
                },
            );
        }
        let confidence =
            weighted_confidence(&sensor_types.iter().map(String::as_str).collect::<Vec<_>>());
        fields.insert(
            "confidence".into(),
            RuntimeValue::Number {
                value: confidence,
                unit: UnitKind::None,
            },
        );
        fields.insert(
            "sources".into(),
            RuntimeValue::String {
                value: sources.join(", "),
            },
        );
        fields.insert(
            "state_estimate".into(),
            RuntimeValue::Object {
                type_name: "StateEstimate".into(),
                fields: HashMap::from([
                    (
                        "pose".into(),
                        fields.get("pose").cloned().unwrap_or(RuntimeValue::Null),
                    ),
                    (
                        "confidence".into(),
                        RuntimeValue::Number {
                            value: confidence,
                            unit: UnitKind::None,
                        },
                    ),
                    (
                        "sources".into(),
                        fields.get("sources").cloned().unwrap_or(RuntimeValue::Null),
                    ),
                ]),
            },
        );
        let fused = RuntimeValue::Object {
            type_name: "FusedObservation".into(),
            fields,
        };
        if self.world_model_fusion_hook {
            let belief = self.world_model.update(&fused);
            self.log(format!(
                "world_model: fused observation -> belief {belief:.2}"
            ));
        }
        Ok(fused)
    }
}
