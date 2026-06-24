//! Error types for persistent telemetry storage.

use thiserror::Error;

/// Errors from reading or writing the telemetry store.
#[derive(Debug, Error)]
pub enum TelemetryStoreError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("telemetry store lock poisoned")]
    LockPoisoned,
}

pub type TelemetryStoreResult<T> = Result<T, TelemetryStoreError>;
