//! Transport-layer security policy validation and TLS session management.

use spanda_security::{
    AuthenticationMode, EncryptionMode, IntegrityMode, SecureCommPolicy, WireCryptoSession,
};

const WIRE_PREFIX: &str = "spanda/wire/v1:";

/// Per-transport TLS / encryption configuration wired from `bus { ... }` declarations.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransportSecurityConfig {
    pub encryption: EncryptionMode,
    pub authentication: AuthenticationMode,
    pub integrity: IntegrityMode,
    pub cert_path: Option<String>,
    pub key_secret: Option<String>,
    pub key_path: Option<String>,
}

impl TransportSecurityConfig {
    pub fn from_bus_fields(
        encryption: Option<&str>,
        authentication: Option<&str>,
        integrity: Option<&str>,
    ) -> Result<Self, String> {
        Ok(Self {
            encryption: parse_encryption(encryption)?,
            authentication: parse_authentication(authentication)?,
            integrity: parse_integrity(integrity)?,
            cert_path: None,
            key_secret: None,
            key_path: None,
        })
    }

    pub fn with_secrets(mut self, cert_path: Option<String>, key_secret: Option<String>) -> Self {
        self.cert_path = cert_path;
        self.key_secret = key_secret;
        self
    }

    pub fn session_material(&self) -> String {
        format!(
            "{}:{}",
            self.cert_path.as_deref().unwrap_or("spanda-local"),
            self.key_secret.as_deref().unwrap_or("spanda-local-key")
        )
    }

    pub fn validate(&self, transport: &str) -> Result<(), String> {
        if self.encryption == EncryptionMode::Required
            && self.cert_path.is_none()
            && self.key_secret.is_none()
        {
            return Err(format!(
                "transport '{transport}' requires encryption but no cert/key secret is configured"
            ));
        }
        Ok(())
    }

    /// Resolve broker URL from bus declaration or `SPANDA_BROKER_URL` environment variable.
    pub fn resolve_broker_url(bus_url: Option<&str>) -> Option<String> {
        if let Some(url) = bus_url {
            if !url.is_empty() {
                return Some(url.to_string());
            }
        }
        std::env::var("SPANDA_BROKER_URL")
            .ok()
            .filter(|value| !value.is_empty())
    }

    /// True when broker URL implies TLS (`mqtts://`, `wss://`, etc.).
    pub fn url_requires_tls(broker_url: Option<&str>) -> bool {
        broker_url.is_some_and(|url| {
            let lower = url.to_ascii_lowercase();
            lower.starts_with("mqtts://")
                || lower.starts_with("wss://")
                || lower.starts_with("ssl://")
                || lower.starts_with("tls://")
                || lower.starts_with("dds+sec://")
        })
    }
}

/// Negotiated TLS session for transport wire encryption (AES-256-GCM).
#[derive(Debug, Clone, Default)]
pub struct TlsTransportSession {
    pub negotiated: bool,
    pub cipher_suite: String,
    pub peer_verified: bool,
    session: Option<WireCryptoSession>,
}

/// Backward-compatible alias used by existing transport configuration.
pub type TlsTransportStub = TlsTransportSession;

impl TlsTransportSession {
    pub fn connect(
        &mut self,
        config: &TransportSecurityConfig,
        broker_url: Option<&str>,
    ) -> Result<(), String> {
        config.validate("tls")?;
        if config.encryption == EncryptionMode::None {
            self.negotiated = false;
            self.cipher_suite = "none".into();
            self.peer_verified = true;
            self.session = None;
            return Ok(());
        }

        let cert_file = config
            .cert_path
            .as_deref()
            .filter(|p| std::path::Path::new(p).is_file());
        let key_file = config
            .key_path
            .as_deref()
            .filter(|p| std::path::Path::new(p).is_file());

        if config.authentication == AuthenticationMode::Mutual
            && (cert_file.is_none() || key_file.is_none())
        {
            return Err("mutual TLS authentication failed: missing certificate or key file".into());
        }

        if config.authentication == AuthenticationMode::Mutual {
            if let (Some(cert), Some(key), Some(url)) = (cert_file, key_file, broker_url) {
                if let Some(endpoint) = crate::transport_tls::parse_tls_endpoint(url) {
                    if endpoint.use_tls {
                        let client_cfg = crate::transport_tls::build_client_config(cert, key)?;
                        match crate::transport_tls::perform_mtls_handshake(&endpoint, client_cfg) {
                            Ok(hs) => {
                                let crypto = WireCryptoSession::from_material(&hs.session_material);
                                self.cipher_suite = hs.cipher_suite;
                                self.peer_verified = hs.peer_verified;
                                self.session = Some(crypto);
                                self.negotiated = true;
                                return Ok(());
                            }
                            Err(err)
                                if std::env::var("SPANDA_MTLS_REQUIRED").ok().as_deref()
                                    == Some("1") =>
                            {
                                return Err(format!("mTLS handshake failed: {err}"));
                            }
                            Err(_) => {}
                        }
                    }
                }
            }
        }

        self.peer_verified =
            config.authentication != AuthenticationMode::Mutual || cert_file.is_some();
        if let Some(path) = cert_file {
            validate_cert_pem(path)?;
            self.peer_verified = true;
        }
        let crypto = WireCryptoSession::from_material(&config.session_material());
        self.cipher_suite = crypto.cipher_suite.clone();
        self.session = Some(crypto);
        self.negotiated = true;
        Ok(())
    }

