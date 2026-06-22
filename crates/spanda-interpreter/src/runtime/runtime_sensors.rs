//! Sensor reading and observe-fusion helpers for the interpreter.
//!

use super::{pose_from_state, IntoSpandaError, Interpreter, RobotBackend, RuntimeError, RuntimeValue};
use spanda_ast::nodes::UnitKind;
use spanda_error::SpandaError;
use spanda_lib_registry::{get_sensor_driver, read_with_driver, DriverContext, SimState};
use std::collections::HashMap;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn read_sensor_value(&mut self, target: &RuntimeValue) -> Result<RuntimeValue, SpandaError> {
        // Read sensor value.
        //
        // Parameters:
        // - `self` — method receiver
        // - `target` — input value
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.read_sensor_value(target);

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
        Ok(reading)
    }

    pub(super) fn read_fused_observation(&mut self) -> Result<RuntimeValue, SpandaError> {
        // Read fused observation.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Success value on completion, or an error.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.read_fused_observation();

        // Compute sensors for the following logic.
        let sensors = self.fusion_sensors.clone();
        let mut fields = HashMap::new();

        // Process each sensor.
        for sensor_name in &sensors {
            let sensor_val = self.env.get(sensor_name).cloned().ok_or_else(|| {
                RuntimeError::new(format!("Unknown observe sensor '{sensor_name}'"), 0)
                    .into_spanda()
            })?;
            let reading = self.read_sensor_value(&sensor_val)?;
            fields.insert(sensor_name.clone(), reading);
        }
        let state = self.backend.get_state();
        fields.insert("pose".into(), pose_from_state(&state.pose));
        fields.insert(
            "count".into(),
            RuntimeValue::Number {
                value: sensors.len() as f64,
                unit: UnitKind::None,
            },
        );
        let confidence = if sensors.is_empty() {
            0.0
        } else {
            (sensors.len() as f64 / 4.0).min(1.0)
        };
        fields.insert(
            "confidence".into(),
            RuntimeValue::Number {
                value: confidence,
                unit: UnitKind::None,
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
                ]),
            },
        );
        Ok(RuntimeValue::Object {
            type_name: "FusedObservation".into(),
            fields,
        })
    }
}
