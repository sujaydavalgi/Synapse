//! Optional LLM refinement for generated Spanda scaffolds.
//!
use serde_json::json;
use std::process::{Command, Stdio};

/// Attempt to refine a template scaffold through an external LLM HTTP endpoint.
pub fn refine_with_llm(kind: &str, template: &str) -> Result<String, String> {
    // POST the template to SPANDA_LLM_ENDPOINT and return refined Spanda source.
    //
    // Parameters:
    // - `kind` — generation kind label (`mission`, `robot`, `health-policy`)
    // - `template` — validated template scaffold to refine
    //
    // Returns:
    // Refined source text, or an error when the endpoint is missing or fails.
    //
    // Options:
    // Requires `SPANDA_LLM_ENDPOINT` (HTTP URL). Optional `SPANDA_LLM_API_KEY` bearer token.
    //
    // Example:
    // let refined = refine_with_llm("mission", &template)?;

    let endpoint = std::env::var("SPANDA_LLM_ENDPOINT")
        .map_err(|_| "SPANDA_LLM_ENDPOINT not set".to_string())?;
    let body = json!({
        "kind": kind,
        "prompt": format!(
            "Refine this Spanda ({kind}) scaffold. Return only valid Spanda source.\n\n{template}"
        ),
        "template": template,
    });
    let mut command = Command::new("curl");
    command
        .arg("-sS")
        .arg("-X")
        .arg("POST")
        .arg(&endpoint)
        .arg("-H")
        .arg("Content-Type: application/json");
    if let Ok(token) = std::env::var("SPANDA_LLM_API_KEY") {
        command.arg("-H").arg(format!("Authorization: Bearer {token}"));
    }
    command
        .arg("-d")
        .arg(body.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let output = command
        .output()
        .map_err(|error| format!("failed to invoke curl for LLM endpoint: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("LLM endpoint request failed: {stderr}"));
    }
    let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
    extract_llm_source(&response).ok_or_else(|| {
        "LLM response did not contain Spanda source (expected JSON {\"source\":...} or raw .sd)"
            .to_string()
    })
}

fn extract_llm_source(response: &str) -> Option<String> {
    // Accept either a JSON envelope or raw Spanda source from the LLM gateway.
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(response) {
        if let Some(source) = value.get("source").and_then(|field| field.as_str()) {
            return Some(source.to_string());
        }
        if let Some(text) = value
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
        {
            return Some(text.to_string());
        }
    }
    if response.contains("hardware ") || response.contains("robot ") {
        return Some(response.to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_source_from_json_envelope() {
        let response = r#"{"source":"robot R { }"}"#;
        assert_eq!(
            extract_llm_source(response),
            Some("robot R { }".to_string())
        );
    }
}
