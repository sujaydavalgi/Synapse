//! Connectivity trigger, geofence, and failover logic for the interpreter.
//!

use super::{Interpreter, RobotBackend};
use spanda_comm::CommBus;
use spanda_error::SpandaError;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn load_connectivity_metadata(
        &mut self,
        geofences: &[spanda_ast::foundations::GeofenceDecl],
        policies: &[spanda_ast::foundations::ConnectivityPolicyDecl],
    ) {
        use crate::connectivity_positioning::{connectivity_policy_from_decl, geofence_from_decl};
        self.geofences = geofences.iter().map(geofence_from_decl).collect();
        self.connectivity_policies = policies.iter().map(connectivity_policy_from_decl).collect();
        if let Some(policy) = self.connectivity_policies.first() {
            self.active_connectivity_link = policy.preferred.clone();
            self.default_transport = self
                .host
                .connectivity_link_to_transport(&self.active_connectivity_link);
        }
    }

    pub(super) fn dispatch_connectivity_trigger(
        &mut self,
        domain: &str,
        event: &str,
    ) -> Result<(), SpandaError> {
        let key = format!("{domain}.{event}");
        let ids: Vec<usize> = self
            .trigger_registry
            .handlers_for_connectivity(domain, event)
            .iter()
            .map(|h| h.id)
            .collect();
        if ids.is_empty() {
            return Ok(());
        }
        self.log(format!("connectivity trigger: {key}"));
        self.execute_trigger_handlers(ids)
    }

    pub(super) fn dispatch_geofence_trigger(
        &mut self,
        name: &str,
        phase: &str,
    ) -> Result<(), SpandaError> {
        let ids: Vec<usize> = self
            .trigger_registry
            .handlers_for_geofence(name, phase)
            .iter()
            .map(|h| h.id)
            .collect();
        if ids.is_empty() {
            return Ok(());
        }
        self.log(format!("geofence trigger: {name} {phase}"));
        self.execute_trigger_handlers(ids)
    }

    pub(super) fn run_connectivity_triggers(&mut self) -> Result<(), SpandaError> {
        for fault in self.comm_bus.active_faults() {
            if let Some((domain, event)) = self.host.fault_to_connectivity(&fault) {
                let key = format!("fault:{domain}.{event}");
                if self.connectivity_events_seen.insert(key) {
                    self.apply_connectivity_failover(domain, event);
                    self.dispatch_connectivity_trigger(domain, event)?;
                }
            }
        }
        let gps_ok = !self
            .hardware_monitor
            .injected_faults()
            .iter()
            .any(|f| f == "GpsFailure" || f == "GPSLost");
        if self.gps_available && !gps_ok {
            self.gps_available = false;
            self.dispatch_connectivity_trigger("gps", "lost")?;
        } else if !self.gps_available && gps_ok {
            self.gps_available = true;
            self.dispatch_connectivity_trigger("gps", "acquired")?;
            self.dispatch_sensor_event_trigger("gps", "fix")?;
        }
        Ok(())
    }

    fn active_connectivity_faults(&self) -> std::collections::HashSet<String> {
        let mut faults = self.hardware_monitor.injected_faults().clone();
        for fault in self.comm_bus.active_faults() {
            faults.insert(fault);
        }
        faults
    }

    fn activate_connectivity_link(&mut self, policy_name: &str, link: &str, reason: &str) {
        self.active_connectivity_link = link.to_string();
        self.default_transport = self.host.connectivity_link_to_transport(link);
        self.comm_bus.reconnect_transport(self.default_transport);
        self.log(format!(
            "connectivity_policy '{policy_name}': {reason} (transport {:?})",
            self.default_transport
        ));
    }

    fn apply_connectivity_failover(&mut self, domain: &str, event: &str) {
        if domain != "network" || event != "disconnected" {
            return;
        }
        let faults = self.active_connectivity_faults();
        let policies: Vec<_> = self
            .connectivity_policies
            .iter()
            .map(|policy| {
                (
                    policy.name.clone(),
                    policy.preferred.clone(),
                    policy.fallback.clone(),
                    policy.emergency.clone(),
                )
            })
            .collect();
        for (policy_name, preferred, fallback, emergency) in policies {
            if self.active_connectivity_link == preferred {
                self.activate_connectivity_link(
                    &policy_name,
                    &fallback,
                    &format!("failover {preferred} -> {fallback}"),
                );
            }

            if let Some(em) = &emergency {
                if self.active_connectivity_link != *em
                    && self
                        .host
                        .is_link_impaired(&self.active_connectivity_link, &faults)
                {
                    self.activate_connectivity_link(
                        &policy_name,
                        em,
                        &format!("emergency link {em}"),
                    );
                }
            }
        }
    }

    pub(super) fn current_gps_lat_lon(&self) -> (f64, f64) {
        let state = self.backend.get_state();
        let faults = self.hardware_monitor.injected_faults();
        let (lat, lon, _) = self.host.apply_gps_position_faults(
            faults,
            state.pose.x,
            state.pose.y,
            self.sim_time_ms,
        );
        (lat, lon)
    }

    pub(super) fn run_geofence_triggers(&mut self) -> Result<(), SpandaError> {
        if self.geofences.is_empty() {
            return Ok(());
        }
        let (lat, lon) = self.current_gps_lat_lon();
        let mut entered = Vec::new();
        let mut exited = Vec::new();
        for fence in &self.geofences {
            let inside = self.host.geofence_contains(
                fence.center_lat,
                fence.center_lon,
                fence.radius_m,
                lat,
                lon,
            );
            let was_inside = self.geofence_active.contains(&fence.name);
            if inside && !was_inside {
                self.geofence_active.insert(fence.name.clone());
                entered.push(fence.name.clone());
            } else if !inside && was_inside {
                self.geofence_active.remove(&fence.name);
                exited.push(fence.name.clone());
            } else if inside {
                self.geofence_active.insert(fence.name.clone());
            }
        }
        for name in entered {
            self.dispatch_geofence_trigger(&name, "entered")?;
        }
        for name in exited {
            self.dispatch_geofence_trigger(&name, "exited")?;
        }
        Ok(())
    }
}
