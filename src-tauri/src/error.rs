use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid JSON at {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("unsupported profile schema: {0}")]
    UnsupportedSchema(String),
    #[error("profile is invalid: {0}")]
    InvalidProfile(String),
    #[error("migration conflict: {0}")]
    Conflict(String),
    #[error("unsafe path: {0}")]
    UnsafePath(PathBuf),
    #[error("Bambu Studio is still running")]
    BambuStillRunning,
    #[error("operation was cancelled")]
    Cancelled,
}

impl AppError {
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
