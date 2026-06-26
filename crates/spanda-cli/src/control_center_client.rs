//! REST v1 client for remote Control Center API calls.
//!
use spanda_deploy_http::{http_request, HttpResponse};
use std::env;

/// HTTP client for Control Center REST v1 (`SPANDA_CONTROL_CENTER_URL`, `SPANDA_API_KEY`).
pub struct ControlCenterClient {
    base_url: String,
    api_key: Option<String>,
}

impl ControlCenterClient {
    pub fn from_env() -> Self {
        // Build a client from environment defaults.
        //
        // Parameters:
        // None.
        //
        // Returns:
        // Client using `SPANDA_CONTROL_CENTER_URL` and optional `SPANDA_API_KEY`.
        //
        // Options:
        // `SPANDA_CONTROL_CENTER_URL` defaults to `http://127.0.0.1:8080`.
        //
        // Example:
        // let client = ControlCenterClient::from_env();

        let base_url = env::var("SPANDA_CONTROL_CENTER_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8080".into());
        let api_key = env::var("SPANDA_API_KEY").ok();
        Self { base_url, api_key }
    }

    pub fn with_url(mut self, base_url: String) -> Self {
        // Override the API base URL for this client instance.
        //
        // Parameters:
        // - `base_url` — origin without trailing slash (e.g. `http://127.0.0.1:8080`)
        //
        // Returns:
        // Updated client.
        //
        // Options:
        // None.
        //
        // Example:
        // let client = ControlCenterClient::from_env().with_url("http://10.0.0.5:8080".into());

        self.base_url = base_url.trim_end_matches('/').to_string();
        self
    }

    pub fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
        auth: bool,
    ) -> Result<HttpResponse, String> {
        // Issue an HTTP request against the Control Center API.
        //
        // Parameters:
        // - `method` — HTTP verb (`GET`, `POST`, …)
        // - `path` — API path starting with `/v1/`
        // - `body` — optional JSON request body
        // - `auth` — attach `Authorization: Bearer` when `SPANDA_API_KEY` is set
        //
        // Returns:
        // Parsed HTTP response, or a transport/parse error string.
        //
        // Options:
        // None.
        //
        // Example:
        // let resp = client.request("GET", "/v1/dashboard", None, false)?;

        let path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        };
        let url = format!("{}{}", self.base_url, path);
        let token = if auth { self.api_key.as_deref() } else { None };
        http_request(method, &url, body, token)
    }

    pub fn get(&self, path: &str, auth: bool) -> Result<HttpResponse, String> {
        self.request("GET", path, None, auth)
    }

    pub fn post(&self, path: &str, body: &str, auth: bool) -> Result<HttpResponse, String> {
        self.request("POST", path, Some(body), auth)
    }
}
