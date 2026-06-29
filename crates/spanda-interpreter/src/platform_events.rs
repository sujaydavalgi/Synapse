//! Platform event emission for interpreter lifecycle hooks.
//!
use spanda_ast::nodes::{Program, RobotDecl};
use spanda_audit::platform_event::names;
use spanda_audit::{AuditRuntime, PlatformEvent};
use serde_json::json;

/// Record a mission lifecycle platform event when audit runtime is configured.
pub(crate) fn emit_mission_platform_event(
    audit: Option<&mut AuditRuntime>,
    event_type: &str,
    program: &Program,
    trace_source: Option<&str>,
    success: bool,
) {
    let Some(rt) = audit else {
        return;
    };

    let mission_key = trace_source
        .map(str::to_string)
        .or_else(|| first_robot_name(program))
        .unwrap_or_else(|| "program".into());
    let event = PlatformEvent::new(
        event_type,
        "spanda-interpreter",
        json!({
            "mission": mission_key,
            "success": success,
            "robot_count": robot_count(program),
        }),
    )
    .with_entity_id(format!("mission/{mission_key}"));
    let _ = rt.record_platform_event(&event);
}

pub(crate) fn emit_mission_started(
    audit: Option<&mut AuditRuntime>,
    program: &Program,
    trace_source: Option<&str>,
) {
    emit_mission_platform_event(audit, names::MISSION_STARTED, program, trace_source, true);
}

pub(crate) fn emit_mission_completed(
    audit: Option<&mut AuditRuntime>,
    program: &Program,
    trace_source: Option<&str>,
    success: bool,
) {
    emit_mission_platform_event(audit, names::MISSION_COMPLETED, program, trace_source, success);
}

fn robot_count(program: &Program) -> usize {
    let Program::Program { robots, .. } = program;
    robots.len()
}

fn first_robot_name(program: &Program) -> Option<String> {
    let Program::Program { robots, .. } = program;
    robots.first().map(|robot| match robot {
        RobotDecl::RobotDecl { name, .. } => name.clone(),
    })
}
