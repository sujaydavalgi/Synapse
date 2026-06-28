//! Experimental HRI session API for remote expert and collaborative workflows.
//!
use crate::handlers::{bad_request, json_ok, unauthorized};
use crate::state::ControlCenterState;
use serde::{Deserialize, Serialize};
use spanda_deploy_http::HttpResponse;
use spanda_security::{ApiKeyStore, RbacAction, RbacContext};
use std::collections::HashMap;

/// Annotation published during a remote expert session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HriAnnotation {
    pub layer: String,
    pub text: String,
    pub author_human_id: Option<String>,
    pub created_at_ms: f64,
}

/// Active or configured HRI collaboration session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HriSessionRecord {
    pub id: String,
    pub session_type: String,
    pub field_human_id: Option<String>,
    pub expert_human_id: Option<String>,
    pub robot_id: Option<String>,
    pub ar_device_id: Option<String>,
    pub camera_device_id: Option<String>,
    pub status: String,
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub annotations: Vec<HriAnnotation>,
    #[serde(default)]
    pub replay_url: Option<String>,
}

/// In-memory store for active remote expert sessions and annotations.
#[derive(Debug, Default)]
pub struct HriSessionStore {
    active: HashMap<String, HriSessionRecord>,
}

impl HriSessionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn list_sessions(&self, state: &ControlCenterState) -> Vec<HriSessionRecord> {
        let mut sessions: Vec<HriSessionRecord> = state
            .resolved
            .as_ref()
            .map(|cfg| {
                cfg.human_registry
                    .spatial_sessions
                    .iter()
                    .map(|session| HriSessionRecord {
                        id: session.id.clone(),
                        session_type: session
                            .session_type
                            .clone()
                            .unwrap_or_else(|| "remote_expert".into()),
                        field_human_id: session.field_human_id.clone(),
                        expert_human_id: session.expert_human_id.clone(),
                        robot_id: session.robot_id.clone(),
                        ar_device_id: session.ar_device_id.clone(),
                        camera_device_id: session.camera_device_id.clone(),
                        status: "configured".into(),
                        capabilities: session.capabilities.clone(),
                        annotations: Vec::new(),
                        replay_url: None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        for active in self.active.values() {
            if let Some(existing) = sessions.iter_mut().find(|s| s.id == active.id) {
                *existing = active.clone();
            } else {
                sessions.push(active.clone());
            }
        }
        sessions
    }

    pub fn start_session(&mut self, req: StartHriSessionRequest) -> HriSessionRecord {
        let id = req
            .id
            .unwrap_or_else(|| format!("hri-session-{}", self.active.len() + 1));
        let record = HriSessionRecord {
            id: id.clone(),
            session_type: req
                .session_type
                .unwrap_or_else(|| "remote_expert".into()),
            field_human_id: req.field_human_id,
            expert_human_id: req.expert_human_id,
            robot_id: req.robot_id,
            ar_device_id: req.ar_device_id,
            camera_device_id: req.camera_device_id,
            status: "active".into(),
            capabilities: req.capabilities.unwrap_or_default(),
            annotations: Vec::new(),
            replay_url: Some(format!("/v1/hri/sessions/{id}/replay")),
        };
        self.active.insert(id, record.clone());
        record
    }

    pub fn annotate(
        &mut self,
        session_id: &str,
        req: AnnotateHriSessionRequest,
        now_ms: f64,
    ) -> Option<HriSessionRecord> {
        let record = self.active.get_mut(session_id)?;
        record.annotations.push(HriAnnotation {
            layer: req.layer,
            text: req.text,
            author_human_id: req.author_human_id,
            created_at_ms: now_ms,
        });
        Some(record.clone())
    }

    pub fn replay_for(&self, session_id: &str) -> Option<HriSessionRecord> {
        self.active.get(session_id).cloned()
    }
}

#[derive(Debug, Deserialize)]
pub struct StartHriSessionRequest {
    pub id: Option<String>,
    #[serde(default)]
    pub session_type: Option<String>,
    #[serde(default)]
    pub field_human_id: Option<String>,
    #[serde(default)]
    pub expert_human_id: Option<String>,
    #[serde(default)]
    pub robot_id: Option<String>,
    #[serde(default)]
    pub ar_device_id: Option<String>,
    #[serde(default)]
    pub camera_device_id: Option<String>,
    #[serde(default)]
    pub capabilities: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct AnnotateHriSessionRequest {
    pub layer: String,
    pub text: String,
    #[serde(default)]
    pub author_human_id: Option<String>,
}

pub fn hri_sessions_list(state: &ControlCenterState) -> HttpResponse {
    let sessions = state.hri_session_store.list_sessions(state);
    json_ok(&serde_json::json!({
        "version": "v1",
        "sessions": sessions,
        "count": sessions.len(),
    }))
}

pub fn hri_sessions_create(
    state: &mut ControlCenterState,
    body: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Approve) {
        return unauthorized();
    }
    let req: StartHriSessionRequest = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(error) => return bad_request(&error.to_string()),
    };
    let record = state.hri_session_store.start_session(req);
    json_ok(&serde_json::json!({
        "version": "v1",
        "ok": true,
        "session": record,
    }))
}

pub fn hri_session_annotate(
    state: &mut ControlCenterState,
    session_id: &str,
    body: &str,
    ctx: Option<&RbacContext>,
    now_ms: f64,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Approve) {
        return unauthorized();
    }
    let req: AnnotateHriSessionRequest = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(error) => return bad_request(&error.to_string()),
    };
    match state
        .hri_session_store
        .annotate(session_id, req, now_ms)
    {
        Some(session) => json_ok(&serde_json::json!({
            "version": "v1",
            "ok": true,
            "session": session,
        })),
        None => bad_request("session not found or not active"),
    }
}

