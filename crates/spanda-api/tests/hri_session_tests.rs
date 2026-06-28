//! Experimental HRI session REST handlers.
use spanda_api::hri::{AnnotateHriSessionRequest, StartHriSessionRequest};
use spanda_api::state::ControlCenterState;

#[test]
fn hri_session_store_start_and_annotate() {
    let mut state = ControlCenterState::new();
    let record = state
        .hri_session_store
        .start_session(StartHriSessionRequest {
            id: Some("repair-session-001".into()),
            session_type: Some("remote_expert".into()),
            field_human_id: Some("tech-001".into()),
            expert_human_id: Some("expert-002".into()),
            robot_id: Some("AMR".into()),
            ar_device_id: Some("hololens-tech-001".into()),
            camera_device_id: None,
            capabilities: Some(vec!["annotation".into()]),
        });
    assert_eq!(record.status, "active");
    let updated = state
        .hri_session_store
        .annotate(
            "repair-session-001",
            AnnotateHriSessionRequest {
                layer: "annotation".into(),
                text: "Replace fuse F3".into(),
                author_human_id: Some("expert-002".into()),
            },
            1_000.0,
        )
        .expect("annotate");
    assert_eq!(updated.annotations.len(), 1);
    let sessions = state.hri_session_store.list_sessions(&state);
    assert!(sessions.iter().any(|s| s.id == "repair-session-001"));
}
