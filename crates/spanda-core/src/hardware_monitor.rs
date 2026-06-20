//! Runtime hardware health monitoring for hardware trigger dispatch.

use crate::runtime::RuntimeValue;
use std::collections::{HashMap, HashSet};

const FAILURE_THRESHOLD: u32 = 2;

/// Tracks sensor/actuator health and maps failures to hardware trigger event names.
#[derive(Debug, Default)]
pub struct HardwareMonitor {
    sensors: Vec<(String, String)>,
    actuators: Vec<(String, String)>,
    injected_faults: HashSet<String>,
    active_events: HashSet<String>,
    dispatched_events: HashSet<String>,
    read_failures: HashMap<String, u32>,
}

impl HardwareMonitor {
    pub fn register_sensor(&mut self, name: impl Into<String>, sensor_type: impl Into<String>) {
        // Register sensor.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `sensor_type` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.register_sensor(name, sensor_type);

        // Append into self.
        self.sensors.push((name.into(), sensor_type.into()));
    }

    pub fn register_actuator(&mut self, name: impl Into<String>, actuator_type: impl Into<String>) {
        // Register actuator.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `actuator_type` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.register_actuator(name, actuator_type);

        // Append into self.
        self.actuators.push((name.into(), actuator_type.into()));
    }

    pub fn inject_fault(&mut self, fault: impl Into<String>) {
        // Inject fault.
        //
        // Parameters:
        // - `self` — method receiver
        // - `fault` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.inject_fault(fault);

        // Append into self.
        self.injected_faults.insert(fault.into());
    }

    pub fn sensor_event_for_type(sensor_type: &str) -> Option<&'static str> {
        // Sensor event for type.
        //
        // Parameters:
        // - `sensor_type` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::hardware_monitor::sensor_event_for_type(sensor_type);

        // Match on sensor type and handle each case.
        match sensor_type {
            "Lidar" => Some("LidarFailure"),
            "Camera" | "VisionCamera" | "RGBCamera" => Some("CameraFailure"),
            "IMU" | "BoschBNO055" => Some("ImuFailure"),
            "GPS" => Some("GpsFailure"),
            _ => None,
        }
    }

    pub fn actuator_event_for_type(actuator_type: &str) -> Option<&'static str> {
        // Actuator event for type.
        //
        // Parameters:
        // - `actuator_type` — input value
        //
        // Returns:
        // Some value on success, otherwise none.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::hardware_monitor::actuator_event_for_type(actuator_type);

        // Match on actuator type and handle each case.
        match actuator_type {
            "DifferentialDrive" | "Wheels" => Some("DriveFailure"),
            "Arm" | "Manipulator" => Some("ActuatorFailure"),
            _ => None,
        }
    }

    fn fault_matches_sensor(fault: &str, sensor_type: &str, sensor_name: &str) -> bool {
        // Fault matches sensor.
        //
        // Parameters:
        // - `fault` — input value
        // - `sensor_type` — input value
        // - `sensor_name` — input value
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::hardware_monitor::fault_matches_sensor(fault, sensor_type, sensor_name);

        // Compute fault lower for the following logic.
        let fault_lower = fault.to_ascii_lowercase();
        let name_lower = sensor_name.to_ascii_lowercase();

        // Take the branch when fault equals sensor name || fault lower == name lower.
        if fault == sensor_name || fault_lower == name_lower {
            return true;
        }

        // Emit output when sensor event for type provides a event.
        if let Some(event) = Self::sensor_event_for_type(sensor_type) {
            // Take the branch when fault equals to ascii lowercase.
            if fault == event || fault_lower == event.to_ascii_lowercase() {
                return true;
            }
        }

        // Match on sensor type and handle each case.
        match sensor_type {
            "Lidar" => fault_lower.contains("lidar"),
            "Camera" | "VisionCamera" | "RGBCamera" => fault_lower.contains("camera"),
            "IMU" | "BoschBNO055" => fault_lower.contains("imu"),
            "GPS" => fault_lower.contains("gps"),
            _ => false,
        }
    }

    pub fn record_sensor_reading(&mut self, name: &str, sensor_type: &str, reading: &RuntimeValue) {
        // Record sensor reading.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — input value
        // - `sensor_type` — input value
        // - `reading` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.record_sensor_reading(name, sensor_type, reading);

        // take this path when Self::reading failed(reading).
        if Self::reading_failed(reading) {
            let count = self.read_failures.entry(name.to_string()).or_insert(0);
            *count += 1;

            // Take this path when *count >= FAILURE THRESHOLD.
            if *count >= FAILURE_THRESHOLD {
                // Emit output when sensor event for type provides a event.
                if let Some(event) = Self::sensor_event_for_type(sensor_type) {
                    self.active_events.insert(event.to_string());
                }
            }
        } else {
            self.read_failures.remove(name);
        }
    }

    fn reading_failed(reading: &RuntimeValue) -> bool {
        // Reading failed.
        //
        // Parameters:
        // - `reading` — input value
        //
        // Returns:
        // true or false.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::hardware_monitor::reading_failed(reading);

        // Produce matches! as the result.
        matches!(
            reading,
            RuntimeValue::Null | RuntimeValue::Void | RuntimeValue::Result { ok: false, .. }
        )
    }

    pub fn evaluate_injected_faults(&mut self) {
        // Evaluate injected faults.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.evaluate_injected_faults();

        // Inject each configured hardware fault.
        for fault in self.injected_faults.clone() {
            // Iterate over sensors with destructured elements.
            for (name, sensor_type) in &self.sensors {
                // Take this path when Self::fault matches sensor(&fault, sensor type, name).
                if Self::fault_matches_sensor(&fault, sensor_type, name) {
                    // Emit output when sensor event for type provides a event.
                    if let Some(event) = Self::sensor_event_for_type(sensor_type) {
                        self.active_events.insert(event.to_string());
                    }
                }
            }

            // Proceed only when is some is available.
            if Self::actuator_event_for_type(&fault).is_some()
                || fault.to_ascii_lowercase().contains("actuator")
                || fault.to_ascii_lowercase().contains("drive")
            {
                self.active_events.insert(fault.clone());
            }
        }
    }

    /// Returns newly detected hardware events to dispatch (edge-triggered).
    pub fn poll_new_events(&mut self) -> Vec<String> {
        // Poll new events.
        //
        // Parameters:
        // - `self` — method receiver
        //
        // Returns:
        // Vec<String>.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.poll_new_events();

        // Call evaluate injected faults on the current instance.
        self.evaluate_injected_faults();
        let mut new_events = Vec::new();

        // Process each active event.
        for event in &self.active_events {
            // Take this path when self.dispatched events.insert(event.clone()).
            if self.dispatched_events.insert(event.clone()) {
                new_events.push(event.clone());
            }
        }
        new_events
    }

    pub fn clear_event(&mut self, event: &str) {
        // Clear event.
        //
        // Parameters:
        // - `self` — method receiver
        // - `event` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.clear_event(event);

        // Call remove on the current instance.
        self.active_events.remove(event);
        self.dispatched_events.remove(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injected_lidar_fault_maps_to_event() {
        // Injected lidar fault maps to event.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = spanda_core::hardware_monitor::injected_lidar_fault_maps_to_event();

        let mut monitor = HardwareMonitor::default();
        monitor.register_sensor("lidar", "Lidar");
        monitor.inject_fault("LidarFailure");
        let events = monitor.poll_new_events();
        assert!(events.contains(&"LidarFailure".to_string()));
    }
}
