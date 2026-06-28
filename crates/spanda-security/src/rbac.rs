//! Role-based access control for Spanda Control Center and APIs.
//!
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enterprise operator roles (v1 — four primary roles plus safety and audit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Administrator,
    Developer,
    Operator,
    Supervisor,
    SafetyOfficer,
    Auditor,
    Guest,
}

impl Role {
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "administrator" | "admin" => Self::Administrator,
            "developer" | "dev" => Self::Developer,
            "operator" => Self::Operator,
            "supervisor" => Self::Supervisor,
            "safety_officer" | "safety" => Self::SafetyOfficer,
            "auditor" => Self::Auditor,
            "guest" => Self::Guest,
            _ => Self::Guest,
        }
    }
}

/// Mutating actions guarded by RBAC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RbacAction {
    Deploy,
    Operate,
    Approve,
    Override,
    Shutdown,
    Recover,
    Delete,
    Provision,
}

/// Authenticated request context after API key validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RbacContext {
    pub key_id: String,
    pub role: Role,
    #[serde(default = "default_tenant_field")]
    pub tenant_id: String,
}

fn default_tenant_field() -> String {
    crate::tenant::default_tenant_id()
}

/// API key record (token value is stored hashed or as opaque secret).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiKeyRecord {
    pub key_id: String,
    pub token: String,
    pub role: Role,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default = "default_tenant_field")]
    pub tenant_id: String,
}

/// In-memory API key store for Control Center v1.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiKeyStore {
    pub keys: Vec<ApiKeyRecord>,
}

impl ApiKeyStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_env() -> Self {
        let tenant_id = crate::tenant::default_tenant_id();
        let mut store = Self::new();
        if let Ok(token) = std::env::var("SPANDA_API_KEY") {
            store.keys.push(ApiKeyRecord {
                key_id: "env-default".into(),
                token,
                role: Role::Administrator,
                label: Some("SPANDA_API_KEY".into()),
                tenant_id: tenant_id.clone(),
            });
        }
        store
    }

    pub fn from_env_and_file() -> Self {
        let mut store = Self::from_env();
        if let Ok(path) = std::env::var("SPANDA_API_KEYS_FILE") {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(keys) = serde_json::from_str::<Vec<ApiKeyRecord>>(&content) {
                    store.keys.extend(keys);
                }
            }
        }
        store
    }

    pub fn authenticate(&self, bearer: Option<&str>) -> Option<RbacContext> {
        let token = bearer?.trim();
        if token.is_empty() {
            return None;
        }
        self.keys
            .iter()
            .find(|k| k.token == token)
            .map(|k| RbacContext {
                key_id: k.key_id.clone(),
                role: k.role,
                tenant_id: k.tenant_id.clone(),
            })
    }

    pub fn check_tenant(ctx: Option<&RbacContext>, server_tenant: &str) -> bool {
        ctx.is_some_and(|context| crate::tenant::tenant_matches(server_tenant, &context.tenant_id))
    }

    pub fn check_scoped(
        ctx: Option<&RbacContext>,
        server_tenant: &str,
        action: RbacAction,
    ) -> bool {
        match ctx {
            Some(context) if crate::tenant::tenant_matches(server_tenant, &context.tenant_id) => {
                Self::authorize(context.role, action)
            }
            _ => false,
        }
    }

    pub fn authorize(role: Role, action: RbacAction) -> bool {
        use RbacAction::*;
        use Role::*;
        match (role, action) {
            (Administrator, _) => true,
            (Supervisor, _) => !matches!(action, Delete),
            (Developer, Deploy | Operate) => true,
            (Operator, Operate | Shutdown | Recover) => true,
            (SafetyOfficer, Operate | Approve | Shutdown) => true,
            (Auditor, _) => false,
            (Guest, _) => false,
            _ => false,
        }
    }

    pub fn check(ctx: Option<&RbacContext>, action: RbacAction) -> bool {
        match ctx {
            Some(c) => Self::authorize(c.role, action),
            None => false,
        }
    }
}

/// Permission matrix for documentation and UI.
pub fn permission_matrix() -> HashMap<String, Vec<String>> {
    let roles = [
        Role::Administrator,
        Role::Developer,
        Role::Operator,
        Role::Supervisor,
        Role::SafetyOfficer,
        Role::Auditor,
        Role::Guest,
    ];
    let actions = [
        RbacAction::Deploy,
        RbacAction::Operate,
        RbacAction::Approve,
        RbacAction::Override,
        RbacAction::Shutdown,
        RbacAction::Recover,
        RbacAction::Delete,
        RbacAction::Provision,
    ];
    let mut matrix = HashMap::new();
    for role in roles {
        let allowed: Vec<String> = actions
            .iter()
            .filter(|a| ApiKeyStore::authorize(role, **a))
            .map(|a| format!("{a:?}"))
            .collect();
        matrix.insert(format!("{role:?}"), allowed);
    }
    matrix
}

/// Generate a random 256-bit Control Center API key token (hex-encoded).
pub fn generate_api_key_token() -> String {
    // Produce a cryptographically random Bearer token for operator auth.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Lowercase hex string (64 characters).
    //
    // Options:
    // None.
    //
    // Example:
    // let token = generate_api_key_token();

    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_can_recover_not_deploy() {
        assert!(ApiKeyStore::authorize(Role::Operator, RbacAction::Recover));
        assert!(!ApiKeyStore::authorize(Role::Operator, RbacAction::Deploy));
    }

    #[test]
    fn generated_api_key_token_is_64_hex_chars() {
        let token = generate_api_key_token();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn env_key_authenticates() {
        std::env::set_var("SPANDA_API_KEY", "test-token-123");
        let store = ApiKeyStore::from_env();
        let ctx = store.authenticate(Some("test-token-123"));
        assert_eq!(ctx.unwrap().role, Role::Administrator);
        std::env::remove_var("SPANDA_API_KEY");
    }
}
