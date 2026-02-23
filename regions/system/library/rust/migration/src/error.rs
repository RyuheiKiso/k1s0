use thiserror::Error;

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    #[error("migration failed at version {version}: {message}")]
    MigrationFailed { version: String, message: String },

    #[error("checksum mismatch for version {version}: expected {expected}, actual {actual}")]
    ChecksumMismatch {
        version: String,
        expected: String,
        actual: String,
    },

    #[error("directory not found: {0}")]
    DirectoryNotFound(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
