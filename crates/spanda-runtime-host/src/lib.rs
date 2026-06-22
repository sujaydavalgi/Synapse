//! Default `RuntimeHost` wiring for navigation, SLAM, and connectivity.
//!
pub mod nav2_adapter;
pub mod slam_adapter;

use spanda_comm::TransportKind;
use spanda_connectivity::adapter_bridge;
use spanda_runtime::{RuntimeHost, RuntimeValue};
use std::collections::HashSet;

/// Default host wiring domain adapters from `spanda-core` into `spanda-runtime`.
pub struct CoreRuntimeHost;

impl RuntimeHost for CoreRuntimeHost {
    fn slam_import_known(&self, path: &str) -> bool {
        slam_adapter::slam_import_paths().contains(&path)
    }

    fn navigation_import_known(&self, path: &str) -> bool {
        nav2_adapter::nav2_import_paths().contains(&path)
    }

    fn invoke_nav2_bridge(&self, goal: &str) -> Option<String> {
        adapter_bridge::invoke_nav2_bridge(goal)
    }

    fn invoke_slam_bridge(&self, op: &str) -> Option<String> {
        adapter_bridge::invoke_slam_bridge(op)
    }

    fn connectivity_link_to_transport(&self, link: &str) -> TransportKind {
        spanda_connectivity_runtime::connectivity_link_to_transport(link)
    }

    fn hardware_event_to_connectivity(&self, event: &str) -> Option<(&'static str, &'static str)> {
        spanda_connectivity::hardware_event_to_connectivity(event)
    }

    fn fault_to_connectivity(&self, fault: &str) -> Option<(&'static str, &'static str)> {
        spanda_connectivity::fault_to_connectivity(fault)
    }

    fn is_link_impaired(&self, link: &str, faults: &HashSet<String>) -> bool {
        spanda_connectivity::is_link_impaired(link, faults)
    }

    fn apply_gps_position_faults(
        &self,
        faults: &HashSet<String>,
        true_lat: f64,
        true_lon: f64,
        sim_time_ms: f64,
    ) -> (f64, f64, f64) {
        spanda_connectivity::apply_gps_position_faults(faults, true_lat, true_lon, sim_time_ms)
    }

    fn geofence_contains(
        &self,
        center_lat: f64,
        center_lon: f64,
        radius_m: f64,
        lat: f64,
        lon: f64,
    ) -> bool {
        let fence = spanda_connectivity::GeofenceRuntime {
            name: "runtime".into(),
            center_lat,
            center_lon,
            radius_m,
        };
        spanda_connectivity::geofence_contains(&fence, lat, lon)
    }

    fn is_modem_bearer(&self, link: &str) -> bool {
        spanda_connectivity::is_modem_bearer(link)
    }

    fn apply_gps_reading_faults(
        &self,
        reading: RuntimeValue,
        faults: &HashSet<String>,
        true_lat: f64,
        true_lon: f64,
        sim_time_ms: f64,
    ) -> RuntimeValue {
        spanda_connectivity_runtime::apply_gps_reading_faults(
            reading,
            faults,
            true_lat,
            true_lon,
            sim_time_ms,
        )
    }

    fn runtime_sim_identity(&self, link: &str, attested: bool) -> RuntimeValue {
        spanda_connectivity_runtime::runtime_sim_identity(link, attested)
    }
}

/// Shared core runtime host instance for interpreter wiring.
pub fn core_runtime_host() -> &'static CoreRuntimeHost {
    &CoreRuntimeHost
}