//! Minimal HTTP/1.1 helpers for the Spanda deploy agent protocol.
//!
pub mod fleet_continuity;
pub mod fleet_recovery;
pub mod fleet_tamper;

use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use rustls::{
    ClientConfig, ClientConnection, RootCertStore, ServerConfig, ServerConnection, StreamOwned,
};
use rustls_pemfile::{certs, private_key};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub path: String,
    pub use_tls: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub body: String,
    pub authorization: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeployAgentTls {
    pub cert_path: String,
    pub key_path: String,
}

pub fn parse_http_url(url: &str) -> Result<ParsedUrl, String> {
    // Description:

    //     Parse http url.

    //

    // Inputs:

    //     url: &str
    //         Deploy agent URL string (`http://` or `https://`).

    //

    // Outputs:

    //     result: Result<ParsedUrl, String>

    //         Return value from `parse_http_url`.

    //

    // Example:

    //     let result = spanda_deploy_http::parse_http_url(url);
    let (scheme, rest) = if let Some(tail) = url.strip_prefix("https://") {
        ("https", tail)
    } else if let Some(tail) = url.strip_prefix("http://") {
        ("http", tail)
    } else {
        return Err(format!(
            "deploy agent URL must start with http:// or https:// (got {url})"
        ));
    };
    let (authority, path) = match rest.split_once('/') {
        Some((auth, tail)) => (auth, format!("/{tail}")),
        None => (rest, "/".into()),
    };
    let default_port = if scheme == "https" { 443 } else { 80 };
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) => {
            let port = p
                .parse::<u16>()
                .map_err(|_| format!("invalid port in deploy agent URL '{url}'"))?;
            (h.to_string(), port)
        }
        None => (authority.to_string(), default_port),
    };
    Ok(ParsedUrl {
        scheme: scheme.into(),
        host,
        port,
        path,
        use_tls: scheme == "https",
    })
}

fn load_pem_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>, String> {
    // Description:
    //     Load pem certs.
    //
    // Inputs:
    //     path: &Path
    //         Caller-supplied path.
    //
    // Outputs:
    //     result: Result<Vec<CertificateDer<'static>>, String>
    //         Return value from `load_pem_certs`.
    //
    // Example:

    //     let result = spanda_deploy_http::load_pem_certs(path);

    let file = File::open(path).map_err(|e| format!("open cert '{}': {e}", path.display()))?;
    let mut reader = BufReader::new(file);
    certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("parse cert '{}': {e}", path.display()))
}

fn load_pem_key(path: &Path) -> Result<PrivateKeyDer<'static>, String> {
    // Description:
    //     Load pem key.
    //
    // Inputs:
    //     path: &Path
    //         Caller-supplied path.
    //
    // Outputs:
    //     result: Result<PrivateKeyDer<'static>, String>
    //         Return value from `load_pem_key`.
    //
    // Example:

    //     let result = spanda_deploy_http::load_pem_key(path);

    let file = File::open(path).map_err(|e| format!("open key '{}': {e}", path.display()))?;
    let mut reader = BufReader::new(file);
    private_key(&mut reader)
        .map_err(|e| format!("parse key '{}': {e}", path.display()))?
        .ok_or_else(|| format!("no private key found in '{}'", path.display()))
}

pub fn build_deploy_client_config() -> Result<Arc<ClientConfig>, String> {
    // Description:

    //     Build deploy client config.

    //

    // Inputs:

    //     None.

    //

    // Outputs:

    //     result: Result<Arc<ClientConfig>, String>

    //         Return value from `build_deploy_client_config`.

    //

    // Example:

    //     let result = spanda_deploy_http::build_deploy_client_config();
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    Ok(Arc::new(
        ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth(),
    ))
}

pub fn build_deploy_server_config(tls: &DeployAgentTls) -> Result<Arc<ServerConfig>, String> {
    // Description:

    //     Build deploy server config.

    //

    // Inputs:

    //     ls: &DeployAgentTls

    //         Caller-supplied ls.

    //

    // Outputs:

    //     result: Result<Arc<ServerConfig>, String>

    //         Return value from `build_deploy_server_config`.

    //

    // Example:

    //     let result = spanda_deploy_http::build_deploy_server_config(ls);
    let certs = load_pem_certs(Path::new(&tls.cert_path))?;
    let key = load_pem_key(Path::new(&tls.key_path))?;
    Ok(Arc::new(
        ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| format!("deploy agent TLS config: {e}"))?,
    ))
}