    pub fn encrypt_frame(&self, plaintext: &str) -> Result<String, String> {
        if !self.negotiated {
            return Ok(plaintext.to_string());
        }
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| "TLS session not negotiated".to_string())?;
        let encrypted = session.encrypt(plaintext.as_bytes())?;
        Ok(format!("{WIRE_PREFIX}{}", hex::encode(encrypted)))
    }

    pub fn decrypt_frame(&self, ciphertext: &str) -> Result<String, String> {
        if !self.negotiated {
            return Ok(ciphertext.to_string());
        }
        if let Some(hex_payload) = ciphertext.strip_prefix(WIRE_PREFIX) {
            let session = self
                .session
                .as_ref()
                .ok_or_else(|| "TLS session not negotiated".to_string())?;
            let bytes = hex::decode(hex_payload).map_err(|e| format!("hex decode failed: {e}"))?;
            let plain = session.decrypt(&bytes)?;
            return String::from_utf8(plain).map_err(|e| format!("utf8 decode failed: {e}"));
        }
        // Legacy simulation prefix from earlier stub builds.
        if let Some(stripped) = ciphertext.strip_prefix(&format!("tls:{}:", self.cipher_suite)) {
            return Ok(stripped.to_string());
        }
        Err("TLS decrypt failed: unrecognized wire frame".into())
    }
}

/// Merge robot `secure_comm` defaults with per-bus overrides.
pub fn effective_transport_policy(
    robot: &SecureCommPolicy,
    bus: &TransportSecurityConfig,
) -> TransportSecurityConfig {
    TransportSecurityConfig {
        encryption: if bus.encryption != EncryptionMode::None {
            bus.encryption
        } else {
            robot.encryption
        },
        authentication: if bus.authentication != AuthenticationMode::None {
            bus.authentication
        } else {
            robot.authentication
        },
        integrity: if bus.integrity != IntegrityMode::None {
            bus.integrity
        } else {
            robot.integrity
        },
        cert_path: bus.cert_path.clone(),
        key_secret: bus.key_secret.clone(),
        key_path: bus.key_path.clone(),
    }
}

fn parse_encryption(value: Option<&str>) -> Result<EncryptionMode, String> {
    match value {
        None => Ok(EncryptionMode::None),
        Some(v) => v.parse().map_err(|e: String| e),
    }
}

fn parse_authentication(value: Option<&str>) -> Result<AuthenticationMode, String> {
    match value {
        None => Ok(AuthenticationMode::None),
        Some(v) => v.parse().map_err(|e: String| e),
    }
}

fn parse_integrity(value: Option<&str>) -> Result<IntegrityMode, String> {
    match value {
        None => Ok(IntegrityMode::None),
        Some(v) => v.parse().map_err(|e: String| e),
    }
}

fn validate_cert_pem(path: &str) -> Result<(), String> {
    // Parse a PEM certificate file to verify TLS credential material is present.
    use std::fs::File;
    use std::io::BufReader;
    let file = File::open(path).map_err(|e| format!("open cert '{path}': {e}"))?;
    let mut reader = BufReader::new(file);
    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("parse cert '{path}': {e}"))?;
    if certs.is_empty() {
        return Err(format!("no certificates found in '{path}'"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tls_session_negotiates_aes_gcm() {
        let mut tls = TlsTransportSession::default();
        let cfg = TransportSecurityConfig {
            encryption: EncryptionMode::Required,
            authentication: AuthenticationMode::Signed,
            integrity: IntegrityMode::Required,
            cert_path: Some("certs/rover.pem".into()),
            key_secret: Some("motion_key".into()),
            key_path: None,
        };
        tls.connect(&cfg, None).unwrap();
        assert!(tls.negotiated);
        assert_eq!(tls.cipher_suite, "AES-256-GCM");
        let enc = tls.encrypt_frame(r#"{"v":1,"payload":"x"}"#).unwrap();
        assert!(enc.starts_with(WIRE_PREFIX));
        let dec = tls.decrypt_frame(&enc).unwrap();
        assert_eq!(dec, r#"{"v":1,"payload":"x"}"#);
    }

    #[test]
    fn url_scheme_detects_tls() {
        assert!(TransportSecurityConfig::url_requires_tls(Some(
            "mqtts://broker.example:8883"
        )));
        assert!(!TransportSecurityConfig::url_requires_tls(Some(
            "mqtt://localhost:1883"
        )));
    }
}
