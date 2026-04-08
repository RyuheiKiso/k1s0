use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::fs;

use crate::error::AppUpdaterError;

/// ファイルの SHA-256 チェックサムを計算・検証するユーティリティ
pub struct ChecksumVerifier;

impl ChecksumVerifier {
    /// ファイルの SHA-256 チェックサムを計算して16進数文字列で返す
    pub async fn calculate(file_path: &Path) -> Result<String, AppUpdaterError> {
        let bytes = fs::read(file_path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();
        Ok(format!("{result:x}"))
    }

    /// ファイルのチェックサムが期待値と一致するかを検証する
    pub async fn verify(file_path: &Path, expected: &str) -> Result<bool, AppUpdaterError> {
        let actual = Self::calculate(file_path).await?;
        Ok(actual == expected.to_lowercase())
    }

    /// ファイルのチェックサムが期待値と一致しない場合はエラーを返す
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
