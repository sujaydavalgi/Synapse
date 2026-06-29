//! Injectable readiness evaluation boundary for fleet agents and OTA agents.
//!
use std::sync::{Arc, OnceLock};

/// Extension points for agent readiness evaluation outside the interpreter.
pub trait ReadinessRuntime: Send + Sync {
    /// Evaluate agent readiness from program source and return serialised JSON report.
    ///
    /// Parameters:
    /// - `source` — deployed program source text
    /// - `target` — optional target hardware profile name
    /// - `include_runtime` — whether to include runtime health evaluation
    /// - `inject_health_faults` — whether to inject simulated health faults
    ///
    /// Returns:
    /// Ok(json_body) or Err(error_message).
    ///
    /// Options:
    /// None.
    ///
    /// Example:
    /// let json = readiness_runtime().evaluate_agent_readiness_json(src, None, false, false).unwrap();
    fn evaluate_agent_readiness_json(
        &self,
        source: &str,
        target: Option<&str>,
        include_runtime: bool,
        inject_health_faults: bool,
    ) -> Result<String, String>;
}

/// No-op readiness runtime for tests and runs without a real readiness engine.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopReadinessRuntime;

impl ReadinessRuntime for NoopReadinessRuntime {
    fn evaluate_agent_readiness_json(
        &self,
        _source: &str,
        _target: Option<&str>,
        _include_runtime: bool,
        _inject_health_faults: bool,
    ) -> Result<String, String> {
        // Return a minimal passing report structure when no real engine is wired.
        Ok(r#"{"ok":true,"passed":true,"readiness":{"status":"Ready","score":1.0,"issues":[]},"checks":[]}"#.into())
    }
}

static READINESS_RUNTIME: OnceLock<Arc<dyn ReadinessRuntime>> = OnceLock::new();

/// Inject a real readiness runtime from a higher-layer crate (e.g. CLI, spanda-readiness bridge).
pub fn set_readiness_runtime(runtime: Arc<dyn ReadinessRuntime>) {
    // Accept the first injection; subsequent calls are silently ignored via OnceLock semantics.
    let _ = READINESS_RUNTIME.set(runtime);
}

/// Return the active readiness runtime, falling back to the no-op implementation.
pub fn readiness_runtime() -> Arc<dyn ReadinessRuntime> {
    // Return the injected runtime if set, otherwise use the noop default.
    READINESS_RUNTIME
        .get()
        .cloned()
        .unwrap_or_else(|| Arc::new(NoopReadinessRuntime))
}