pub fn http_request(
    method: &str,
    url: &str,
    body: Option<&str>,
    token: Option<&str>,
) -> Result<HttpResponse, String> {
    // Description:

    //     Http request.

    //

    // Inputs:

    //     ethod: &str

    //         Caller-supplied ethod.

    //     url: &str
    //         Deploy agent URL string (`http://` or `https://`).

    //     body: Option<&str>

    //         Caller-supplied body.

    //     token: Option<&str>

    //         Caller-supplied token.

    //

    // Outputs:

    //     result: Result<HttpResponse, String>

    //         Return value from `http_request`.

    //

    // Example:

    //     let result = spanda_deploy_http::http_request(ethod, rl, body, oken);
    let parsed = parse_http_url(url)?;
    let payload = body.unwrap_or("");
    let mut request = format!(
        "{method} {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
        parsed.path,
        parsed.host,
        payload.len()
    );
    if let Some(token) = token {
        request.push_str(&format!("Authorization: Bearer {token}\r\n"));
    }
    request.push_str("\r\n");
    request.push_str(payload);
    let timeout = Duration::from_secs(30);
    let connect_timeout = Duration::from_secs(10);
    let socket_addr = format!("{}:{}", parsed.host, parsed.port);

    if parsed.use_tls {
        let config = build_deploy_client_config()?;
        let host = parsed.host.clone();
        let server_name = ServerName::try_from(host)
            .map_err(|_| format!("invalid TLS server name '{}'", parsed.host))?;
        let tcp = TcpStream::connect_timeout(
            &socket_addr
                .parse()
                .map_err(|_| format!("invalid address '{socket_addr}'"))?,
            connect_timeout,
        )
        .map_err(|e| format!("connect to {socket_addr} failed: {e}"))?;
        tcp.set_read_timeout(Some(timeout))
            .map_err(|e| format!("read timeout: {e}"))?;
        let conn = ClientConnection::new(config, server_name)
            .map_err(|e| format!("TLS client connection: {e}"))?;
        let mut tls = StreamOwned::new(conn, tcp);
        tls.write_all(request.as_bytes())
            .map_err(|e| format!("write request failed: {e}"))?;
        tls.sock
            .shutdown(Shutdown::Write)
            .map_err(|e| format!("shutdown request failed: {e}"))?;
        let mut raw = String::new();
        tls.read_to_string(&mut raw)
            .map_err(|e| format!("read response failed: {e}"))?;
        return parse_http_response(&raw);
    }

    let mut stream = TcpStream::connect_timeout(
        &socket_addr
            .parse()
            .map_err(|_| format!("invalid address '{socket_addr}'"))?,
        connect_timeout,
    )
    .map_err(|e| format!("connect to {socket_addr} failed: {e}"))?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|e| format!("read timeout: {e}"))?;
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("write request failed: {e}"))?;
    stream
        .shutdown(Shutdown::Write)
        .map_err(|e| format!("shutdown request failed: {e}"))?;
    let mut raw = String::new();
    stream
        .read_to_string(&mut raw)
        .map_err(|e| format!("read response failed: {e}"))?;
    parse_http_response(&raw)
}

pub fn parse_http_response(raw: &str) -> Result<HttpResponse, String> {
    // Description:

    //     Parse http response.

    //

    // Inputs:

    //     raw: &str

    //         Caller-supplied raw.

    //

    // Outputs:

    //     result: Result<HttpResponse, String>

    //         Return value from `parse_http_response`.

    //

    // Example:

    //     let result = spanda_deploy_http::parse_http_response(raw);
    let (head, body) = raw
        .split_once("\r\n\r\n")
        .ok_or_else(|| "invalid HTTP response".to_string())?;
    let status_line = head
        .lines()
        .next()
        .ok_or_else(|| "missing status line".to_string())?;
    let status = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| "missing HTTP status code".to_string())?
        .parse::<u16>()
        .map_err(|_| "invalid HTTP status code".to_string())?;
    Ok(HttpResponse {
        status,
        body: body.to_string(),
    })
}

