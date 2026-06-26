//! Phase E4 handlers — compliance export, digital thread, executive analytics, reporting.
//!
use crate::program::parse_program_file;
use crate::state::ControlCenterState;
use serde::Deserialize;
use spanda_compliance::{format_accreditation_report, generate_accreditation_report};
use spanda_deploy_http::HttpResponse;
use spanda_graph::{query_digital_thread, DigitalThreadQuery};
use spanda_readiness::{
    analyze_readiness_trends, default_readiness_history_path, load_readiness_history,
};
use spanda_score::{evaluate_scorecard, ScorecardOptions};
use spanda_security::{ApiKeyStore, RbacAction, RbacContext};
use std::sync::Arc;

use crate::handlers::{bad_request, json_ok, parse_query, unauthorized};

#[derive(Deserialize)]
struct ComplianceExportRequest {
    profile: String,
}

pub fn compliance_export(
    state: &ControlCenterState,
    query: &str,
    body: Option<&str>,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Deploy) {
        return unauthorized();
    }
    let profile = body
        .and_then(|raw| serde_json::from_str::<ComplianceExportRequest>(raw).ok())
        .map(|req| req.profile)
        .or_else(|| parse_query(query).get("profile").cloned())
        .unwrap_or_else(|| "defense".to_string());
    let (program, _source, label) = match load_program(state) {
        Ok(value) => value,
        Err(message) => return bad_request(&message),
    };
    match generate_accreditation_report(&program, &profile, &label) {
        Ok(report) => json_ok(&serde_json::json!({
            "version": "v1",
            "export": report,
            "markdown": format_accreditation_report(&report, false),
        })),
        Err(message) => bad_request(&message),
    }
}

pub fn digital_thread_query(state: &ControlCenterState, query: &str) -> HttpResponse {
    let params = parse_query(query);
    let thread_query = DigitalThreadQuery {
        capability: params.get("capability").cloned(),
        device_id: params.get("device_id").cloned(),
        node_id: params.get("node_id").cloned(),
    };
    let (program, _source, label) = match load_program(state) {
        Ok(value) => value,
        Err(message) => return bad_request(&message),
    };
    let report = query_digital_thread(
        &program,
        &label,
        state.resolved.as_ref(),
        &thread_query,
    );
    json_ok(&serde_json::json!({
        "version": "v1",
        "digital_thread": report,
    }))
}

pub fn executive_scorecard(state: &ControlCenterState) -> HttpResponse {
    let (program, source, label) = match load_program(state) {
        Ok(value) => value,
        Err(message) => return bad_request(&message),
    };
    let options = ScorecardOptions {
        system_config: state.resolved.clone().map(Arc::new),
    };
    let report = evaluate_scorecard(&program, &source, &label, &options);
    json_ok(&serde_json::json!({
        "version": "v1",
        "scorecard": report,
    }))
}

pub fn analytics_readiness(state: &ControlCenterState, query: &str) -> HttpResponse {
    let params = parse_query(query);
    let forecast_days = params
        .get("forecast_days")
        .and_then(|value| value.parse::<u32>().ok());
    let (_program, _source, label) = match load_program(state) {
        Ok(value) => value,
        Err(message) => return bad_request(&message),
    };
    let history = load_readiness_history(&default_readiness_history_path());
    let report = analyze_readiness_trends(&history, &label, forecast_days, 80);
    json_ok(&serde_json::json!({
        "version": "v1",
        "program": label,
        "analytics": report,
    }))
}

pub fn reports_export(
    state: &ControlCenterState,
    query: &str,
    ctx: Option<&RbacContext>,
) -> HttpResponse {
    if !ApiKeyStore::check(ctx, RbacAction::Deploy) {
        return unauthorized();
    }
    let params = parse_query(query);
    let profile = params
        .get("profile")
        .cloned()
        .unwrap_or_else(|| "defense".to_string());
    let format = params
        .get("format")
        .map(String::as_str)
        .unwrap_or("markdown");
    let (program, source, label) = match load_program(state) {
        Ok(value) => value,
        Err(message) => return bad_request(&message),
    };
    let accreditation = match generate_accreditation_report(&program, &profile, &label) {
        Ok(report) => report,
        Err(message) => return bad_request(&message),
    };
    let options = ScorecardOptions {
        system_config: state.resolved.clone().map(Arc::new),
    };
    let scorecard = evaluate_scorecard(&program, &source, &label, &options);
    let digital_thread = query_digital_thread(
        &program,
        &label,
        state.resolved.as_ref(),
        &DigitalThreadQuery::default(),
    );
    let markdown = format!(
        "# Spanda executive report — {}\n\n## Compliance ({})\n\n{}\n\n## Scorecard\n\nOverall: {}/100\n\n## Digital thread\n\nNodes: {}, edges: {}\n",
        label,
        profile,
        format_accreditation_report(&accreditation, false),
        scorecard.overall_score,
        digital_thread.matched_node_count,
        digital_thread.matched_edge_count,
    );
    if format == "json" {
        return json_ok(&serde_json::json!({
            "version": "v1",
            "program": label,
            "profile": profile,
            "accreditation": accreditation,
            "scorecard": scorecard,
            "digital_thread_summary": digital_thread.chain_summary,
        }));
    }
    json_ok(&serde_json::json!({
        "version": "v1",
        "program": label,
        "profile": profile,
        "format": "markdown",
        "body": markdown,
    }))
}

fn load_program(
    state: &ControlCenterState,
) -> Result<(spanda_ast::nodes::Program, String, String), String> {
    let Some(path) = state.program_path.as_ref() else {
        return Err("no program loaded; use control-center serve --program <file.sd>".into());
    };
    parse_program_file(path)
}
