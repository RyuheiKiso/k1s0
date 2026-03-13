use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::fs;

use crate::error::AppUpdaterError;

pub struct ChecksumVerifier;

impl ChecksumVerifier {
    pub async fn calculate(file_path: &Path) -> Result<String, AppUpdaterError> {
        let bytes = fs::read(file_path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    pub async fn verify(file_path: &Path, expected: &str) -> Result<bool, AppUpdaterError> {
        let actual = Self::calculate(file_path).await?;
        Ok(actual == expected.to_lowercase())
    }

    pub async fn verify_or_error(file_path: &Path, expected: &str) -> Result<(), AppUpdaterError> {
        let verified = Self::verify(file_path, expected).await?;
        if !verified {
            return Err(AppUpdaterError::Checksum(
                "file checksum did not match expected value".to_string(),
            ));
        }
        Ok(())
    }
}
