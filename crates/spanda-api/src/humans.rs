//! Human operator, wearable, and readiness REST handlers for Control Center.
//!
use crate::handlers::{bad_request, json_ok};
use crate::state::ControlCenterState;
use spanda_deploy_http::HttpResponse;
use spanda_readiness::evaluate_human_collaboration;

pub fn humans_list(state: &ControlCenterState) -> HttpResponse {
    let Some(resolved) = state.resolved.as_ref() else {
        return bad_request("no resolved configuration loaded");
    };
    let humans: Vec<_> = resolved
        .human_registry
        .humans
        .iter()
        .map(|human| {
            serde_json::json!({
                "id": human.id,
                "role": human.role,
                "display_name": human.display_name,
                "capabilities": human.capabilities,
                "availability": human.availability,
                "trust_level": human.trust_level,
                "certification_count": human.certifications.len(),
                "wearable_count": resolved.human_registry.wearables_for_human(&human.id).len(),
            })
        })
        .collect();
    json_ok(&serde_json::json!({
        "version": "v1",
        "humans": humans,
        "count": humans.len(),
    }))
}

pub fn wearables_list(state: &ControlCenterState) -> HttpResponse {
    let Some(resolved) = state.resolved.as_ref() else {
        return bad_request("no resolved configuration loaded");
    };
    let health_gate = resolved.human_health_gate();
    let wearables: Vec<_> = resolved
        .human_registry
        .wearables
        .iter()
        .map(|wearable| {
            serde_json::json!({
                "id": wearable.id,
                "type": wearable.device_type,
                "provider": wearable.provider,
                "human_id": wearable.human_id,
                "capabilities": wearable.capabilities,
                "trust_level": wearable.trust_level,
                "health_telemetry_allowed": health_gate.allows_health_telemetry_read(),
            })
        })
        .collect();
    json_ok(&serde_json::json!({
        "version": "v1",
        "wearables": wearables,
        "count": wearables.len(),
        "human_health": health_gate,
    }))
}

pub fn human_health_policy(state: &ControlCenterState) -> HttpResponse {
    let Some(resolved) = state.resolved.as_ref() else {
        return bad_request("no resolved configuration loaded");
    };
    let gate = resolved.human_health_gate();
    json_ok(&serde_json::json!({
        "version": "v1",
        "policy": gate,
        "hint": "Set [security.human_health] enabled=true and export SPANDA_HUMAN_HEALTH_ENABLED=1",
    }))
}

pub fn human_readiness_get(state: &ControlCenterState, human_id: &str) -> HttpResponse {
    let Some(resolved) = state.resolved.as_ref() else {
        return bad_request("no resolved configuration loaded");
    };
    let Some(human) = resolved.human_registry.human(human_id) else {
        return bad_request("human not found");
    };
    let today = chrono::Utc::now().date_naive().to_string();
    let certs_valid = human.certifications.iter().all(|cert| {
        cert.expires
            .as_ref()
            .map(|date| date.as_str() >= today.as_str())
            .unwrap_or(true)
    });
    let wearables: Vec<_> = resolved
        .human_registry
        .wearables_for_human(human_id)
        .into_iter()
        .map(|wearable| {
            serde_json::json!({
                "id": wearable.id,
                "type": wearable.device_type,
                "provider": wearable.provider,
                "capabilities": wearable.capabilities,
            })
        })
        .collect();
    let mut payload = serde_json::json!({
        "version": "v1",
        "human_id": human.id,
        "role": human.role,
        "display_name": human.display_name,
        "available": human.is_available(),
        "capabilities": human.capabilities,
        "certifications_valid": certs_valid,
        "wearables": wearables,
        "trust_level": human.trust_level,
    });
    if let Some(path) = state.program_path.as_ref() {
        if let Ok((program, _, _)) = crate::program::parse_program_file(path) {
            let report = evaluate_human_collaboration(resolved, &program);
            payload["team_readiness"] = serde_json::json!({
                "profile": report.profile,
                "operator_ready": report.operator_ready,
                "team_ready": report.team_ready,
                "mission_ready": report.mission_ready,
                "total_score": report.total_score,
                "minimum_score": report.minimum_score,
            });
        }
    }
    json_ok(&payload)
}

pub fn humans_readiness_team(state: &ControlCenterState) -> HttpResponse {
    let Some(resolved) = state.resolved.as_ref() else {
        return bad_request("no resolved configuration loaded");
    };
    let today = chrono::Utc::now().date_naive().to_string();
    let operators: Vec<_> = resolved
        .human_registry
        .humans
        .iter()
        .map(|human| {
            let certs_valid = human.certifications.iter().all(|cert| {
                cert.expires
                    .as_ref()
                    .map(|date| date.as_str() >= today.as_str())
                    .unwrap_or(true)
            });
            serde_json::json!({
                "id": human.id,
                "role": human.role,
                "display_name": human.display_name,
                "available": human.is_available(),
                "certifications_valid": certs_valid,
                "trust_level": human.trust_level,
                "wearable_count": resolved.human_registry.wearables_for_human(&human.id).len(),
            })
        })
        .collect();
    let mut payload = serde_json::json!({
        "version": "v1",
        "operator_count": operators.len(),
        "operators": operators,
    });
    if let Some(path) = state.program_path.as_ref() {
        if let Ok((program, _, _)) = crate::program::parse_program_file(path) {
            let report = evaluate_human_collaboration(resolved, &program);
            payload["team_readiness"] = serde_json::to_value(&report).unwrap_or(serde_json::json!({}));
        }
    }
    json_ok(&payload)
}
