//! Mission approval queue for operator workflows in Control Center.
//!
use crate::error::{ConfigError, ConfigResult};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Lifecycle for a human mission approval request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissionApprovalStatus {
    Pending,
    Approved,
    Rejected,
}

/// Declarative pending mission in TOML (`[[mission_approvals]]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionApprovalSeed {
    pub id: String,
    pub mission_id: String,
    #[serde(default)]
    pub requested_by: Option<String>,
    #[serde(default)]
    pub requires_capability: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

/// One mission approval record in the persisted queue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionApprovalRecord {
    pub id: String,
    pub mission_id: String,
    #[serde(default)]
    pub requested_by: Option<String>,
    pub status: MissionApprovalStatus,
    pub created_at_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Persisted mission approval queue.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MissionApprovalQueue {
    pub requests: Vec<MissionApprovalRecord>,
}

/// Default path for mission approval queue persistence.
pub fn default_mission_approvals_path() -> PathBuf {
    PathBuf::from(".spanda/mission-approvals.json")
}

/// Load mission approval queue from disk.
pub fn load_mission_approval_queue(path: &Path) -> ConfigResult<MissionApprovalQueue> {
    if !path.exists() {
        return Ok(MissionApprovalQueue::default());
    }
    let text = fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| ConfigError::JsonParse {
        path: path.to_path_buf(),
        source,
    })
}

/// Persist mission approval queue to disk.
pub fn save_mission_approval_queue(path: &Path, queue: &MissionApprovalQueue) -> ConfigResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| ConfigError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let text = serde_json::to_string_pretty(queue).map_err(|source| ConfigError::JsonParse {
        path: path.to_path_buf(),
        source,
    })?;
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .map_err(|source| ConfigError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    file.write_all(text.as_bytes())
        .map_err(|source| ConfigError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    Ok(())
}

/// Merge config-declared seeds with persisted queue (seeds only add missing ids).
pub fn merge_mission_approval_seeds(
    queue: &mut MissionApprovalQueue,
    seeds: &[MissionApprovalSeed],
    now_ms: f64,
) {
    for seed in seeds {
        if queue.requests.iter().any(|r| r.id == seed.id) {
            continue;
        }
        let status = seed
            .status
            .as_deref()
            .map(|s| s.to_ascii_lowercase())
            .map(|s| {
                if s == "approved" {
                    MissionApprovalStatus::Approved
                } else if s == "rejected" {
                    MissionApprovalStatus::Rejected
                } else {
                    MissionApprovalStatus::Pending
                }
            })
            .unwrap_or(MissionApprovalStatus::Pending);
        queue.requests.push(MissionApprovalRecord {
            id: seed.id.clone(),
            mission_id: seed.mission_id.clone(),
            requested_by: seed.requested_by.clone(),
            status,
            created_at_ms: now_ms,
            resolved_at_ms: None,
            resolver: None,
            note: seed.note.clone(),
        });
    }
}

/// Resolve a pending mission approval by id.
pub fn resolve_mission_approval(
    queue: &mut MissionApprovalQueue,
    id: &str,
    approved: bool,
    resolver: &str,
    now_ms: f64,
) -> ConfigResult<MissionApprovalRecord> {
    let record = queue
        .requests
        .iter_mut()
        .find(|r| r.id == id || r.mission_id == id)
        .ok_or_else(|| ConfigError::Approval {
            detail: format!("mission approval not found: {id}"),
        })?;
    record.status = if approved {
        MissionApprovalStatus::Approved
    } else {
        MissionApprovalStatus::Rejected
    };
    record.resolved_at_ms = Some(now_ms);
    record.resolver = Some(resolver.to_string());
    Ok(record.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_seeds_adds_pending_mission() {
        let mut queue = MissionApprovalQueue::default();
        merge_mission_approval_seeds(
            &mut queue,
            &[MissionApprovalSeed {
                id: "pick-001".into(),
                mission_id: "pick_mission".into(),
                requested_by: Some("operator-001".into()),
                requires_capability: Some("approve_mission".into()),
                status: Some("pending".into()),
                note: None,
            }],
            1.0,
        );
        assert_eq!(queue.requests.len(), 1);
        assert_eq!(queue.requests[0].status, MissionApprovalStatus::Pending);
    }
}
