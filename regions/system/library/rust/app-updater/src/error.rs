#[derive(Debug, thiserror::Error)]
pub enum AppUpdaterError {
    #[error("connection error: {0}")]
    Connection(String),
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("app not found: {0}")]
    AppNotFound(String),
    #[error("version not found: {0}")]
    VersionNotFound(String),
    #[error("checksum mismatch: {0}")]
    Checksum(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
