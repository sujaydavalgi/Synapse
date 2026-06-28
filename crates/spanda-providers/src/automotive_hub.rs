//! In-memory automotive sensor distance store and live-read dispatch for ADAS packages.
//!
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static AUTOMOTIVE_HUB: OnceLock<Mutex<AutomotiveHub>> = OnceLock::new();

fn hub() -> &'static Mutex<AutomotiveHub> {
    AUTOMOTIVE_HUB.get_or_init(|| Mutex::new(AutomotiveHub::default()))
}

/// Shared in-memory automotive sensor readings for sim and stub integrations.
#[derive(Default)]
pub struct AutomotiveHub {
    radar_distances: HashMap<String, f64>,
    lidar_distances: HashMap<String, f64>,
    ultrasonic_distances: HashMap<String, f64>,
}

impl AutomotiveHub {
    pub fn seed_automotive_demo(&mut self) {
        self.radar_distances.insert("front-radar".into(), 25.0);
        self.radar_distances.insert("rear-radar".into(), 8.0);
        self.lidar_distances.insert("front-lidar".into(), 12.0);
        self.lidar_distances.insert("rear-lidar".into(), 3.5);
        self.ultrasonic_distances
            .insert("ultrasonic-array".into(), 1.2);
        self.ultrasonic_distances
            .insert("rear-ultrasonic".into(), 0.45);
    }

    pub fn read_radar(&self, sensor_id: &str) -> f64 {
        let key = if sensor_id.is_empty() {
            "front-radar"
        } else {
            sensor_id
        };
        self.radar_distances.get(key).copied().unwrap_or(20.0)
    }

    pub fn read_lidar(&self, sensor_id: &str) -> f64 {
        let key = if sensor_id.is_empty() {
            "front-lidar"
        } else {
            sensor_id
        };
        self.lidar_distances.get(key).copied().unwrap_or(10.0)
    }

    pub fn read_ultrasonic(&self, sensor_id: &str) -> f64 {
        let key = if sensor_id.is_empty() {
            "ultrasonic-array"
        } else {
            sensor_id
        };
        self.ultrasonic_distances.get(key).copied().unwrap_or(1.0)
    }
}

/// Seed demo automotive sensor distances for installed ADAS packages.
pub fn seed_automotive_demos() {
    hub().lock().unwrap().seed_automotive_demo();
}

/// Read radar range in metres (live bridge when enabled, otherwise hub stub).
pub fn read_radar_distance(sensor_id: &str) -> f64 {
    if let Some(value) = crate::iot_live::read_radar_distance_live(sensor_id) {
        return value;
    }
    hub().lock().unwrap().read_radar(sensor_id)
}

/// Read LiDAR range in metres (live bridge when enabled, otherwise hub stub).
pub fn read_lidar_distance(sensor_id: &str) -> f64 {
    if let Some(value) = crate::iot_live::read_lidar_distance_live(sensor_id) {
        return value;
    }
    hub().lock().unwrap().read_lidar(sensor_id)
}

/// Read ultrasonic range in metres (live bridge when enabled, otherwise hub stub).
pub fn read_ultrasonic_distance(sensor_id: &str) -> f64 {
    if let Some(value) = crate::iot_live::read_ultrasonic_distance_live(sensor_id) {
        return value;
    }
    hub().lock().unwrap().read_ultrasonic(sensor_id)
}
