//! Mutation audit trail for Control Center API operations.
//!
use crate::state::ControlCenterState;
use spanda_security::RbacContext;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// Append-only audit log path under `.spanda/`.
pub fn default_mutation_audit_path() -> PathBuf {
    std::env::var("SPANDA_MUTATION_AUDIT_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".spanda/control-center-mutations.jsonl"))
}

/// Record a successful mutating REST request.
pub fn maybe_record_mutation(
    state: &mut ControlCenterState,
    method: &str,
    path: &str,
    status: u16,
    ctx: Option<&RbacContext>,
    correlation_id: &str,
) {
    if !matches!(method, "POST" | "PATCH" | "PUT" | "DELETE") {
        return;
    }
    if !(200..300).contains(&status) {
        return;
    }
    let payload = serde_json::json!({
        "method": method,
        "path": path,
        "status": status,
        "correlation_id": correlation_id,
        "actor_key_id": ctx.map(|c| c.key_id.as_str()).unwrap_or("anonymous"),
        "actor_role": ctx.map(|c| format!("{:?}", c.role)).unwrap_or_else(|| "guest".into()),
    })
    .to_string();
    if let Ok(record_id) = state
        .mutation_audit
        .record_event("control_center.api.mutation", &payload)
    {
        let _ = append_audit_event(
            &default_mutation_audit_path(),
            &record_id.0,
            "control_center.api.mutation",
            &payload,
        );
    }
}

/// Record a successful mutating gRPC RPC.
pub fn record_grpc_mutation(
    state: &mut ControlCenterState,
    rpc_name: &str,
    response_json: &str,
    ctx: Option<&RbacContext>,
) {
    if response_json.contains("\"ok\":false") || response_json.contains("unauthorized") {
        return;
    }
    let payload = serde_json::json!({
        "transport": "grpc",
        "rpc": rpc_name,
        "actor_key_id": ctx.map(|c| c.key_id.as_str()).unwrap_or("anonymous"),
        "actor_role": ctx.map(|c| format!("{:?}", c.role)).unwrap_or_else(|| "guest".into()),
    })
    .to_string();
    if let Ok(record_id) = state
        .mutation_audit
        .record_event("control_center.grpc.mutation", &payload)
    {
        let _ = append_audit_event(
            &default_mutation_audit_path(),
            &record_id.0,
            "control_center.grpc.mutation",
            &payload,
        );
    }
}

fn append_audit_event(
    path: &std::path::Path,
    record_id: &str,
    event_type: &str,
    payload: &str,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let timestamp_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64() * 1000.0)
        .unwrap_or(0.0);
    let line = serde_json::json!({
        "id": record_id,
        "event_type": event_type,
        "payload": payload,
        "timestamp_ms": timestamp_ms,
    });
    let encoded = serde_json::to_string(&line).map_err(|e| e.to_string())?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    writeln!(file, "{encoded}").map_err(|e| e.to_string())
}
