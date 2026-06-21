//! Minimal HTTP/1.1 helpers for the Spanda deploy agent protocol.

use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use rustls::{ClientConfig, ClientConnection, RootCertStore, ServerConfig, ServerConnection, StreamOwned};
use rustls_pemfile::{certs, private_key};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;

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
    // Parse an http(s)://host:port/path URL for deploy agent calls.
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
    let file = File::open(path).map_err(|e| format!("open cert '{}': {e}", path.display()))?;
    let mut reader = BufReader::new(file);
    certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("parse cert '{}': {e}", path.display()))
}

fn load_pem_key(path: &Path) -> Result<PrivateKeyDer<'static>, String> {
    let file = File::open(path).map_err(|e| format!("open key '{}': {e}", path.display()))?;
    let mut reader = BufReader::new(file);
    private_key(&mut reader)
        .map_err(|e| format!("parse key '{}': {e}", path.display()))?
        .ok_or_else(|| format!("no private key found in '{}'", path.display()))
}

pub fn build_deploy_client_config() -> Result<Arc<ClientConfig>, String> {
    // Build a rustls client config using the public WebPKI trust store.
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    Ok(Arc::new(
        ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth(),
    ))
}

pub fn build_deploy_server_config(tls: &DeployAgentTls) -> Result<Arc<ServerConfig>, String> {
    // Load PEM server credentials for HTTPS deploy agents.
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
    // Issue a single HTTP/1.1 request and return the response body.
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

    if parsed.use_tls {
        let config = build_deploy_client_config()?;
        let host = parsed.host.clone();
        let server_name = ServerName::try_from(host)
            .map_err(|_| format!("invalid TLS server name '{}'", parsed.host))?;
        let tcp = TcpStream::connect(format!("{}:{}", parsed.host, parsed.port))
            .map_err(|e| format!("connect to {}:{} failed: {e}", parsed.host, parsed.port))?;
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

    let mut stream = TcpStream::connect(format!("{}:{}", parsed.host, parsed.port))
        .map_err(|e| format!("connect to {}:{} failed: {e}", parsed.host, parsed.port))?;
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
    // Split an HTTP response into status code and body.
    let (head, body) = raw
        .split_once("\r\n\r\n")
        .ok_or_else(|| "invalid HTTP response".to_string())?;
    let status_line = head.lines().next().ok_or_else(|| "missing status line".to_string())?;
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
    // Parse a minimal HTTP/1.1 request for the deploy agent server.
    let (head, body) = raw
        .split_once("\r\n\r\n")
        .ok_or_else(|| "invalid HTTP request".to_string())?;
    let mut lines = head.lines();
    let request_line = lines.next().ok_or_else(|| "missing request line".to_string())?;
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
    format!(
        "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

pub fn serve_once(listener: &TcpListener, handler: impl Fn(HttpRequest) -> HttpResponse) -> Result<(), String> {
    // Accept one HTTP connection and write the handler response.
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
    let mut raw = String::new();
    stream
        .read_to_string(&mut raw)
        .map_err(|e| format!("read request failed: {e}"))?;
    Ok(raw)
}

pub fn write_plain_response(stream: &mut TcpStream, response: &HttpResponse) -> Result<(), String> {
    let encoded = http_response(response.status, &response.body);
    stream
        .write_all(encoded.as_bytes())
        .map_err(|e| format!("write response failed: {e}"))
}

pub fn serve_tls_connection(
    server_config: &Arc<ServerConfig>,
    stream: TcpStream,
    handler: impl FnOnce(HttpRequest) -> HttpResponse,
) -> Result<(), String> {
    // Complete one HTTPS request/response cycle on an accepted TCP connection.
    let conn = ServerConnection::new(Arc::clone(server_config))
        .map_err(|e| format!("TLS server connection: {e}"))?;
    let mut tls = StreamOwned::new(conn, stream);
    let mut raw = String::new();
    tls.read_to_string(&mut raw)
        .map_err(|e| format!("read request failed: {e}"))?;
    let request = parse_http_request(&raw)?;
    let response = handler(request);
    let encoded = http_response(response.status, &response.body);
    tls.write_all(encoded.as_bytes())
        .map_err(|e| format!("write response failed: {e}"))
}
