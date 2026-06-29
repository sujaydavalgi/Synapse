//! Injectable fleet telemetry merge runtime for mesh coordinators.
//!
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

/// Extension points for fleet-level OTLP telemetry merging.
pub trait FleetTelemetryRuntime: Send + Sync {
    /// Merge OTLP JSON telemetry shards from all fleet members into a single JSON payload.
    ///
    /// Parameters:
    /// - `shards` — map of robot_id → OTLP JSON string
    ///
    /// Returns:
    /// Ok(merged_otlp_json) or Err(error_message).
    ///
    /// Options:
    /// None.
    ///
    /// Example:
    /// let json = fleet_telemetry_runtime().merge_fleet_otlp_json(&shards).unwrap();
    fn merge_fleet_otlp_json(&self, shards: &HashMap<String, String>) -> Result<String, String>;
}

/// No-op fleet telemetry runtime for tests and runs without a real telemetry store.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopFleetTelemetryRuntime;

impl FleetTelemetryRuntime for NoopFleetTelemetryRuntime {
    fn merge_fleet_otlp_json(&self, _shards: &HashMap<String, String>) -> Result<String, String> {
        // Return an empty merged payload when no telemetry engine is wired.
        Ok(r#"{"resourceSpans":[]}"#.into())
    }
}

static FLEET_TELEMETRY_RUNTIME: OnceLock<Arc<dyn FleetTelemetryRuntime>> = OnceLock::new();

/// Inject a real fleet telemetry runtime from a higher-layer crate (e.g. spanda-telemetry-store bridge).
pub fn set_fleet_telemetry_runtime(runtime: Arc<dyn FleetTelemetryRuntime>) {
    // Accept the first injection; subsequent calls are silently ignored via OnceLock semantics.
    let _ = FLEET_TELEMETRY_RUNTIME.set(runtime);
}

/// Return the active fleet telemetry runtime, falling back to the no-op implementation.
pub fn fleet_telemetry_runtime() -> Arc<dyn FleetTelemetryRuntime> {
    // Return the injected runtime if set, otherwise use the noop default.
    FLEET_TELEMETRY_RUNTIME
        .get()
        .cloned()
        .unwrap_or_else(|| Arc::new(NoopFleetTelemetryRuntime))
}
