//! `robot.*` runtime method dispatch for the interpreter.
//!

use super::{
    get_string, pose_from_state, velocity_from_state, Interpreter, RobotBackend, RuntimeValue,
};
use spanda_ast::nodes::Expr;
use spanda_comm::CommBus;
use spanda_error::SpandaError;
use spanda_safety::Pose2d;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn eval_robot_method(
        &mut self,
        method: &str,
        args: &[Expr],
        _named_args: &[spanda_ast::nodes::NamedArg],
    ) -> Result<RuntimeValue, SpandaError> {
        let state = self.backend.get_state();
        match method {
            "pose" => Ok(pose_from_state(&state.pose)),
            "velocity" => Ok(velocity_from_state(&state.velocity)),
            "in_zone" => {
                let zone_name = args
                    .first()
                    .map(|e| self.eval_expr(e))
                    .transpose()?
                    .map(|v| get_string(&v, ""))
                    .unwrap_or_default();
                let pose2d = Pose2d {
                    x: state.pose.x,
                    y: state.pose.y,
                };
                let in_zone = self
                    .safety_monitor
                    .as_ref()
                    .map(|m| m.is_in_zone(&zone_name, &pose2d))
                    .unwrap_or(false);
                Ok(RuntimeValue::Bool { value: in_zone })
            }
            "in_geofence" => {
                let name = args
                    .first()
                    .map(|e| self.eval_expr(e))
                    .transpose()?
                    .map(|v| get_string(&v, ""))
                    .unwrap_or_default();
                let (lat, lon) = self.current_gps_lat_lon();
                let inside = self.geofences.iter().any(|f| {
                    f.name == name
                        && self.host.geofence_contains(
                            f.center_lat,
                            f.center_lon,
                            f.radius_m,
                            lat,
                            lon,
                        )
                });
                Ok(RuntimeValue::Bool { value: inside })
            }
            "connectivity_link" => Ok(RuntimeValue::String {
                value: self.active_connectivity_link.clone(),
            }),
            "sim_identity" => {
                self.security
                    .require_operation("cellular.sim_identity")
                    .map_err(|e| self.security_error(e, 0))?;
                let cellular_active = self.host.is_modem_bearer(&self.active_connectivity_link);
                let outage = self.comm_bus.active_faults().iter().any(|f| {
                    matches!(
                        f.as_str(),
                        "LteOutage" | "SatelliteOutage" | "NetworkOutage"
                    )
                }) || self.hardware_monitor.injected_faults().iter().any(|f| {
                    f == "LteOutage" || f == "SatelliteOutage" || f == "NetworkOutage"
                });
                Ok(self.host.runtime_sim_identity(
                    &self.active_connectivity_link,
                    cellular_active && !outage,
                ))
            }
            "identity" => self
                .env
                .get("identity")
                .cloned()
                .ok_or_else(|| SpandaError::Runtime {
                    message: "robot has no identity — declare an identity block".into(),
                    line: 0,
                }),
            _ => Ok(RuntimeValue::Void),
        }
    }
}
