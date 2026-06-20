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
        self.sensors.push((name.into(), sensor_type.into()));
    }

    pub fn register_actuator(&mut self, name: impl Into<String>, actuator_type: impl Into<String>) {
        self.actuators.push((name.into(), actuator_type.into()));
    }

    pub fn inject_fault(&mut self, fault: impl Into<String>) {
        self.injected_faults.insert(fault.into());
    }

    pub fn sensor_event_for_type(sensor_type: &str) -> Option<&'static str> {
        match sensor_type {
            "Lidar" => Some("LidarFailure"),
            "Camera" | "VisionCamera" | "RGBCamera" => Some("CameraFailure"),
            "IMU" | "BoschBNO055" => Some("ImuFailure"),
            "GPS" => Some("GpsFailure"),
            _ => None,
        }
    }

    pub fn actuator_event_for_type(actuator_type: &str) -> Option<&'static str> {
        match actuator_type {
            "DifferentialDrive" | "Wheels" => Some("DriveFailure"),
            "Arm" | "Manipulator" => Some("ActuatorFailure"),
            _ => None,
        }
    }

    fn fault_matches_sensor(fault: &str, sensor_type: &str, sensor_name: &str) -> bool {
        let fault_lower = fault.to_ascii_lowercase();
        let name_lower = sensor_name.to_ascii_lowercase();
        if fault == sensor_name || fault_lower == name_lower {
            return true;
        }
        if let Some(event) = Self::sensor_event_for_type(sensor_type) {
            if fault == event || fault_lower == event.to_ascii_lowercase() {
                return true;
            }
        }
        match sensor_type {
            "Lidar" => fault_lower.contains("lidar"),
            "Camera" | "VisionCamera" | "RGBCamera" => fault_lower.contains("camera"),
            "IMU" | "BoschBNO055" => fault_lower.contains("imu"),
            "GPS" => fault_lower.contains("gps"),
            _ => false,
        }
    }

    pub fn record_sensor_reading(&mut self, name: &str, sensor_type: &str, reading: &RuntimeValue) {
        if Self::reading_failed(reading) {
            let count = self.read_failures.entry(name.to_string()).or_insert(0);
            *count += 1;
            if *count >= FAILURE_THRESHOLD {
                if let Some(event) = Self::sensor_event_for_type(sensor_type) {
                    self.active_events.insert(event.to_string());
                }
            }
        } else {
            self.read_failures.remove(name);
        }
    }

    fn reading_failed(reading: &RuntimeValue) -> bool {
        matches!(
            reading,
            RuntimeValue::Null | RuntimeValue::Void | RuntimeValue::Result { ok: false, .. }
        )
    }

    pub fn evaluate_injected_faults(&mut self) {
        for fault in self.injected_faults.clone() {
            for (name, sensor_type) in &self.sensors {
                if Self::fault_matches_sensor(&fault, sensor_type, name) {
                    if let Some(event) = Self::sensor_event_for_type(sensor_type) {
                        self.active_events.insert(event.to_string());
                    }
                }
            }
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
        self.evaluate_injected_faults();
        let mut new_events = Vec::new();
        for event in &self.active_events {
            if self.dispatched_events.insert(event.clone()) {
                new_events.push(event.clone());
            }
        }
        new_events
    }

    pub fn clear_event(&mut self, event: &str) {
        self.active_events.remove(event);
        self.dispatched_events.remove(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injected_lidar_fault_maps_to_event() {
        let mut monitor = HardwareMonitor::default();
        monitor.register_sensor("lidar", "Lidar");
        monitor.inject_fault("LidarFailure");
        let events = monitor.poll_new_events();
        assert!(events.contains(&"LidarFailure".to_string()));
    }
}
