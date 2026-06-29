//! Injectable fleet tamper correlation runtime for mesh coordinators.
//!
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

/// Extension points for fleet-level tamper trace correlation.
pub trait FleetTamperRuntime: Send + Sync {
    /// Correlate tamper trace shards from all fleet members into a single JSON report.
    ///
    /// Parameters:
    /// - `fleet_name` — name of the fleet being correlated
    /// - `shards` — map of robot_id → serialised MissionTrace JSON
    ///
    /// Returns:
    /// Ok(fleet_tamper_report_json) or Err(error_message).
    ///
    /// Options:
    /// None.
    ///
    /// Example:
    /// let json = fleet_tamper_runtime().correlate_fleet_tamper_traces_json("Fleet", &shards).unwrap();
    fn correlate_fleet_tamper_traces_json(
        &self,
        fleet_name: &str,
        shards: &HashMap<String, String>,
    ) -> Result<String, String>;

    /// Format a serialised FleetTamperReport JSON into a human-readable text report.
    ///
    /// Parameters:
    /// - `report_json` — serialised FleetTamperReport JSON
    ///
    /// Returns:
    /// Formatted text report string.
    ///
    /// Options:
    /// None.
    ///
    /// Example:
    /// let text = fleet_tamper_runtime().format_fleet_tamper_report_json(json);
    fn format_fleet_tamper_report_json(&self, report_json: &str) -> String;
}

/// No-op fleet tamper runtime for tests and runs without a real tamper engine.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopFleetTamperRuntime;

impl FleetTamperRuntime for NoopFleetTamperRuntime {
    fn correlate_fleet_tamper_traces_json(
        &self,
        fleet_name: &str,
        _shards: &HashMap<String, String>,
    ) -> Result<String, String> {
        // Return an empty report when no tamper engine is wired.
        Ok(format!(
            r#"{{"fleet":"{fleet_name}","members":[],"anomalies":[],"risk_score":0.0}}"#
        ))
    }

    fn format_fleet_tamper_report_json(&self, _report_json: &str) -> String {
        // Return a placeholder message when no tamper engine is wired.
        "Fleet tamper correlation not available (no engine wired).".into()
    }
}

static FLEET_TAMPER_RUNTIME: OnceLock<Arc<dyn FleetTamperRuntime>> = OnceLock::new();

/// Inject a real fleet tamper runtime from a higher-layer crate (e.g. spanda-tamper bridge).
pub fn set_fleet_tamper_runtime(runtime: Arc<dyn FleetTamperRuntime>) {
    // Accept the first injection; subsequent calls are silently ignored via OnceLock semantics.
    let _ = FLEET_TAMPER_RUNTIME.set(runtime);
}

/// Return the active fleet tamper runtime, falling back to the no-op implementation.
pub fn fleet_tamper_runtime() -> Arc<dyn FleetTamperRuntime> {
    // Return the injected runtime if set, otherwise use the noop default.
    FLEET_TAMPER_RUNTIME
        .get()
        .cloned()
        .unwrap_or_else(|| Arc::new(NoopFleetTamperRuntime))
}