pub fn hri_session_replay(state: &ControlCenterState, session_id: &str) -> HttpResponse {
    match state.hri_session_store.replay_for(session_id) {
        Some(session) => json_ok(&serde_json::json!({
            "version": "v1",
            "session_id": session_id,
            "replay_url": session.replay_url,
            "annotation_count": session.annotations.len(),
            "status": session.status,
        })),
        None => json_ok(&serde_json::json!({
            "version": "v1",
            "session_id": session_id,
            "replay_url": format!("/v1/hri/sessions/{session_id}/replay"),
            "status": "configured",
            "annotation_count": 0,
        })),
    }
}

pub fn hri_collaboration_graph(state: &ControlCenterState) -> HttpResponse {
    let Some(resolved) = state.resolved.as_ref() else {
        return bad_request("no resolved configuration loaded");
    };
    let registry = &resolved.human_registry;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for human in &registry.humans {
        nodes.push(serde_json::json!({
            "id": human.id,
            "kind": "human",
            "role": human.role,
            "availability": human.availability,
        }));
        if let Some(assignments) = human.assignments.as_ref().and_then(|v| v.as_table()) {
            if let Some(robot_id) = assignments.get("robot_id").and_then(|v| v.as_str()) {
                edges.push(serde_json::json!({
                    "from": human.id,
                    "to": robot_id,
                    "relation": "assignment",
                }));
            }
            if let Some(mission_id) = assignments.get("mission_id").and_then(|v| v.as_str()) {
                edges.push(serde_json::json!({
                    "from": human.id,
                    "to": mission_id,
                    "relation": "mission",
                }));
            }
        }
    }

    if let Some(ref tree) = resolved.device_tree.fleet {
        for robot in &tree.robots {
            nodes.push(serde_json::json!({
                "id": robot.id,
                "kind": "robot",
                "model": robot.model,
            }));
        }
        for drone in &tree.drones {
            nodes.push(serde_json::json!({
                "id": drone.id,
                "kind": "drone",
                "type": drone.device_type,
            }));
        }
    }

    for session in &registry.spatial_sessions {
        nodes.push(serde_json::json!({
            "id": session.id,
            "kind": "session",
            "session_type": session.session_type,
        }));
        if let Some(field_id) = session.field_human_id.as_deref() {
            edges.push(serde_json::json!({
                "from": field_id,
                "to": session.id,
                "relation": "field_operator",
            }));
        }
        if let Some(expert_id) = session.expert_human_id.as_deref() {
            edges.push(serde_json::json!({
                "from": expert_id,
                "to": session.id,
                "relation": "remote_expert",
            }));
        }
        if let Some(robot_id) = session.robot_id.as_deref() {
            edges.push(serde_json::json!({
                "from": session.id,
                "to": robot_id,
                "relation": "session_robot",
            }));
        }
    }

    let active = state.hri_session_store.list_sessions(state);
    for session in &active {
        if session.status == "active" {
            edges.push(serde_json::json!({
                "from": session.id,
                "to": session.status,
                "relation": "live_status",
            }));
        }
    }

    json_ok(&serde_json::json!({
        "version": "v1",
        "node_count": nodes.len(),
        "edge_count": edges.len(),
        "nodes": nodes,
        "edges": edges,
        "active_session_count": active.iter().filter(|s| s.status == "active").count(),
    }))
}

pub fn hri_context_snapshot(state: &ControlCenterState) -> HttpResponse {
    let Some(resolved) = state.resolved.as_ref() else {
        return bad_request("no resolved configuration loaded");
    };
    let registry = &resolved.human_registry;
    let hazard_zones: Vec<_> = registry
        .hazard_zones
        .iter()
        .map(|zone| {
            serde_json::json!({
                "id": zone.id,
                "type": zone.zone_type,
                "severity": zone.severity,
                "center": zone.center,
                "radius_m": zone.radius_m,
                "linked_robots": zone.linked_robots,
                "alert_on_entry": zone.alert_on_entry,
                "description": zone.description,
            })
        })
        .collect();
    let human_locations: Vec<_> = registry
        .humans
        .iter()
        .filter(|human| human.location.is_some())
        .map(|human| {
            serde_json::json!({
                "human_id": human.id,
                "role": human.role,
                "location": human.location,
            })
        })
        .collect();
    json_ok(&serde_json::json!({
        "version": "v1",
        "hazard_zone_count": hazard_zones.len(),
        "hazard_zones": hazard_zones,
        "human_locations": human_locations,
        "spatial_session_count": registry.spatial_sessions.len(),
        "active_hri_sessions": state
            .hri_session_store
            .list_sessions(state)
            .iter()
            .filter(|s| s.status == "active")
            .count(),
    }))
}
