//! Entity mutation REST handlers — register, tag, relate, and TOML sync.
//!
use crate::handlers::{bad_request, json_ok, unauthorized};
use crate::state::ControlCenterState;
use spanda_audit::platform_event::{names, PlatformEvent};
use spanda_config::{
    default_entity_overlay_path, register_entity_overlay, relate_entities_overlay,
    save_entity_overlay, sync_entity_overlay_to_toml, tag_entity_overlay, EntityRegisterRequest,
    EntityRelateRequest, EntityTagRequest,
};
use spanda_deploy_http::HttpResponse;
use spanda_security::{ApiKeyStore, RbacAction, RbacContext};

const API_VERSION: &str = "v1";

fn record_entity_audit(state: &mut ControlCenterState, action: &str, payload: &serde_json::Value) {
    let event_type = match action {
        "register" => names::ENTITY_CREATED,
        "tag" => names::ENTITY_TAGGED,
        "relate" => names::ENTITY_RELATED,
        "sync" => names::ENTITY_UPDATED,
        _ => names::ENTITY_UPDATED,
    };
    let entity_id = payload
        .get("entity_id")
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let event = {
        let mut event = PlatformEvent::new(event_type, "spanda-api", payload.clone());
        if let Some(id) = entity_id {
            event = event.with_entity_id(id);
        }
        event
    };
    let _ = state.mutation_audit.record_platform_event(&event);
    let _ = spanda_telemetry_store::record_platform_event(&event);
}

/// POST /v1/entities/register — register or update an entity in the overlay.
pub fn entity_register(
    state: &mut ControlCenterState,
    body: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Provision) {
        return unauthorized();
    }
    let request: EntityRegisterRequest = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(error) => return bad_request(&error.to_string()),
    };
    if request.id.trim().is_empty() {
        return bad_request("entity id is required");
    }
    let record = register_entity_overlay(&mut state.entity_overlay, &request);
    if let Err(error) = save_entity_overlay(&default_entity_overlay_path(), &state.entity_overlay) {
        return bad_request(&error);
    }
    record_entity_audit(
        state,
        "register",
        &serde_json::json!({ "entity_id": record.id, "entity_type": record.entity_type }),
    );
    let mut sync = None;
    if request.persist {
        sync = Some(run_entity_sync(state));
    }
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "entity": record,
        "overlay_version": state.entity_overlay.version,
        "sync": sync,
    }))
}

/// POST /v1/entities/{id}/tags — add or remove tags on an entity overlay.
pub fn entity_tag(
    state: &mut ControlCenterState,
    entity_id: &str,
    body: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Provision) {
        return unauthorized();
    }
    let request: EntityTagRequest = serde_json::from_str(body).unwrap_or_default();
    if state.entity_overlay.entities.contains_key(entity_id) {
        let Some(record) = tag_entity_overlay(&mut state.entity_overlay, entity_id, &request)
        else {
            return bad_request(&format!("entity '{entity_id}' not found in overlay"));
        };
        if let Err(error) =
            save_entity_overlay(&default_entity_overlay_path(), &state.entity_overlay)
        {
            return bad_request(&error);
        }
        record_entity_audit(state, "tag", &serde_json::json!({ "entity_id": entity_id }));
        return json_ok(&serde_json::json!({
            "version": API_VERSION,
            "entity": record,
            "overlay_version": state.entity_overlay.version,
        }));
    }
    let base = state.entity_registry();
    let Some(base_record) = base.get(entity_id) else {
        return bad_request(&format!("entity '{entity_id}' not found"));
    };
    let mut record = base_record.clone();
    for tag in &request.add {
        if !record.tags.contains(tag) {
            record.tags.push(tag.clone());
        }
    }
    if !request.remove.is_empty() {
        let remove: std::collections::HashSet<_> = request.remove.iter().cloned().collect();
        record.tags.retain(|tag| !remove.contains(tag));
    }
    state
        .entity_overlay
        .entities
        .insert(entity_id.to_string(), record.clone());
    state.entity_overlay.version = state.entity_overlay.version.saturating_add(1);
    if let Err(error) = save_entity_overlay(&default_entity_overlay_path(), &state.entity_overlay) {
        return bad_request(&error);
    }
    record_entity_audit(state, "tag", &serde_json::json!({ "entity_id": entity_id }));
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "entity": record,
        "overlay_version": state.entity_overlay.version,
    }))
}

/// POST /v1/entities/relationships — relate two entities in the overlay.
pub fn entity_relate(
    state: &mut ControlCenterState,
    body: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Provision) {
        return unauthorized();
    }
    let request: EntityRelateRequest = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(error) => return bad_request(&error.to_string()),
    };
    let registry = state.entity_registry();
    if !registry.entities.contains_key(&request.from_id) {
        return bad_request(&format!("from entity '{}' not found", request.from_id));
    }
    if !registry.entities.contains_key(&request.to_id) {
        return bad_request(&format!("to entity '{}' not found", request.to_id));
    }
    let edge = relate_entities_overlay(&mut state.entity_overlay, &request);
    if let Err(error) = save_entity_overlay(&default_entity_overlay_path(), &state.entity_overlay) {
        return bad_request(&error);
    }
    record_entity_audit(
        state,
        "relate",
        &serde_json::json!({
            "from_id": edge.from_id,
            "to_id": edge.to_id,
            "kind": edge.kind.as_str(),
        }),
    );
    json_ok(&serde_json::json!({
        "version": API_VERSION,
        "relationship": edge,
        "overlay_version": state.entity_overlay.version,
    }))
}

/// POST /v1/entities/sync — flush overlay entities to TOML fragments.
pub fn entity_sync(state: &mut ControlCenterState, ctx: Option<&RbacContext>) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Provision) {
        return unauthorized();
    }
    match run_entity_sync(state) {
        Ok(result) => {
            record_entity_audit(state, "sync", &serde_json::json!({ "path": result.path }));
            json_ok(&serde_json::json!({
                "version": API_VERSION,
                "sync": result,
            }))
        }
        Err(message) => bad_request(&message),
    }
}

fn run_entity_sync(
    state: &mut ControlCenterState,
) -> Result<spanda_config::EntitySyncResult, String> {
    let root = state
        .project_root()
        .ok_or_else(|| "config path not set".to_string())?;
    let manifest = state
        .resolved
        .as_ref()
        .map(|resolved| resolved.manifest.clone())
        .ok_or_else(|| "no resolved configuration".to_string())?;
    let result = sync_entity_overlay_to_toml(&root, &manifest, &state.entity_overlay)
        .map_err(|e| e.to_string())?;
    state.reload_config()?;
    Ok(result)
}

/// JSON body for gRPC `RegisterEntity`.
pub fn entity_register_json(
    state: &mut ControlCenterState,
    body: &str,
    ctx: Option<&RbacContext>,
) -> String {
    entity_register(state, body, ctx).body
}

/// JSON body for gRPC `TagEntity`.
pub fn entity_tag_json(
    state: &mut ControlCenterState,
    entity_id: &str,
    body: &str,
    ctx: Option<&RbacContext>,
) -> String {
    entity_tag(state, entity_id, body, ctx).body
}

/// JSON body for gRPC `RelateEntities`.
pub fn entity_relate_json(
    state: &mut ControlCenterState,
    body: &str,
    ctx: Option<&RbacContext>,
) -> String {
    entity_relate(state, body, ctx).body
}

/// JSON body for gRPC `SyncEntities`.
pub fn entity_sync_json(state: &mut ControlCenterState, ctx: Option<&RbacContext>) -> String {
    entity_sync(state, ctx).body
}