pub fn parse_http_request(raw: &str) -> Result<HttpRequest, String> {
    // Description:

    //     Parse http request.

    //

    // Inputs:

    //     raw: &str

    //         Caller-supplied raw.

    //

    // Outputs:

    //     result: Result<HttpRequest, String>

    //         Return value from `parse_http_request`.

    //

    // Example:

    //     let result = spanda_deploy_http::parse_http_request(raw);
    let (head, body) = raw
        .split_once("\r\n\r\n")
        .ok_or_else(|| "invalid HTTP request".to_string())?;
    let mut lines = head.lines();
    let request_line = lines
        .next()
        .ok_or_else(|| "missing request line".to_string())?;
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| "missing HTTP method".to_string())?
        .to_string();
    let path = parts
        .next()
        .ok_or_else(|| "missing HTTP path".to_string())?
        .to_string();
    let mut authorization = None;
    for line in lines {
        if let Some(token) = line.strip_prefix("Authorization: Bearer ") {
            authorization = Some(token.trim().to_string());
        }
    }
    Ok(HttpRequest {
        method,
        path,
        body: body.to_string(),
        authorization,
    })
}

pub fn http_response(status: u16, body: &str) -> String {
    // Description:
    //     Http response.
    //
    // Inputs:
    //     status: u16
    //         Caller-supplied status.
    //     body: &str
    //         Caller-supplied body.
    //
    // Outputs:
    //     result: String
    //         Return value from `http_response`.
    //
    // Example:

    //     let result = spanda_deploy_http::http_response(status, body);

    format!(
        "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

pub fn serve_once(
    listener: &TcpListener,
    handler: impl Fn(HttpRequest) -> HttpResponse,
) -> Result<(), String> {
    // Description:

    //     Serve once.

    //

    // Inputs:

    //     listener: &TcpListener

    //         Caller-supplied listener.

    //

    // Outputs:

    //     result: Result<(), String>

    //         Return value from `serve_once`.

    //

    // Example:

    //     let result = spanda_deploy_http::serve_once(listener);
    let (mut stream, _) = listener
        .accept()
        .map_err(|e| format!("accept failed: {e}"))?;
    let mut raw = String::new();
    stream
        .read_to_string(&mut raw)
        .map_err(|e| format!("read request failed: {e}"))?;
    let request = parse_http_request(&raw)?;
    let response = handler(request);
    let encoded = http_response(response.status, &response.body);
    stream
        .write_all(encoded.as_bytes())
        .map_err(|e| format!("write response failed: {e}"))?;
    Ok(())
}

pub fn read_plain_request(stream: &mut TcpStream) -> Result<String, String> {
    // Description:
    //     Read plain request.
    //
    // Inputs:
    //     strea: &mut TcpStream
    //         Caller-supplied strea.
    //
    // Outputs:
    //     result: Result<String, String>
    //         Return value from `read_plain_request`.
    //
    // Example:

    //     let result = spanda_deploy_http::read_plain_request(strea);

    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
    let mut raw = String::new();
    stream
        .read_to_string(&mut raw)
        .map_err(|e| format!("read request failed: {e}"))?;
    Ok(raw)
}

pub fn write_plain_response(stream: &mut TcpStream, response: &HttpResponse) -> Result<(), String> {
    // Description:
    //     Write plain response.
    //
    // Inputs:
    //     strea: &mut TcpStream
    //         Caller-supplied strea.
    //     response: &HttpResponse
    //         Caller-supplied response.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `write_plain_response`.
    //
    // Example:

    //     let result = spanda_deploy_http::write_plain_response(strea, response);

    let encoded = http_response(response.status, &response.body);
    stream
        .write_all(encoded.as_bytes())
        .map_err(|e| format!("write response failed: {e}"))?;
    stream
        .shutdown(Shutdown::Write)
        .map_err(|e| format!("shutdown response failed: {e}"))
}

pub fn serve_tls_connection(
    server_config: &Arc<ServerConfig>,
    stream: TcpStream,
    handler: impl FnOnce(HttpRequest) -> HttpResponse,
) -> Result<(), String> {
    // Description:

    //     Serve tls connection.

    //

    // Inputs:

    //     server_config: &Arc<ServerConfig>

    //         Caller-supplied server config.

    //     strea: TcpStream

    //         Caller-supplied strea.

    //

    // Outputs:

    //     result: Result<(), String>

    //         Return value from `serve_tls_connection`.

    //

    // Example:

    //     let result = spanda_deploy_http::serve_tls_connection(server_config, strea);
    let conn = ServerConnection::new(Arc::clone(server_config))
        .map_err(|e| format!("TLS server connection: {e}"))?;
    let mut tls = StreamOwned::new(conn, stream);
    let _ = tls.sock.set_read_timeout(Some(Duration::from_secs(30)));
    let mut raw = String::new();
    tls.read_to_string(&mut raw)
        .map_err(|e| format!("read request failed: {e}"))?;
    let request = parse_http_request(&raw)?;
    let response = handler(request);
    let encoded = http_response(response.status, &response.body);
    tls.write_all(encoded.as_bytes())
        .map_err(|e| format!("write response failed: {e}"))?;
    tls.sock
        .shutdown(Shutdown::Write)
        .map_err(|e| format!("shutdown response failed: {e}"))
}

/// Return true when the bind address listens on a non-loopback interface.
pub fn bind_requires_agent_token(bind: &str) -> bool {
    // Description:
    //     Bind requires agent token.
    //
    // Inputs:
    //     bind: &str
    //         Caller-supplied bind.
    //
    // Outputs:
    //     result: bool
    //         Return value from `bind_requires_agent_token`.
    //
    // Example:

    //     let result = spanda_deploy_http::bind_requires_agent_token(bind);

    !is_loopback_host(bind_host(bind))
}

/// Validate agent startup options for public bind addresses.
pub fn ensure_agent_auth(
    bind: &str,
    token: &Option<String>,
    allow_unauthenticated: bool,
) -> Result<(), String> {
    // Description:
    //     Ensure agent auth.
    //
    // Inputs:
    //     bind: &str
    //         Caller-supplied bind.
    //     token: &Option<String>
    //         Caller-supplied token.
    //     allow_unauthenticated: bool
    //         Caller-supplied allow unauthenticated.
    //
    // Outputs:
    //     result: Result<(), String>
    //         Return value from `ensure_agent_auth`.
    //
    // Example:

    //     let result = spanda_deploy_http::ensure_agent_auth(bind, oken, allow_unauthenticated);

    if bind_requires_agent_token(bind) && token.is_none() && !allow_unauthenticated {
        return Err(format!(
            "binding to {bind} requires --token (or pass --allow-unauthenticated for lab use only)"
        ));
    }
    Ok(())
}

fn bind_host(bind: &str) -> &str {
    // Description:
    //     Bind host.
    //
    // Inputs:
    //     bind: &str
    //         Caller-supplied bind.
    //
    // Outputs:
    //     result: &str
    //         Return value from `bind_host`.
    //
    // Example:

    //     let result = spanda_deploy_http::bind_host(bind);

    let host = bind.rsplit_once(':').map(|(h, _)| h).unwrap_or(bind);
    host.trim_matches(|c| c == '[' || c == ']')
}

fn is_loopback_host(host: &str) -> bool {
    // Description:
    //     Is loopback host.
    //
    // Inputs:
    //     hos: &str
    //         Caller-supplied hos.
    //
    // Outputs:
    //     result: bool
    //         Return value from `is_loopback_host`.
    //
    // Example:

    //     let result = spanda_deploy_http::is_loopback_host(hos);

    matches!(host, "127.0.0.1" | "localhost" | "::1")
}

#[cfg(test)]
mod agent_bind_tests {
    use super::*;

    #[test]
    fn loopback_bind_allows_missing_token() {
        // Description:
        //     Loopback bind allows missing token.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_deploy_http::loopback_bind_allows_missing_token();

        assert!(!bind_requires_agent_token("127.0.0.1:8765"));
        assert!(ensure_agent_auth("127.0.0.1:8765", &None, false).is_ok());
    }

    #[test]
    fn public_bind_requires_token() {
        // Description:
        //     Public bind requires token.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     None.
        //
        // Example:

        //     let result = spanda_deploy_http::public_bind_requires_token();

        assert!(bind_requires_agent_token("0.0.0.0:8765"));
        assert!(ensure_agent_auth("0.0.0.0:8765", &None, false).is_err());
        assert!(ensure_agent_auth("0.0.0.0:8765", &None, true).is_ok());
    }
}

pub use fleet_continuity::{
    relay_continuity_via_mesh, FleetContinuityRequest, FleetContinuityResponse,
};
pub use fleet_recovery::{relay_recovery_via_mesh, FleetRecoveryRequest, FleetRecoveryResponse};
pub use fleet_tamper::{
    fetch_fleet_tamper_report, ingest_fleet_tamper_trace, FleetTamperIngestRequest,
    FleetTamperIngestResponse,
};
